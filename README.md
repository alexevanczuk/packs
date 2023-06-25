# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

![Logo](logo.png)

WIP Rust implementation of [packwerk](https://github.com/Shopify/packwerk), a gradual modularization platform for ruby

# About
- It's entirely built in Rust, so it's pretty fast! In Gusto's monolith, it's about 10x faster ([Benchmarks](#benchmarks)) than the ruby implementation. Your mileage may vary! Other performance optimizations could potentially get to 20x faster.
- The goal is for this to be able to be a drop-in replacement for `packwerk`.
- Currently, `packs` implements `check` and `update`.

# Installation
## Option 1:
- Install Rust: https://www.rust-lang.org/tools/install
  - TLDR: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, and you're done!
- `cargo install packs` (it's like `gem install`)

## Option 2:
(Mac only – for other platforms, please create an issue/PR or try option 1.)

- Go to https://github.com/alexevanczuk/packs/releases
- Download the `packs` asset and run `chmod +x path/to/packs`. This makes the asset executable on your machine.
- Open the containing directory, right click on the binary, click open, and then accept the warning message that says its from an unknown developer (it's me!)
- Execute `path/to/packs` to see the CLI help message.

You can add `path/to/packs` to your `PATH` so it's available in every terminal session.

## Option 3 (coming soon):
I'm looking into installing via `brew` or as a native ruby gem extension. More coming soon!

# Usage
Once installed and added to your `$PATH`, just call `packs` to see the CLI help message and documentation.

# Verification
As `packs` is still a work-in-progress, it's possible it will not produce the same results as the ruby implementation (see [Not Yet Supported](#not-yet-supported)). If so, please file an issue – I'd love to try to support your use case!

Instructions:
- Follow directions above to install `packs`
- Run `packs delete_cache`
- Run `packs update`
- Confirm the output of `git diff` is empty
- Please file an issue if it's not! 

# Distribution Improvements
In the future, I hope to:
- Somehow sign the binary so it does not get a warning message
- Make it executable before download
- Add directions to download via some other tool, or ship as a native ruby gem extension.

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
