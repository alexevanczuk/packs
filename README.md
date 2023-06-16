# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

WIP Rust implementation of [packs](https://github.com/rubyatscale/use_packs) and [packwerk](https://github.com/Shopify/packwerk) for ruby

# About
- It's entirely built in Rust, so it's pretty fast! In Gusto's monolith, it's about 10x faster ([Benchmarks](#benchmarks)) than the ruby implementation. Your mileage may vary! Other performance optimizations could potentially get to 20x faster.
- The goal is for this to be able to be a drop-in replacement for `packwerk`.
- Currently, `packs` implements `check`, with `update` coming soon.

# Usage
Once installed and added to your `$PATH`, just call `packs` to see the CLI help message.

# Verification
As `packs` is still a work-in-progress, it's possible it will not produce the same results as the ruby implementation (see below). If not, please file an issue!

To verify:
1. Run `rm -rf tmp/cache/packwerk` to delete the existing cache.
2. Run `packs generate_cache` (see directions below to get binary)
3. Run `bin/packwerk update` to see how violations change using `packs`.

Separately, you can run:
`packs check` and compare the output to `bin/packwerk check`

# Downloading the Binary
Deployment ergonomics are still a WIP.

If you want to try it out:
- Go to https://github.com/alexevanczuk/packs/releases
- Download the `packs` asset and run `chmod +x path/to/packs`
- Open the containing directory, right click on the binary, click open, and then accept the warning message that says its from an unknown developer (it's me!)
- Execute `path/to/packs` to see the CLI help message.

You can add `path/to/packs` to your `PATH` so it's available in every terminal session.

# Deployment Improvements
In the future, I hope to:
- Somehow sign the binary so it does not get a warning message
- Make it executable before download
- Add directions to download via some other tool, or ship as a native ruby gem extension.

# New to Rust?
Me too! This is my first Rust project, so I'd love to have feedback, advice, and contributions!
If you're new to Rust, don't be intimidated! [https://www.rust-lang.org](https://www.rust-lang.org/learn) has tons of great learning resources.

# Not yet supported:
- privacy checker or other checkers
- custom associations
- custom inflections
- custom load paths
- zeitwerk default namespaces
- extensible plugin system

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

# Profiling
I've been using https://github.com/flamegraph-rs/flamegraph to generate flamegraphs to improve performance.

Specifically, this command which merges similar code paths to see where most of the time is spent:
```
sudo cargo flamegraph --profile=release --reverse --min-width=0.5 -- --project-root=../your_app check
```
For more, see: https://nnethercote.github.io/perf-book/profiling.html

# Local Development
## Running the CLI in release mode against a target app
```
RUST_LOG=debug time cargo run --profile=release -- --project-root=../your_app check
```
