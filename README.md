# packs
![CI](https://github.com/alexevanczuk/packs/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/alexevanczuk/packs/actions/workflows/audit.yml/badge.svg)

WIP: Rust implementation of [packs](https://github.com/rubyatscale/use_packs) and [packwerk](https://github.com/Shopify/packwerk) for ruby

# Features
- It's entirely built in Rust, so it's really fast, and doesn't require any external dependencies since the binary contains everything that needed to run `packs`!
- Currently all `packs` can do is generate a cache to be used by the ruby implementation.

# Usage
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

# Verification
As `packs` is still a work-in-progress, it's possible it will not produce the same results as the ruby implementation (see below). If not, please file an issue!

To verify:
1. Run `rm -rf tmp/cache/packwerk` to delete the existing cache.
2. Run `packs generate_cache` (see directions below to get binary)
3. Run `bin/packwerk update` to see how violations change using `packs`.

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
- custom associations
- custom inflections
- custom load paths
- zeitwerk default namespaces

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
