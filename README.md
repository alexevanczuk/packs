# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

WIP: Rust implementation of [packs](https://github.com/rubyatscale/use_packs) and [packwerk](https://github.com/Shopify/packwerk) for ruby

# Features
- Currently all `packs` can do is generate a cache to be used by the ruby implementation.

# Usage
Deployment ergonomics are still a WIP.

If you want to try it out to see how well it works on your repo, you may want to:
1. Verify it produces the same results as the ruby implementation (see below). If not, please file an issue!
2. Download the binary.
3. Add to your `bin/packwerk`

# Verification
1. Run `rm -rf tmp/cache/packwerk` to delete the existing cache.
2. Run `packs generate_cache` (see directions below to get binary) 
3. Run `bin/packwerk update` to see how violations change using `packs`.

# Downloading the Binary
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
