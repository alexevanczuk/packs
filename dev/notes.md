# TODO
- Explore alternate implementation of extractor that does not use constant resolver but instead actually gets definitions – implement it with an optional flag in packwerk.yml and/or CLI flag.
  - This feature should have a "monkeypatches" key in packwerk.yml. This is a hash of constants and what file monkey patches them. "Validate" should check this. This can allow the alternate implementation to avoid violations on a monkey patched "String" class, for example.
  - This parser thinks a constant can be defined in many places. For each place, it establishes one reference.
  - This parser does not consider definitions to be references. (This is a bug in the current implementation – see below.)
- Explore alternate caching mechanisms:
  - Convert existing cache to be `PackwerkCompatibleCache`.
  - Consider using SQLite cache (for less file IO)
  - We could consider caching the RESOLVED references in a file, which would allow us to potentially skip generating the constant resolver and resolving all of the unresolved constants. This makes cache invalidation more complex though, but it might work in the happy path.
- Reduce cost of packs::for_file by building cache during directory walk, see https://github.com/alexevanczuk/packs/pull/46 for starter
- Implement cycle detection within check command, see https://docs.rs/petgraph/latest/petgraph/algo/index.html

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
  - Note this logic can be moved into the experimental parser, since it does not need to preserve behavior.

# Abandoned Performance Improvement Attempts
- In https://github.com/alexevanczuk/packs/pull/37, I looked into getting the constants *as* we are walking the directory. However, I found that this was hardly much more performant than the current implementation, and it was much more complex. I abandoned this approach in favor of caching the resolver and other performance improvements.

# Modular Architecture
Today, `packwerk` has a modular architecture allowing folks to add new checkers, validators, etc.
Eventually, I'd like to port this idea over to `packs`.
We might consider how we can have specific checkers/validators be responsible for their own portion of the deserialized properties in `package.yml` files.
