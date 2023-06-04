# TODO
- Update `scripts/packwerk_parity_checker.rb` to ensure the exact same set of files are produced (i.e. `include` and `exclude` should be respected)
- Add benchmarking for `packs generate_cache` against `packwerk` if the same set of files are produced
- Improve deployment and share current progress
- Look into `bin/packwerk update`!
- Make sure cache works like this:
  - t(hread)1: Open cache to get cache entry
  - t2: Get digest of file
  - join threads and compare digests
  - if not equal, parse file to get unresolved references

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
