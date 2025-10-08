**LLM Prompt:**

You are an expert Rust developer and testing specialist. Your task is to create a comprehensive unit and integration testing strategy and implementation for the `LogLens` monorepo project.

**Project Context:**

`LogLens` is a Rust-based application for log analysis. The project is structured as a monorepo with several crates, including `loglens-core` (the main logic), `loglens-cli`, `loglens-web`, and `loglens-mcp`. The core functionality, which must be the primary focus of testing, resides in `loglens-core/src/`.

**Core Requirements:**

1.  **Testing Framework:** Utilize Rust's standard, built-in testing framework. All tests should be runnable via the `cargo test` command.

2.  **Test Separation:** All test code MUST be strictly separated from the application source code. Follow Rust best practices by creating a `tests` directory within each crate that requires testing (e.g., `loglens-core/tests/`, `loglens-web/tests/`). Do not use inline `#[cfg(test)]` modules inside the application files.

3.  **Comprehensive Feature Coverage:** The tests must cover all existing features of the program to ensure correct behavior and prevent regressions. Your testing should prioritize the `loglens-core` crate and its modules, including:
    *   `parser.rs`: Test various log formats and edge cases.
    *   `analyzer.rs`: Verify the logic of log analysis.
    *   `classification.rs`: Ensure accurate classification of log entries.
    *   `filter.rs`: Test the filtering logic.
    *   `input.rs`: Check different input handling mechanisms.
    *   `slimmer.rs`: Verify the log slimming functionality.
    *   Also, provide foundational tests for other crates like `loglens-cli` and `loglens-web` to ensure their core components function as expected.

4.  **Continuous Integration:** The testing setup must be designed to be integrated into a CI/CD pipeline, where `cargo test` is run automatically upon every code change or compilation request to validate the application's integrity.

5.  **Automation for Future Tests:** To ensure the testing suite grows with the application, you must automate the process of adding new tests. This will be achieved by:
    *   Establishing a clear, modular, and scalable testing structure that is easy for developers to extend.
    *   Creating a shell script named `scripts/new_test.sh`. This script will take a crate name and a test module name as arguments (e.g., `./scripts/new_test.sh core new_feature`) and automatically generate a boilerplate test file in the correct directory (e.g., `loglens-core/tests/new_feature.rs`) with a basic test function skeleton.

**Deliverables:**

1.  **Create `tests` Directories:** For each relevant crate (`loglens-core`, `loglens-cli`, `loglens-web`, etc.), create a `tests` directory.
2.  **Implement Test Files:** Populate the `tests` directories with Rust files (`*.rs`) containing thorough unit and integration tests for all existing features.
3.  **Create Automation Script:** Create the `scripts/new_test.sh` script as described above.
4.  **Write Documentation:** Create a new markdown file named `docs/TESTING_GUIDE.md`. This document should clearly explain:
    *   The overall testing philosophy.
    *   How to run the entire test suite.
    *   A step-by-step guide on how to use the `scripts/new_test.sh` script and add new tests for any new feature that is introduced.
