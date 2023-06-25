# packs
![Logo](logo.png)

[![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)](https://github.com/alexevanczuk/packs/actions)
[![Crates.io](https://img.shields.io/crates/v/pks.svg)](https://crates.io/crates/pks)
[![Security Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)](https://github.com/alexevanczuk/packs/actions?query=workflow%3A%22Security+audit%22++)

A 100% Rust implementation of [packwerk](https://github.com/Shopify/packwerk), a gradual modularization platform for ruby.

# Goals:
## To be a drop-in replacement for `packwerk` on most projects
- Currently can serve as a drop-in replacement on Gusto's extra-large Rails monolith
- This is a work in progress! Please see [Verification](#verification) for instructions on how to verify the output of `packs` is the same as `packwerk`.

## To be 20x faster than `packwerk` on most projects
- Currently ~10x as fast as the ruby implementation. See [Benchmarks](#benchmarks).
- Your milemage may vary!
- Other performance improvements are coming soon!

# Usage and Documentation
Once installed and added to your `$PATH`, just call `packs` to see the CLI help message and documentation.

```
Welcome! Please see https://github.com/alexevanczuk/packs for more information!

Usage: packs [OPTIONS] <COMMAND>

Commands:
  greet                Just saying hi
  check                Look for violations in the codebase
  update               Update package_todo.yml files with the current violations
  validate             Look for validation errors in the codebase
  generate_cache       Generate a cache to be used by the ruby implementation of packwerk
  list_packs           List packs based on configuration in packwerk.yml
  delete_cache         `rm -rf` on your cache directory, usually `tmp/cache/packwerk`
  list_included_files  List analyzed files based on configuration in packwerk.yml
  help                 Print this message or the help of the given subcommand(s)

Options:
      --project-root <PROJECT_ROOT>  Path for the root of the project [default: .]
  -h, --help                         Print help
  -V, --version                      Print version
```

# Installation
See [INSTALLATION.md](https://github.com/alexevanczuk/packs/blob/main/INSTALLATION.md)

# Verification
As `packs` is still a work-in-progress, it's possible it will not produce the same results as the ruby implementation (see [Not Yet Supported](#not-yet-supported)). If so, please file an issue – I'd love to try to support your use case!

Instructions:
- Follow directions above to install `packs`
- Run `packs delete_cache`
- Run `packs update`
- Confirm the output of `git diff` is empty
- Please file an issue if it's not!

# New to Rust?
Me too! This is my first Rust project, so I'd love to have feedback, advice, and contributions!

Rust is a low-level language with high-level abstractions, a rich type system, with a focus on memory safety through innovative compile type checks on memory usage.

If you're new to Rust, don't be intimidated! [https://www.rust-lang.org](https://www.rust-lang.org/learn) has tons of great learning resources.

If you'd like to contribute but don't know where to start, please reach out! I'd love to help you get started.

# Not yet supported
- privacy checker or other checkers
- custom associations
- custom inflections
- custom load paths
- zeitwerk default namespaces
- extensible plugin system
- stale violation detection
- bin/packwerk validate (e.g. cycle detection)

# Benchmarks
## Cold Cache, without Spring
- `packs check`: `rm -rf tmp/cache/packwerk && DISABLE_SPRING=1 time ../pks/target/release/packs check`
- `packwerk check`: `rm -rf tmp/cache/packwerk && DISABLE_SPRING=1 time bin/packwerk check`

| Run         | `packs check` | `packwerk check` |
|-------------|---------------|------------------|
| 1           | 8.9s          | 107.83s          |
| 2           | 7.31s         | 85.24s           |
| 3           | 7.55s         | 126.52s          |
| 4           | 6.85s         | 80.47s           |
| 5           | 8.45s         | 99.90s           |
| **Average** | 7.812s        | 99.99s           |

## Hot Cache, without Spring
- `packs check`: `DISABLE_SPRING=1 time ../pks/target/release/packs check`
- `packwerk check`: `DISABLE_SPRING=1 time bin/packwerk check`

| Run         | `packs check` | `packwerk check` |
|-------------|---------------|------------------|
| 1           | 3.86s         | 39.33s           |
| 2           | 3.69s         | 34.02s           |
| 3           | 3.6s          | 41.68s           |
| 4           | 3.52s         | 35.26s           |
| 5           | 3.32s         | 37.14s           |
| **Average** | 3.598         | 37.29            |
