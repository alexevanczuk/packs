# Installation
## Option 1:
- Install Rust: https://www.rust-lang.org/tools/install
  - TLDR: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, and you're done!
- `cargo install pks` (it's like `gem install`)

(Note – if you're using [`use_packs`]([url](https://github.com/rubyatscale/use_packs)) AND [`chruby`]([url](https://github.com/capistrano/chruby)), `chruby` might overwrite the `packs` executable with the `packs` executable from `use_packs`. If so, I'd recommend making an alias, e.g. `pks` that points to the cargo installed executable.

## Option 2:
(Mac only – for other platforms, please create an issue/PR or try option 1.)

- Go to https://github.com/alexevanczuk/packs/releases
- Download the `packs` asset and run `chmod +x path/to/packs`. This makes the asset executable on your machine.
- Open the containing directory, right click on the binary, click open, and then accept the warning message that says its from an unknown developer (it's me!)
- Execute `path/to/packs` to see the CLI help message.

You can add `path/to/packs` to your `PATH` so it's available in every terminal session.

## Option 3 (coming soon):
I'm looking into installing via `brew` or as a native ruby gem extension. More coming soon!

