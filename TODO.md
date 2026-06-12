# TODO.md

- [x] Inspect relevant existing modules (`lib.rs`, `suite_runner.rs`, `reproducer.rs`, `run_control.rs`, existing tests).
- [x] Implement `contracts/crashlab-core/src/runner.rs` with `ContractRunner` trait and `RunnerError` error taxonomy, plus an example `MockRunner` and unit tests.
- [x] Export the new module from `contracts/crashlab-core/src/lib.rs` via `pub mod runner;`.
- [x] Bridge simulation module to ContractRunner (via runner-like closure injection) and add mock/bridge test.
