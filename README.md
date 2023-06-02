# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

WIP: Rust implementation of packs for ruby

# Initial Milestone

- [ ] `packs generate_cache`, which can be used to update `tmp/cache/packwerk` for faster `packwerk` output. It should produce the exact same `json` that `packwerk` produces today. 
Current Progress:
  - Current progress is detected using `scripts/packwerk_parity_checker.rb`
  - Currently, `packs` detects roughly 98% of references in Gusto's monolith
Remaining Challenges include:
  - [ ] Parsing ERB
  - [ ] Parsing Rails associations and rewriting them as constant references using a pluralizer. Initially, non-standard inflections will likely not be supported (although I may support it through hard-coded map in `packwerk.yml`)
  - [ ] Replicating packwerk's behavior with respect to not recording "local definitions"
- [ ] `packs check`, which can be used as a drop-in replacement to the VSCode
- [ ] `packs update`, which can be used to update `deprecated_references.yml`
- [ ] `packs lsp`, to launch an LSP-server to provide faster feedback


# Usage
Deployment is still a WIP, and it's not quite ready for prime time. If you want to try it out to see how well it works on your repo, you can:
1. Run `rm -rf tmp/cache/packwerk` to delete the existing cache.
2. Run `packs generate-cache` (see directions below to get binary) 
3. Run `bin/packwerk update` to see how violations change using `packs`.

# Downloading the Binary
If you want to try it out:
- Go to https://github.com/alexevanczuk/packs/releases
- Download the `packs` asset and run `chmod +x path/to/packs`
- Open the containing directory, right click on the binary, click open, and then accept the warning message that says its from an unknown developer (it's me!)
- Execute `path/to/packs` to see the CLI help message.

You can add `path/to/packs` to your `PATH` so it's available in every terminal session.

In the future, I hope to:
- Somehow sign the binary so it does not get a warning message
- Make it executable before download
- Add directions to download via some other tool, or ship as a native ruby gem extension.


# New to Rust?
Me too! This is my first Rust project, so I'd love to have feedback, advice, and contributions!
If you're new to Rust, don't be intimidated! [https://www.rust-lang.org](https://www.rust-lang.org/learn) has tons of great learning resources.
