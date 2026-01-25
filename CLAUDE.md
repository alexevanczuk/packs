# pks

Rust CLI tool for working with packs (modular code organization).

## Commands

- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo clippy` - Run linter (fix issues before committing)
- `cargo fmt` - Format code

## Before committing

Run these commands to ensure CI will pass:
1. `cargo fmt` - Format code (CI checks formatting)
2. `cargo clippy --all-targets --all-features` - Run linter (must pass with no warnings)
3. `cargo test` - Run all tests
