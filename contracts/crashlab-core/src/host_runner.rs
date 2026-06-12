//! Host-based contract runner using soroban-sdk testutils.
//!
//! This module provides [`HostContractRunner`], an implementation of [`ContractRunner`]
//! that uses soroban-sdk's test utilities to execute contracts in a local test environment.
//!
//! # Purpose
//! The host runner enables local contract testing without requiring a full Soroban
//! RPC setup. It uses the soroban-sdk testutils to create an in-memory environment
//! for contract execution.
//!
//! # Usage
//! ```rust,no_run
//! use soroban_sdk::{Env, Address};
//! use soroban_sdk::testutils::Address as _;
//! use crashlab_core::{CaseSeed, HostContractRunner};
//!
//! let mut runner = HostContractRunner::new();
//! let seed = CaseSeed { id: 1, payload: vec![1, 2, 3] };
//! let signature = runner.run_seed(&seed)?;
//! ```

#![cfg(feature = "host-runner")]

use crate::{CaseSeed, CrashSignature};
use crate::runner::{RunnerError, ContractRunner};
use soroban_sdk::Env;

/// A contract runner that uses soroban-sdk testutils for local execution.
///
/// `HostContractRunner` creates a new test environment for each seed execution,
/// allowing for isolated testing of contract behavior.
#[derive(Debug)]
pub struct HostContractRunner {
    /// Whether to mock all authorizations.
    mock_auths: bool,
}

impl HostContractRunner {
    /// Creates a new `HostContractRunner` with default settings.
    ///
    /// By default, all authorizations are mocked to simplify testing.
    pub fn new() -> Self {
        Self { mock_auths: true }
    }

    /// Creates a new `HostContractRunner` with the specified auth mocking setting.
    ///
    /// # Arguments
    /// * `mock_auths` - If true, all authorizations are mocked using `env.mock_all_auths()`.
    pub fn with_mock_auths(mock_auths: bool) -> Self {
        Self { mock_auths }
    }

    /// Executes a seed in a fresh test environment.
    ///
    /// This method creates a new `Env` for each execution, ensuring isolation
    /// between test runs. The seed payload is used to simulate contract execution.
    fn execute_in_env(&self, seed: &CaseSeed) -> Result<CrashSignature, RunnerError> {
        // Create a fresh test environment for each execution
        let env = Env::default();

        // Mock authorizations if configured
        if self.mock_auths {
            env.mock_all_auths();
        }

        // Execute the seed payload
        // For now, we simulate execution by checking the payload
        // In a real implementation, this would invoke contract methods
        self.simulate_execution(&env, seed)
    }

    /// Simulates contract execution based on the seed payload.
    ///
    /// This is a placeholder implementation that demonstrates the pattern.
    /// A real implementation would:
    /// 1. Register the contract under test
    /// 2. Parse the seed payload to determine which function to call
    /// 3. Invoke the contract function with appropriate arguments
    /// 4. Capture the result or panic
    fn simulate_execution(&self, _env: &Env, seed: &CaseSeed) -> Result<CrashSignature, RunnerError> {
        // Placeholder: classify the seed based on its payload
        // In a real implementation, this would execute actual contract code
        if seed.payload.is_empty() {
            return Ok(CrashSignature {
                category: "empty-input".to_string(),
                digest: seed.id,
                signature_hash: crate::compute_signature_hash("empty-input", &seed.payload),
            });
        }

        // Check for specific patterns that might cause panics
        if seed.payload.len() == 1 && seed.payload[0] == 0xFF {
            return Ok(CrashSignature {
                category: "panic-pattern".to_string(),
                digest: seed.id,
                signature_hash: crate::compute_signature_hash("panic-pattern", &seed.payload),
            });
        }

        // Default successful execution
        Ok(CrashSignature {
            category: "success".to_string(),
            digest: seed.id,
            signature_hash: crate::compute_signature_hash("success", &seed.payload),
        })
    }
}

impl Default for HostContractRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractRunner for HostContractRunner {
    fn run_seed(&mut self, seed: &CaseSeed) -> Result<CrashSignature, RunnerError> {
        self.execute_in_env(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_runner_creates_success_signature() {
        let mut runner = HostContractRunner::new();
        let seed = CaseSeed {
            id: 1,
            payload: vec![1, 2, 3],
        };

        let sig = runner.run_seed(&seed).unwrap();

        assert_eq!(sig.category, "success");
        assert_eq!(sig.digest, 1);
    }

    #[test]
    fn host_runner_detects_empty_input() {
        let mut runner = HostContractRunner::new();
        let seed = CaseSeed {
            id: 2,
            payload: vec![],
        };

        let sig = runner.run_seed(&seed).unwrap();

        assert_eq!(sig.category, "empty-input");
        assert_eq!(sig.digest, 2);
    }

    #[test]
    fn host_runner_detects_panic_pattern() {
        let mut runner = HostContractRunner::new();
        let seed = CaseSeed {
            id: 3,
            payload: vec![0xFF],
        };

        let sig = runner.run_seed(&seed).unwrap();

        assert_eq!(sig.category, "panic-pattern");
        assert_eq!(sig.digest, 3);
    }

    #[test]
    fn host_runner_with_mock_auths_disabled() {
        let runner = HostContractRunner::with_mock_auths(false);
        assert!(!runner.mock_auths);
    }

    #[test]
    fn host_runner_default_has_mock_auths_enabled() {
        let runner = HostContractRunner::default();
        assert!(runner.mock_auths);
    }

    #[test]
    fn signature_hash_is_deterministic() {
        let mut runner = HostContractRunner::new();
        let seed = CaseSeed {
            id: 1,
            payload: vec![1, 2, 3],
        };

        let sig1 = runner.run_seed(&seed).unwrap();
        let sig2 = runner.run_seed(&seed).unwrap();

        assert_eq!(sig1.signature_hash, sig2.signature_hash);
    }

    #[test]
    fn different_payloads_produce_different_hashes() {
        let mut runner = HostContractRunner::new();
        let seed1 = CaseSeed {
            id: 1,
            payload: vec![1, 2, 3],
        };
        let seed2 = CaseSeed {
            id: 2,
            payload: vec![3, 2, 1],
        };

        let sig1 = runner.run_seed(&seed1).unwrap();
        let sig2 = runner.run_seed(&seed2).unwrap();

        assert_ne!(sig1.signature_hash, sig2.signature_hash);
    }
}
