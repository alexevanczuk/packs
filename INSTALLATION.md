# Installation
## Option 1:
- Install Rust: https://www.rust-lang.org/tools/install
  - Note: If `which cargo` returns a path, skip this step!
  - TLDR: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`, and you're done!
- `cargo install pks` (it's like `gem install`)

## Option 2:
(Mac only – for other platforms, please create an issue/PR or try option 1.)

- Go to https://github.com/alexevanczuk/packs/releases
- Download the `packs` asset and run `chmod +x path/to/packs`. This makes the asset executable on your machine.
- Open the containing directory, right click on the binary, click open, and then accept the warning message that says its from an unknown developer (it's me!)
- Execute `path/to/packs` to see the CLI help message.

You can add `path/to/packs` to your `PATH` so it's available in every terminal session.

## Option 3 (coming soon):
I'm looking into installing via `brew` or as a native ruby gem extension. More coming soon!

## Option 4:
- Install [dotslash](https://dotslash-cli.com/docs/installation/)
- Download the latest packs release dotslash `pks` file. Example: https://github.com/alexevanczuk/packs/releases/download/v0.2.8/pks
- Save the `pks` file to your ruby project's bin/ directory. You'll then have a `bin/pks` file in your project.
- Use `bin/pks` to execute the CLI.
