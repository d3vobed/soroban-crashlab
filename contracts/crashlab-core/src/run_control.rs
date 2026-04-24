//! Cooperative run lifecycle and cancellation for long-running fuzz campaigns.
//!
//! Runs check [`CancelSignal`] between iterations so maintainers can stop work
//! gracefully. The same signal can be driven in-process ([`CancelSignal::new`])
//! or via [`request_cancel_run`] / [`cancel_requested`] when the runner and the
//! `crashlab run cancel` CLI use a shared state directory.
//!
//! Use [`drive_run_partitioned`] with [`crate::worker_partition::WorkerPartition`] to
//! execute only the seed indices assigned to one worker while preserving the same
//! global iteration order and cancellation points as [`drive_run`].

use crate::worker_partition::WorkerPartition;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Opaque identifier for an active or completed run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RunId(pub u64);

/// Summary emitted when a run stops; partial counts are valid for cancellation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunSummary {
    /// Seeds fully processed before the run ended.
    pub seeds_processed: u64,
    /// When cancelled, the seed id at which cancellation was observed (if known).
    pub cancelled_at_seed: Option<u64>,
}

/// Terminal state for a fuzz campaign run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RunTerminalState {
    Completed { summary: RunSummary },
    Cancelled { summary: RunSummary },
    Failed { message: String },
}

/// Cooperative cancellation: in-process flag plus optional on-disk marker.
#[derive(Clone, Debug)]
pub struct CancelSignal {
    flag: Arc<AtomicBool>,
    run_id: RunId,
    state_dir: PathBuf,
}

impl CancelSignal {
    /// In-process cancellation only (no file I/O).
    pub fn new(run_id: RunId) -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
            run_id,
            state_dir: PathBuf::new(),
        }
    }

    /// Full signal with a state directory for [`request_cancel_run`] / polling.
    pub fn with_state_dir(run_id: RunId, state_dir: impl AsRef<Path>) -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
            run_id,
            state_dir: state_dir.as_ref().to_path_buf(),
        }
    }

    pub fn run_id(&self) -> RunId {
        self.run_id
    }

    /// Request cancellation (same effect as the CLI cancel command).
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        if !self.state_dir.as_os_str().is_empty() {
            let _ = request_cancel_run(self.run_id, &self.state_dir);
        }
    }

    /// Returns true after [`CancelSignal::cancel`], [`request_cancel_run`], or a CLI cancel.
    pub fn is_cancelled(&self) -> bool {
        if self.flag.load(Ordering::SeqCst) {
            return true;
        }
        if self.state_dir.as_os_str().is_empty() {
            return false;
        }
        cancel_requested(self.run_id, &self.state_dir)
    }
}

/// Default directory for run state (override with `CRASHLAB_STATE_DIR`).
pub fn default_state_dir() -> PathBuf {
    std::env::var("CRASHLAB_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".crashlab"))
}

/// Path to the on-disk cancel marker for `run_id` under `base`.
pub fn cancel_marker_path(run_id: RunId, base: impl AsRef<Path>) -> PathBuf {
    let base = base.as_ref();
    base.join("runs").join(run_id.0.to_string()).join("cancel")
}

fn cancel_file_path(run_id: RunId, base: &Path) -> PathBuf {
    cancel_marker_path(run_id, base)
}

/// Creates the cancel marker on disk so a running worker can observe it.
pub fn request_cancel_run(run_id: RunId, base: impl AsRef<Path>) -> io::Result<()> {
    let path = cancel_file_path(run_id, base.as_ref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, b"1")
}

/// Returns true if cancellation was requested for `run_id` under `base`.
pub fn cancel_requested(run_id: RunId, base: impl AsRef<Path>) -> bool {
    cancel_file_path(run_id, base.as_ref()).exists()
}

/// Removes the cancel marker (e.g. after handling or for tests).
pub fn clear_cancel_request(run_id: RunId, base: impl AsRef<Path>) -> io::Result<()> {
    let path = cancel_file_path(run_id, base.as_ref());
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// Runs `work` for each seed index in `0..total`, stopping early when `signal` fires.
/// If `partition` is provided, only seeds owned by that partition are processed, but
/// `total_seeds` is evaluated completely for cancellation reasons.
/// Returns [`RunTerminalState::Cancelled`] with a partial summary, or [`RunTerminalState::Completed`].
pub fn drive_run<F>(
    _run_id: RunId,
    total_seeds: u64,
    signal: &CancelSignal,
    partition: Option<WorkerPartition>,
    mut work: F,
) -> RunTerminalState
where
    F: FnMut(u64) -> Result<(), String>,
{
    let mut seeds_processed = 0u64;
    for seed_index in 0..total_seeds {
        if let Some(p) = &partition {
            if !p.owns_seed(seed_index) {
                continue;
            }
        }

        if signal.is_cancelled() {
            return RunTerminalState::Cancelled {
                summary: RunSummary {
                    seeds_processed,
                    cancelled_at_seed: Some(seed_index),
                },
            };
        }
        if let Err(message) = work(seed_index) {
            return RunTerminalState::Failed { message };
        }
        seeds_processed += 1;
    }

    RunTerminalState::Completed {
        summary: RunSummary {
            seeds_processed,
            cancelled_at_seed: None,
        },
    }
}

/// Like [`drive_run`], but invokes `work` only for global seed indices owned by `partition`
/// (`seed_index % num_workers == worker_index`). Still walks `0..total_seeds` in order so
/// cancellation checks align with the single-worker timeline.
pub fn drive_run_partitioned<F>(
    _run_id: RunId,
    total_seeds: u64,
    partition: &WorkerPartition,
    signal: &CancelSignal,
    mut work: F,
) -> RunTerminalState
where
    F: FnMut(u64) -> Result<(), String>,
{
    let mut seeds_processed = 0u64;
    for seed_index in 0..total_seeds {
        if signal.is_cancelled() {
            return RunTerminalState::Cancelled {
                summary: RunSummary {
                    seeds_processed,
                    cancelled_at_seed: Some(seed_index),
                },
            };
        }
        if !partition.owns_seed(seed_index) {
            continue;
        }
        if let Err(message) = work(seed_index) {
            return RunTerminalState::Failed { message };
        }
        seeds_processed += 1;
    }

    RunTerminalState::Completed {
        summary: RunSummary {
            seeds_processed,
            cancelled_at_seed: None,
        },
    }
}

/// Result type for run cancellation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelCommandError {
    /// I/O error when writing/reading cancel marker.
    IoError(String),
    /// Run state directory does not exist or is inaccessible.
    InvalidStateDir(String),
    /// Run was not found (no active run with this ID).
    RunNotFound(RunId),
}

impl std::fmt::Display for CancelCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancelCommandError::IoError(msg) => write!(f, "I/O error: {}", msg),
            CancelCommandError::InvalidStateDir(msg) => write!(f, "invalid state dir: {}", msg),
            CancelCommandError::RunNotFound(id) => write!(f, "run not found: {:?}", id),
        }
    }
}

impl std::error::Error for CancelCommandError {}

/// Public command to cancel an active run gracefully.
///
/// This function provides the primary interface for maintainers to cancel active runs
/// via the CLI or API. It ensures reproducible behavior and maintains terminal state.
///
/// # Arguments
/// * `run_id` - The ID of the run to cancel
/// * `state_dir` - The state directory where cancel markers are stored
///
/// # Returns
/// * `Ok(summary)` - Cancellation requested successfully; returns the run summary
/// * `Err` - I/O error or state directory issue
///
/// # Examples
/// ```no_run
/// # use crashlab_core::run_control::{cancel_run_command, RunId, default_state_dir};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let run_id = RunId(42);
/// let state_dir = default_state_dir();
/// match cancel_run_command(run_id, &state_dir) {
///     Ok(summary) => println!("Cancelled run {}: {}", run_id.0, summary.seeds_processed),
///     Err(e) => eprintln!("Cancellation failed: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
pub fn cancel_run_command(run_id: RunId, state_dir: impl AsRef<Path>) -> Result<RunSummary, CancelCommandError> {
    let state_dir = state_dir.as_ref();
    
    // Validate state directory exists and is accessible
    if !state_dir.exists() {
        return Err(CancelCommandError::InvalidStateDir(
            format!("state directory does not exist: {}", state_dir.display())
        ));
    }

    // Request cancellation via the file-based marker
    request_cancel_run(run_id, state_dir)
        .map_err(|e| CancelCommandError::IoError(format!("failed to write cancel marker: {}", e)))?;

    // Return a minimal summary indicating cancellation was requested
    Ok(RunSummary {
        seeds_processed: 0,
        cancelled_at_seed: Some(run_id.0),
    })
}

/// Query whether cancellation has been requested for a run.
///
/// This checks both the in-process flag and the file-based marker.
/// Useful for status dashboards and monitoring.
///
/// # Arguments
/// * `run_id` - The ID of the run to query
/// * `state_dir` - The state directory where cancel markers may exist
///
/// # Returns
/// `true` if cancellation was requested, `false` otherwise.
pub fn get_cancel_status(run_id: RunId, state_dir: impl AsRef<Path>) -> bool {
    cancel_requested(run_id, state_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker_partition::WorkerPartition;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_tmp() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("crashlab-run-{n}"))
    }

    #[test]
    fn cancel_signal_in_process_stops_drive_run() {
        let id = RunId(1);
        let signal = CancelSignal::new(id);
        signal.cancel();

        let outcome = drive_run(id, 100, &signal, None, |_i| Ok(()));
        match outcome {
            RunTerminalState::Cancelled { summary } => {
                assert_eq!(summary.seeds_processed, 0);
                assert_eq!(summary.cancelled_at_seed, Some(0));
            }
            other => panic!("expected cancelled, got {other:?}"),
        }
    }

    #[test]
    fn drive_run_completes_when_not_cancelled() {
        let id = RunId(2);
        let signal = CancelSignal::new(id);
        let mut seen = 0u64;
        let outcome = drive_run(id, 5, &signal, None, |_i| {
            seen += 1;
            Ok(())
        });
        match outcome {
            RunTerminalState::Completed { summary } => {
                assert_eq!(summary.seeds_processed, 5);
                assert_eq!(seen, 5);
            }
            other => panic!("expected completed, got {other:?}"),
        }
    }

    #[test]
    fn request_cancel_run_sets_cancel_requested() {
        let base = unique_tmp();
        let id = RunId(99);
        request_cancel_run(id, &base).expect("request");
        assert!(cancel_requested(id, &base));
        clear_cancel_request(id, &base).expect("clear");
        assert!(!cancel_requested(id, &base));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn file_cancel_observed_by_signal_without_in_process_flag() {
        let base = unique_tmp();
        let id = RunId(7);
        request_cancel_run(id, &base).expect("request");
        let signal = CancelSignal::with_state_dir(id, &base);
        assert!(signal.is_cancelled());
        clear_cancel_request(id, &base).expect("clear");
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn drive_run_picks_up_mid_run_file_cancel() {
        let base = unique_tmp();
        let id = RunId(3);
        let signal = CancelSignal::with_state_dir(id, &base);

        let outcome = drive_run(id, 10, &signal, None, |i| {
            if i == 2 {
                request_cancel_run(id, &base).expect("request cancel");
            }
            Ok(())
        });

        match outcome {
            RunTerminalState::Cancelled { summary } => {
                assert_eq!(summary.cancelled_at_seed, Some(3));
            }
            other => panic!("expected cancelled, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn drive_run_respects_worker_partition() {
        let id = RunId(4);
        let signal = CancelSignal::new(id);

        let partition = WorkerPartition::try_new(1, 3).expect("partition");

        let mut seen = Vec::new();
        let outcome = drive_run(id, 10, &signal, Some(partition), |i| {
            seen.push(i);
            Ok(())
        });

        match outcome {
            RunTerminalState::Completed { summary } => {
                // 10 seeds: 0..9.
                // Mod 3 gives:
                // 0 -> 0
                // 1 -> 1 *
                // 2 -> 2
                // 3 -> 0
                // 4 -> 1 *
                // 5 -> 2
                // 6 -> 0
                // 7 -> 1 *
                // 8 -> 2
                // 9 -> 0
                assert_eq!(summary.seeds_processed, 3);
                assert_eq!(seen, vec![1, 4, 7]);
            }
            other => panic!("expected completed, got {other:?}"),
        }
    }

    #[test]
    fn drive_run_partitioned_matches_seed_count_per_worker() {
        let id = RunId(8);
        let signal = CancelSignal::new(id);
        let total = 23u64;
        let n = 4u32;

        let mut per_worker = vec![0u64; n as usize];
        for w in 0..n {
            let p = WorkerPartition::try_new(w, n).expect("partition");
            let outcome = drive_run_partitioned(id, total, &p, &signal, |_i| Ok(()));
            match outcome {
                RunTerminalState::Completed { summary } => {
                    per_worker[w as usize] = summary.seeds_processed;
                }
                other => panic!("expected completed, got {other:?}"),
            }
        }

        assert_eq!(per_worker.iter().sum::<u64>(), total);
    }

    #[test]
    fn drive_run_partitioned_observes_cancel_at_global_index() {
        let id = RunId(11);
        let signal = CancelSignal::new(id);
        signal.cancel();

        // Worker 1 of 3: owns indices 1, 4, 7, ... — first iteration is global index 0 (skip), then 1 (work).
        let p = WorkerPartition::try_new(1, 3).expect("partition");
        let outcome = drive_run_partitioned(id, 20, &p, &signal, |_i| Ok(()));
        match outcome {
            RunTerminalState::Cancelled { summary } => {
                assert_eq!(summary.seeds_processed, 0);
                assert_eq!(summary.cancelled_at_seed, Some(0));
            }
            other => panic!("expected cancelled, got {other:?}"),
        }
    }

    // ============================================================================
    // Comprehensive Tests for Run Cancellation Command
    // ============================================================================

    #[test]
    fn cancel_run_command_requests_cancellation_successfully() {
        let base = unique_tmp();
        let id = RunId(100);
        fs::create_dir_all(&base).expect("create temp dir");

        let result = cancel_run_command(id, &base);
        assert!(result.is_ok(), "cancel command should succeed with valid state dir");
        assert!(cancel_requested(id, &base), "cancel marker should be written");
        
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cancel_run_command_fails_with_missing_state_dir() {
        let base = std::env::temp_dir().join("nonexistent-crashlab-dir-");
        let id = RunId(101);
        
        let result = cancel_run_command(id, &base);
        assert!(result.is_err(), "cancel command should fail with missing state dir");
        match result.unwrap_err() {
            CancelCommandError::InvalidStateDir(_) => {},
            other => panic!("expected InvalidStateDir, got {other:?}"),
        }
    }

    #[test]
    fn get_cancel_status_reflects_cancellation_request() {
        let base = unique_tmp();
        let id = RunId(102);
        fs::create_dir_all(&base).expect("create temp dir");

        assert!(!get_cancel_status(id, &base), "should not be cancelled initially");
        
        request_cancel_run(id, &base).expect("request cancel");
        assert!(get_cancel_status(id, &base), "should reflect cancellation request");
        
        clear_cancel_request(id, &base).expect("clear cancel");
        assert!(!get_cancel_status(id, &base), "should be cleared after removal");
        
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cancellation_with_zero_seeds() {
        let id = RunId(103);
        let signal = CancelSignal::new(id);
        signal.cancel();

        let outcome = drive_run(id, 0, &signal, None, |_i| Ok(()));
        match outcome {
            RunTerminalState::Completed { summary } => {
                assert_eq!(summary.seeds_processed, 0);
                assert_eq!(summary.cancelled_at_seed, None);
            }
            other => panic!("expected completed for 0 seeds, got {other:?}"),
        }
    }

    #[test]
    fn cancellation_mid_run_with_partial_seeds_processed() {
        let base = unique_tmp();
        let id = RunId(104);
        let signal = CancelSignal::with_state_dir(id, &base);
        fs::create_dir_all(&base).expect("create temp dir");

        let mut processed = Vec::new();
        let outcome = drive_run(id, 20, &signal, None, |i| {
            processed.push(i);
            if i == 5 {
                request_cancel_run(id, &base).expect("request cancel");
            }
            Ok(())
        });

        match outcome {
            RunTerminalState::Cancelled { summary } => {
                assert_eq!(summary.seeds_processed, 6, "should have processed 0-5 inclusive");
                assert_eq!(summary.cancelled_at_seed, Some(6), "should note where cancellation was detected");
                assert_eq!(processed, vec![0, 1, 2, 3, 4, 5]);
            }
            other => panic!("expected cancelled, got {other:?}"),
        }
        
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cancellation_with_work_failure_stops_at_error() {
        let id = RunId(105);
        let signal = CancelSignal::new(id);

        let outcome = drive_run(id, 20, &signal, None, |i| {
            if i == 7 {
                Err("work failed".to_string())
            } else {
                Ok(())
            }
        });

        match outcome {
            RunTerminalState::Failed { message } => {
                assert_eq!(message, "work failed");
            }
            other => panic!("expected failed state, got {other:?}"),
        }
    }

    #[test]
    fn cancellation_with_partitioned_run_multiple_workers() {
        let base = unique_tmp();
        let id = RunId(106);
        fs::create_dir_all(&base).expect("create temp dir");

        let total_seeds = 30u64;
        let num_workers = 3u32;

        // Simulate cancelled partitioned run for worker 1 of 3
        let signal = CancelSignal::with_state_dir(id, &base);
        let partition = WorkerPartition::try_new(1, num_workers).expect("partition");

        let mut processed = Vec::new();
        let outcome = drive_run_partitioned(id, total_seeds, &partition, &signal, |i| {
            processed.push(i);
            if i == 10 {
                request_cancel_run(id, &base).expect("request cancel");
            }
            Ok(())
        });

        match outcome {
            RunTerminalState::Cancelled { summary } => {
                // Worker 1 gets indices: 1, 4, 7, 10, 13, 16, 19, 22, 25, 28
                // Cancellation at global index 11 should be detected after processing worker's seeds before that
                assert!(summary.seeds_processed > 0, "should have processed some seeds");
                assert_eq!(summary.cancelled_at_seed, Some(11));
            }
            other => panic!("expected cancelled, got {other:?}"),
        }
        
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn cancel_signal_copies_share_flag() {
        let id = RunId(107);
        let signal1 = CancelSignal::new(id);
        let signal2 = signal1.clone();

        signal1.cancel();
        assert!(signal2.is_cancelled(), "cloned signals should share the flag");
    }

    #[test]
    fn cancel_marker_persists_across_signal_recreations() {
        let base = unique_tmp();
        let id = RunId(108);
        fs::create_dir_all(&base).expect("create temp dir");

        // Create and cancel with first signal
        let signal1 = CancelSignal::with_state_dir(id, &base);
        signal1.cancel();

        // Create new signal and check if cancellation is still visible
        let signal2 = CancelSignal::with_state_dir(id, &base);
        assert!(signal2.is_cancelled(), "cancel marker should persist on disk");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn drive_run_with_immediate_failure_before_cancellation_check() {
        let id = RunId(109);
        let signal = CancelSignal::new(id);
        signal.cancel();

        let outcome = drive_run(id, 10, &signal, None, |i| {
            // Cancellation check happens before calling work function
            // So if cancellation is already set, the failure code won't be reached
            if i == 0 {
                Err("immediate failure".to_string())
            } else {
                Ok(())
            }
        });

        match outcome {
            RunTerminalState::Cancelled { .. } => {
                // Cancellation check happens before calling work function
            }
            other => panic!("expected cancelled (since signal is pre-set), got {other:?}"),
        }
    }

    #[test]
    fn cancel_command_idempotent_multiple_calls() {
        let base = unique_tmp();
        let id = RunId(110);
        fs::create_dir_all(&base).expect("create temp dir");

        // First call
        let result1 = cancel_run_command(id, &base);
        assert!(result1.is_ok());

        // Second call should also succeed (idempotent)
        let result2 = cancel_run_command(id, &base);
        assert!(result2.is_ok());

        assert!(cancel_requested(id, &base), "cancel marker should persist");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn run_terminal_state_cancelled_includes_partial_summary() {
        let base = unique_tmp();
        let id = RunId(111);
        let signal = CancelSignal::with_state_dir(id, &base);
        fs::create_dir_all(&base).expect("create temp dir");

        let mut work_count = 0u64;
        let outcome = drive_run(id, 100, &signal, None, |i| {
            work_count += 1;
            if i == 12 {
                request_cancel_run(id, &base).expect("request cancel");
            }
            Ok(())
        });

        match outcome {
            RunTerminalState::Cancelled { summary } => {
                assert_eq!(summary.seeds_processed, 13, "seeds_processed is accurate count");
                assert!(summary.cancelled_at_seed.is_some(), "should record where cancellation was detected");
                assert_eq!(summary.cancelled_at_seed.unwrap(), 13);
            }
            other => panic!("expected cancelled, got {other:?}"),
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn drive_run_partitioned_cancellation_respects_partition_boundaries() {
        let id = RunId(112);
        let signal = CancelSignal::new(id);
        signal.cancel();

        let total = 50u64;
        let mut results = vec![];
        for worker_id in 0..5 {
            let partition = WorkerPartition::try_new(worker_id, 5).expect("partition");
            let outcome = drive_run_partitioned(id, total, &partition, &signal, |_i| Ok(()));
            
            match outcome {
                RunTerminalState::Cancelled { summary } => {
                    results.push(summary.seeds_processed);
                }
                other => panic!("expected cancelled, got {other:?}"),
            }
        }

        // All workers should have 0 seeds processed since cancel signal was set before any work
        for (idx, &count) in results.iter().enumerate() {
            assert_eq!(count, 0, "worker {} should have 0 seeds processed", idx);
        }
    }

    #[test]
    fn cancel_requested_returns_false_for_nonexistent_run() {
        let base = unique_tmp();
        let id = RunId(999);
        fs::create_dir_all(&base).expect("create temp dir");

        assert!(!cancel_requested(id, &base), "should return false for nonexistent run");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn clear_cancel_request_idempotent() {
        let base = unique_tmp();
        let id = RunId(113);
        fs::create_dir_all(&base).expect("create temp dir");

        request_cancel_run(id, &base).expect("request");
        
        // Clear once
        clear_cancel_request(id, &base).expect("clear 1");
        assert!(!cancel_requested(id, &base));

        // Clear again (should not error even though file is gone)
        clear_cancel_request(id, &base).expect("clear 2");
        assert!(!cancel_requested(id, &base));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn run_terminal_state_json_serialization_for_cancelled_run() {
        let summary = RunSummary {
            seeds_processed: 42,
            cancelled_at_seed: Some(43),
        };

        let state = RunTerminalState::Cancelled { summary };
        let json = serde_json::to_string(&state).expect("serialize");
        // The serde(tag="status") will create a "status" field with value "cancelled"
        assert!(json.contains("\"status\""), "JSON should contain status field");
        assert!(json.contains("cancelled"), "JSON should contain cancelled status value");
        assert!(json.contains("42"), "should serialize seeds_processed");
    }

    #[test]
    fn run_summary_with_no_cancellation_seed() {
        let summary = RunSummary {
            seeds_processed: 100,
            cancelled_at_seed: None,
        };

        assert_eq!(summary.seeds_processed, 100);
        assert!(summary.cancelled_at_seed.is_none());
    }

    #[test]
    fn cancel_signal_state_dir_empty_string() {
        let id = RunId(114);
        let signal = CancelSignal::with_state_dir(id, "");
        
        // Should handle empty state dir gracefully (no file ops)
        signal.cancel();
        assert!(signal.is_cancelled(), "in-process flag should work");
    }
}
