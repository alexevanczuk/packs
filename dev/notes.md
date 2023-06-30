# TODO
- Explore not counting definitions as references. This should not change the diff but should speed up serde. If it doesn't work, put this logic into the Experimental parser
- Explore alternate implementation that does not use constant resolver but instead actually gets definitions – implement it with an optional flag in packwerk.yml
  - This feature should have a "monkeypatches" key in packwerk.yml. This is a hash of constants and what file monkey patches them. "Validate" should check this. This can allow the alternate implementation to avoid violations on a monkey patched "String" class, for example.
- Explore alternate caching mechanisms:
  - Convert existing cache to be `PackwerkCompatibleCache`.
  - Consider using SQLite cache (for less file IO)
  - We could consider caching the RESOLVED references in a file, which would allow us to potentially skip generating the constant resolver and resolving all of the unresolved constants. This makes cache invalidation more complex though, but it might work in the happy path.
- Explore caching constant resolver

## Distribution
- Sign the binary
- Distribute with brew: https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/
- Add directions to download via some other tool, or ship as a native ruby gem extension.

# Milestones
- [x] Generate `packwerk` compatible cache with `packs generate_cache`
- [x] Parse ERB files
- [x] `packs update`, which can be used to update `package_todo.yml`
- [x] `packs check`, which can be used as a drop-in replacement to the VSCode

# Profiling
I've been using https://github.com/flamegraph-rs/flamegraph to generate flamegraphs to improve performance.

Specifically, this command which merges similar code paths to see where most of the time is spent:
```
sudo CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --profile=release --reverse --min-width=0.5 --bin=pks -- --project-root=../your_app check
```
For more, see: https://nnethercote.github.io/perf-book/profiling.html

# Local Development
## Running the CLI in release mode against a target app
```
RUST_LOG=perf_events=debug time cargo run --profile=release -- --project-root=../your_app check
```

# Packwerk Implementation Considerations
- Packwerk considers a definition to be a reference. I explored removing this in this branch: https://github.com/alexevanczuk/packs/pull/44
  - This results in a diff in violations, because if a class opens up a module defined by another class, its considered to be a reference to that other class.
  - I think this is actually a bug in packwerk, since a definition is not really a reference. Even though monkey patching / opening up other moduels is not great, we should surface that information through a different mechanism (such as allowing packs to have a monkey patches violation)
