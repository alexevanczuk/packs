# TODO
- Create new cache implementation that creates one large file and only uses cache when no files are inputted
- Convert existing cache to be `PackwerkCompatibleCache`.
- We end up looking at a lot more references than necessary because we record class and superclass definitions each as their own reference.
  - See tests in `src/packs/parser/ruby/packwerk.rs` for examples.
  - If intead we did *not* consider these references, we'd save a lot of time resolving constants and running checkers.
  - A second parser implementation could do this and be specified in packwerk.yml perhaps?
- Update `scripts/packwerk_parity_checker.rb` to ensure the exact same set of files are produced (i.e. `include` and `exclude` should be respected)
- Add benchmarking for `packs generate_cache` against `packwerk` if the same set of files are produced
- Improve deployment and share current progress
- Look into `bin/packwerk update`!
- Make sure cache works like this:
  - t(hread)1: Open cache to get cache entry
  - t2: Get digest of file
  - join threads and compare digests
  - if not equal, parse file to get unresolved references
- look for additional speed ups for cold cache generation. Consider progress bar.
- create two CLIs: `generate_cache_cold` and `generate_cache`. The latter reuses existing caches if the digests match.
- if files are inputted into generate_cache, we should compare them to include/exclude globs rather than doing the directory walk
- We could consider caching the RESOLVED references in a file, which would allow us to potentially skip generating the constant resolver and resolving all of the unresolved constants. This makes cache invalidation more complex though, but it might work in the happy path.
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
