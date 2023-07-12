# TODO
## Features
- `packs init | create | move`
- CLI could have `-i` interactive mode (like `use_packs`, also see https://github.com/mikaelmello/inquire)
- Privacy violation inversion?

## Performance
Although `packs` is intended to be fast, there are ways it can be made a lot faster!

- Explore alternate caching mechanisms:
  - Convert existing cache to be `PackwerkCompatibleCache`.
  - Consider using SQLite cache (for less file IO)
  - We could consider caching the RESOLVED references in a file, which would allow us to potentially skip generating the constant resolver and resolving all of the unresolved constants. This makes cache invalidation more complex though, but it might work in the happy path.
- Conditional cache usage. For example, implemented as an LSP, packs could always use cache and only bust specific caches (asychronously) when certain events (e.g. file changes) are received.
- By using modified time, we can avoid opening the entire file and parsing it and calculating the md5 hash. It's possible this would not be a meaningful performance improvement.

### Improved use of references (less cloning)
As I'm new to Rust, I don't take advantage of a lot of features in Rust that would improve the performance, such as making sure I minimize the use of deep clones and use references.

# Distribution Considerations
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

Install with:
```
cargo install flamegraph
```

I have aliased this command which generates something like a "left-shifted" flamegraph, to show where most of the time is spent:
```
alias profile_packs='CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --profile=release --reverse --min-width=0.5 -- --project-root="../your_app" check'
```
For more, see: https://nnethercote.github.io/perf-book/profiling.html

# Local Development
## Running the CLI in release mode against a target app
```
time cargo run --profile=release -- --debug --project-root=../your_app check
```

# Packwerk Implementation Considerations
- See `EXPERIMENTAL_PARSER_USAGE.md` for more info
- Packwerk considers a definition to be a reference. I explored removing this in this branch: https://github.com/alexevanczuk/packs/pull/44
  - This results in a diff in violations, because if a class opens up a module defined by another class, its considered to be a reference to that other class.
  - I think this is actually a bug in packwerk, since a definition is not really a reference. Even though monkey patching / opening up other moduels is not great, we should surface that information through a different mechanism (such as allowing packs to have a monkey patches violation)

# Abandoned Performance Improvement Attempts
- In https://github.com/alexevanczuk/packs/pull/37, I looked into getting the constants *as* we are walking the directory. However, I found that this was hardly much more performant than the current implementation, and it was much more complex. I abandoned this approach in favor of caching the resolver and other performance improvements.

# Modular Architecture
Today, `packwerk` has a modular architecture allowing folks to add new checkers, validators, etc.
Eventually, I'd like to port this idea over to `packs`.
We might consider how we can have specific checkers/validators be responsible for their own portion of the deserialized properties in `package.yml` files.

# Feature Ideas
## Monkey-patch detection?
Example:
```
$ packs --experimental-parser expose-monkey-patches --rubydir="/Users/alexevanczuk/.rbenv/versions/3.2.2/lib/ruby/3.2.0/" --gemdir="/Users/alexevanczuk/.rbenv/versions/3.2.2/lib/ruby/gems/3.2.0/gems"

The following is a list of constants that are redefined by your app.

# Ruby Standard Library
These monkey patches redefine behavior in the Ruby standard library (as determined by parsing the contents of `/Users/alexevanczuk/.rbenv/versions/3.2.2/lib/ruby/3.2.0/`):

::String is redefined at lib/string_extensions.rb
::Date is redefined at lib/date_extensions.rb

# Gems
These monkey patches redefine behavior in gems your app depends on (as determined by parsing the contents of `vendor/bundle`):

::Rails from gem `rails` is redefined at path/to/redefinition.rb
::Thor from gem `thor` is redefined at other/path/to/redefinition.rb

# app
These monkey patches redefine behavior in a pack within your app (as determined by parsing your app's packs):

::Foo is defined at packs/foo/app/services/foo.rb
::Foo is defined at packs/bar/app/models/foo.rb
```

Error mode:
```
$ packs expose-monkey-patches --rubydir="/Users/alexevanczuk/.rbenv/versions/3.2.2/lib/ruby/3.2.0/" --gemdir="/Users/alexevanczuk/.rbenv/versions/3.2.2/lib/ruby/gems/3.2.0/gems"

Error: This command is only supported with the experimental parser. See documentation for more information. Please file an issue if you have questions!
```

The general implementation of this would be to call `process_files_with_cache` on the globbed out rubydir/gemdir and on the project's included files. Then we build an (experimental) constant resolver with the processed files.

In the constant resolver, for each constant we check if any of the definitions are in ruby, in a gem, or in our app. There are 9 possibilities, so we can use rust pattern matching and build a truth table:
```rust
match (defined_in_stdlib, defined_in_gems, defined_in_app, defined_in_app_count) {
  // Ruby section monkey patches
  (true, false, false, _) => {} // skip: ruby only
  (true, true, false, _) => {} // skip: gems monkey patching ruby
  (true, _, true, _) => {} // ruby_section += [*stdlib_definitions, *app_definitions]

  // Gem monkey patches
  (false, true, _, _) => {} // gem_section += [*gem_definitions, *app_definitions]
  
  // Application monkey patches
  (_, _, true, 1) => {} // skip: single definition in app
  (_, _, true, _) => {} // app_section += [*app_definitions]  
}
```

With this, I think we could remove `--ambiguous` from `list-definitions` in favor of `expose-monkey-patches`.
