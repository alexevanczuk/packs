# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

WIP: Rust implementation of packs for ruby

# Initial Milestone

- [ ] `packs generate_cache`, which can be used to update `tmp/cache/packwerk` for faster `packwerk` output. It should produce the exact same `json` that `packwerk` produces today. Challenges include:
  - [ ] Parsing ERB
  - [ ] Parsing Rails associations and rewriting them as constant references using a pluralizer. Initially, non-standard inflections will likely not be supported (although I may support it through hard-coded map in `packwerk.yml`)
  - [ ] Replicating packwerk's behavior with respect to not recording "local definitions"
- [ ] `packs check`, which can be used as a drop-in replacement to the VSCode
- [ ] `packs update`, which can be used to update `deprecated_references.yml`
- [ ] `packs lsp`, to launch an LSP-server to provide faster feedback
