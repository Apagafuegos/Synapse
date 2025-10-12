# Synapse Development Guide

## Build Commands
- `cargo build` - Build the project
- `cargo run -- <args>` - Run the CLI with arguments
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo check` - Type check without building
- `cargo clippy` - Lint the codebase
- `cargo fmt` - Format code

## Code Style Guidelines

### Imports
- Use `use` statements at the top of files
- Group imports: std crates, external crates, local modules
- Prefer explicit imports over glob imports

### Formatting
- Follow Rust standard formatting (rustfmt)
- Use 4-space indentation
- Line length: 100 characters max

### Types
- Use strong typing with explicit types where clarity is needed
- Prefer `String` over `&str` for owned data
- Use `Result<T, E>` for error handling
- Define custom error types with `thiserror`

### Naming Conventions
- Functions and variables: `snake_case`
- Types and structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling
- Use `Result` types for fallible operations
- Provide context with `context()` from `anyhow`
- Handle errors gracefully at boundaries
- Use `?` operator for propagation

### Project Structure
- Core logic in `src/` with modular organization
- AI providers in `src/ai_provider/`
- Each module should have clear responsibility
- Integration tests in `tests/` directory

### Testing
- Unit tests in module files using `#[cfg(test)]`
- Integration tests for end-to-end functionality
- Mock external dependencies in tests
- Use `assert_eq!` and `assert!` macros