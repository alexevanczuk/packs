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
2. `cargo clippy -- -D warnings` - Run linter (must pass with zero warnings; CI runs with `-D warnings` so any warning is a build failure)
3. `cargo test` - Run all tests

**All three must pass before every commit.** CI will reject pushes that fail any of these.

## Versioning

Always bump the version in `Cargo.toml` when making changes. This triggers a new release via CI. Use patch version bumps (e.g. 0.2.31 → 0.2.32) for most changes.
