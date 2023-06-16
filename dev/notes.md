# TODO
## Perforamnce
- Create new cache implementation that creates one large file and only uses cache when no files are inputted. Could consider using `rustql`
- Convert existing cache to be `PackwerkCompatibleCache`.
- We end up looking at a lot more references than necessary because we record class and superclass definitions each as their own reference.
  - See tests in `src/packs/parser/ruby/packwerk.rs` for examples.
  - If intead we did *not* consider these references, we'd save a lot of time resolving constants and running checkers.
  - A second parser implementation could do this and be specified in packwerk.yml perhaps?
- if files are inputted into generate_cache, we should compare them to include/exclude globs rather than doing the directory walk
- consider: create two CLIs: `generate_cache_cold` and `generate_cache`. The latter reuses existing caches if the digests match.
- look for additional speed ups for cold cache generation, mostly in parsing logic. Consider progress bar.
- We could consider caching the RESOLVED references in a file, which would allow us to potentially skip generating the constant resolver and resolving all of the unresolved constants. This makes cache invalidation more complex though, but it might work in the happy path.

## Features
- Remove `packwerk_parity_checker`, as it should be replaced with simply using `packs update` to compare accuracy.

## Deployment
- Sign the binary

# Milestones
- [x] Generate `packwerk` compatible cache with `packs generate_cache`
- [x] Parse ERB files
- [ ] `packs check`, which can be used as a drop-in replacement to the VSCode
- [ ] `packs update`, which can be used to update `package_todo.yml`

# Other notes
## Wrap Packwerk
One simple way to try out `packs` to generate your cache would be to create a bash function which wraps the call to `bin/packwerk`, like so:
```bash
# In your ~/.bash_profile or analogous file
packwerk() {
    if [ "$1" = "check" ] || [ "$1" = "update" ]; then
        echo "Calling packs generate_cache with args: ${@:2}"
        packs generate_cache "${@:2}"
    fi

    echo "Now calling packwerk with args: $@"
    bin/packwerk "$@"
}
```

You can also modify the `bin/packwerk` executable to call `packs` conditionally, e.g.
```ruby
# In bin/packwerk
packs_executable = `which packs`.chomp
if !packs_executable.empty?
  if ARGV.first == 'check' || ARGV.first == 'update'
    puts "Calling packs generate_cache with args: #{ARGV[1..-1]}"
    system('packs', 'generate_cache', *ARGV[1..-1])
  end
end
```
