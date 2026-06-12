//! Integration runner trait for executing seeds against a contract host.
//!
//! # Purpose
//! CrashLab's core logic works with [`CaseSeed`] and produces/compares [`CrashSignature`].
//! Different integrators (e.g. local sandbox, RPC-backed test harnesses, CI runners)
//! need a common way to execute a seed and obtain the resulting signature.
//!
//! This module defines the [`ContractRunner`] trait and associated error types.
//!
//! Implementors are responsible for:
//! - Translating a [`CaseSeed`] payload into a call against a contract host.
//! - Executing the call.
//! - Returning the [`CrashSignature`] observed by the runner.
//!
//! The core crate also needs a structured error taxonomy so callers can
//! distinguish transient failures (eligible for retries) from permanent
//! simulation/runtime errors.

use crate::{CaseSeed, CrashSignature};

/// Error returned when a [`ContractRunner`] fails to produce a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerError {
    /// An error that is expected to resolve by retrying.
    ///
    /// Examples:
    /// - RPC timeouts
    /// - temporary resource exhaustion
    /// - short-lived host/service interruptions
    Transient {
        /// Human-readable explanation.
        message: String,
    },

    /// A failure of the contract under test that should be treated as a
    /// stable outcome.
    ///
    /// Examples:
    /// - Revert/panic with deterministic error
    /// - Contract execution that violates invariants
    Permanent {
        /// Human-readable explanation.
        message: String,
    },

    /// The runner could not execute the seed because configuration is
    /// invalid or the host cannot be reached.
    ///
    /// This is treated as permanent from the caller's perspective.
    Misconfigured {
        /// Human-readable explanation.
        message: String,
    },
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerError::Transient { message } => write!(f, "transient runner error: {message}"),
            RunnerError::Permanent { message } => write!(f, "permanent runner error: {message}"),
            RunnerError::Misconfigured { message } => write!(f, "misconfigured runner: {message}"),
        }
    }
}

impl std::error::Error for RunnerError {}

/// Integration contract for Soroban integrators.
///
/// A `ContractRunner` executes a [`CaseSeed`] against some contract host
/// (e.g. local test environment, Soroban RPC cluster, or a custom harness)
/// and returns the resulting [`CrashSignature`].
///
/// # Trait contract
/// Implementations must obey the following expectations:
/// - Returned signatures must be comparable with those produced by other
///   parts of the core crate (i.e. stable across equivalent executions).
/// - `RunnerError::Transient` should be used for errors that retrying may
///   resolve.
/// - `RunnerError::Permanent` / `RunnerError::Misconfigured` should be used
///   for non-retryable failures.
pub trait ContractRunner {
    /// Executes `seed` and returns the observed [`CrashSignature`].
    fn run_seed(&mut self, seed: &CaseSeed) -> Result<CrashSignature, RunnerError>;
}

/// Simple test runner used in unit tests.
#[derive(Debug, Default)]
pub struct MockRunner {
    /// If set, this error is returned for all seeds.
    pub forced_error: Option<RunnerError>,
}

impl ContractRunner for MockRunner {
    fn run_seed(&mut self, seed: &CaseSeed) -> Result<CrashSignature, RunnerError> {
        if let Some(err) = &self.forced_error {
            return Err(err.clone());
        }

        // Deterministic signature for tests.
        Ok(CrashSignature {
            category: "runtime-failure".to_string(),
            digest: seed.id,
            signature_hash: seed.payload.iter().fold(0u64, |acc, b| acc.wrapping_add(*b as u64)),
        })
    }
}

/// Error type for runner creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerCreationError {
    /// Invalid runner type specified in `CRASHLAB_RUNNER` environment variable.
    InvalidRunnerType {
        /// The invalid runner type value.
        runner_type: String,
    },
    /// The requested runner type requires a feature that is not enabled.
    FeatureNotEnabled {
        /// Name of the required feature.
        feature: String,
        /// The requested runner type.
        runner_type: String,
    },
}

impl std::fmt::Display for RunnerCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerCreationError::InvalidRunnerType { runner_type } => {
                write!(f, "invalid runner type: '{}'. Supported types: mock, host", runner_type)
            }
            RunnerCreationError::FeatureNotEnabled { feature, runner_type } => {
                write!(f, "runner type '{}' requires the '{}' feature to be enabled", runner_type, feature)
            }
        }
    }
}

impl std::error::Error for RunnerCreationError {}

/// Creates a [`ContractRunner`] based on the `CRASHLAB_RUNNER` environment variable.
///
/// # Environment Variable
/// The `CRASHLAB_RUNNER` environment variable controls which runner implementation is used:
/// - `"mock"` (or unset): Creates a [`MockRunner`] for testing purposes.
/// - `"host"`: Creates a [`HostContractRunner`] for Soroban SDK testutils-based execution.
///   This requires the `host-runner` feature to be enabled.
///
/// # Errors
/// Returns [`RunnerCreationError`] if:
/// - The `CRASHLAB_RUNNER` value is not a recognized runner type.
/// - A requested runner type requires an unmet feature.
///
/// # Examples
/// ```rust,no_run
/// # use crashlab_core::runner::create_runner;
/// // With CRASHLAB_RUNNER=mock (or unset):
/// let mut runner = create_runner().expect("failed to create runner");
///
/// // With CRASHLAB_RUNNER=host (requires host-runner feature):
/// # std::env::set_var("CRASHLAB_RUNNER", "host");
/// # #[cfg(feature = "host-runner")]
/// # let mut runner = create_runner().expect("failed to create runner");
/// ```
pub fn create_runner() -> Result<Box<dyn ContractRunner>, RunnerCreationError> {
    let runner_type = std::env::var("CRASHLAB_RUNNER")
        .unwrap_or_else(|_| "mock".to_string());

    match runner_type.as_str() {
        "mock" => Ok(Box::new(MockRunner::default())),
        "host" => {
            #[cfg(feature = "host-runner")]
            {
                Ok(Box::new(crate::host_runner::HostContractRunner::new()))
            }
            #[cfg(not(feature = "host-runner"))]
            {
                Err(RunnerCreationError::FeatureNotEnabled {
                    feature: "host-runner".to_string(),
                    runner_type: "host".to_string(),
                })
            }
        }
        _ => Err(RunnerCreationError::InvalidRunnerType {
            runner_type,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CaseSeed;

    #[test]
    fn mock_runner_returns_signature_for_seed() {
        let seed = CaseSeed { id: 1, payload: vec![1, 2, 3] };
        let mut runner = MockRunner::default();

        let sig = runner.run_seed(&seed).unwrap();

        assert_eq!(sig.category, "runtime-failure");
        assert_eq!(sig.digest, 1);
        assert_eq!(
            sig.signature_hash,
            1u64.wrapping_add(2).wrapping_add(3)
        );
    }

    #[test]
    fn mock_runner_forced_transient_error() {
        let seed = CaseSeed { id: 1, payload: vec![1] };
        let mut runner = MockRunner {
            forced_error: Some(RunnerError::Transient {
                message: "rpc timeout".to_string(),
            }),
        };

        let err = runner.run_seed(&seed).unwrap_err();
        assert_eq!(
            err,
            RunnerError::Transient {
                message: "rpc timeout".to_string(),
            }
        );
    }

    #[test]
    fn mock_runner_forced_permanent_error() {
        let seed = CaseSeed { id: 1, payload: vec![1] };
        let mut runner = MockRunner {
            forced_error: Some(RunnerError::Permanent {
                message: "contract panic".to_string(),
            }),
        };

        let err = runner.run_seed(&seed).unwrap_err();
        assert_eq!(
            err,
            RunnerError::Permanent {
                message: "contract panic".to_string(),
            }
        );
    }

    #[test]
    fn mock_runner_forced_misconfigured_error() {
        let seed = CaseSeed { id: 1, payload: vec![1] };
        let mut runner = MockRunner {
            forced_error: Some(RunnerError::Misconfigured {
                message: "missing contract id".to_string(),
            }),
        };

        let err = runner.run_seed(&seed).unwrap_err();
        assert_eq!(
            err,
            RunnerError::Misconfigured {
                message: "missing contract id".to_string(),
            }
        );
    }

    #[test]
    fn create_runner_defaults_to_mock_when_env_unset() {
        // Ensure CRASHLAB_RUNNER is unset
        std::env::remove_var("CRASHLAB_RUNNER");

        let runner = create_runner();
        assert!(runner.is_ok(), "create_runner should succeed with no env var");
    }

    #[test]
    fn create_runner_creates_mock_when_env_is_mock() {
        std::env::set_var("CRASHLAB_RUNNER", "mock");

        let runner = create_runner();
        assert!(runner.is_ok(), "create_runner should succeed with CRASHLAB_RUNNER=mock");
    }

    #[test]
    fn create_runner_rejects_invalid_runner_type() {
        std::env::set_var("CRASHLAB_RUNNER", "invalid");

        let result = create_runner();
        assert!(result.is_err(), "create_runner should fail with invalid runner type");
        
        if let Err(err) = result {
            assert_eq!(
                err,
                RunnerCreationError::InvalidRunnerType {
                    runner_type: "invalid".to_string(),
                }
            );
        }
    }

    #[test]
    fn create_runner_rejects_host_without_feature() {
        std::env::set_var("CRASHLAB_RUNNER", "host");

        #[cfg(not(feature = "host-runner"))]
        {
            let result = create_runner();
            assert!(result.is_err(), "create_runner should fail without host-runner feature");
            
            if let Err(err) = result {
                assert_eq!(
                    err,
                    RunnerCreationError::FeatureNotEnabled {
                        feature: "host-runner".to_string(),
                        runner_type: "host".to_string(),
                    }
                );
            }
        }

        #[cfg(feature = "host-runner")]
        {
            let runner = create_runner();
            assert!(runner.is_ok(), "create_runner should succeed with host-runner feature and CRASHLAB_RUNNER=host");
        }
    }

    #[test]
    fn runner_creation_error_display_invalid_type() {
        let err = RunnerCreationError::InvalidRunnerType {
            runner_type: "xyz".to_string(),
        };

        assert_eq!(
            err.to_string(),
            "invalid runner type: 'xyz'. Supported types: mock, host"
        );
    }

    #[test]
    fn runner_creation_error_display_feature_not_enabled() {
        let err = RunnerCreationError::FeatureNotEnabled {
            feature: "host-runner".to_string(),
            runner_type: "host".to_string(),
        };

        assert_eq!(
            err.to_string(),
            "runner type 'host' requires the 'host-runner' feature to be enabled"
        );
    }

    #[test]
    fn mock_runner_from_factory_executes_seed() {
        std::env::set_var("CRASHLAB_RUNNER", "mock");

        let mut runner = create_runner().expect("failed to create runner");
        let seed = CaseSeed { id: 42, payload: vec![1, 2, 3] };

        let sig = runner.run_seed(&seed).expect("seed execution failed");
        assert_eq!(sig.digest, 42);
        assert_eq!(sig.category, "runtime-failure");
    }
}

