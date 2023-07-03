# Experimental Parser Usage

TLDR:
- `packs --experimental-parser update` and `packs --experimental-parser check` OR use `experimental_parser: true` in your `packwerk.yml`.
- When switching between the `experimental` and `packwerk` parser, run `packs delete-cache` – the two caches are not compatible!
- `packwerk` infers constant definitions based on file names
- The `experimental` parser explicitly parses constant definitions from files
- There are some limitations still that might produce unexpected behavior. Please share your feedback!

## What's the difference?
First, some context:
- Packs builds a graph of each package, the files within those packages, the constants (i.e. class, modules, or CONSTANTS) referenced within those files, and the constants defined within those files.
- The *packwerk* parser will parse files for references, but it has some quirks:
  - A *definition* counts as a reference. So `class Foo::Bar; end` is a reference to both "Foo" and "Foo::Bar". This means that if `Foo` is defined in another pack, it might show up as a violation.
  - Packwerk uses zeitwerk conventions (hence the name) to infer file definitions. So for example, `foo/bar.rb` defines `Foo::Bar`. It uses various Rails conventions (autoload paths, inflections, etc.) to infer what constants a path defines.
  - As a result of this, it has some limitations:
    - It cannot be used in non-Rails apps, or Rails apps that do not follow zeitwerk conventions (meaning it can't parse non-autoloaded code).
    - A file can only be considered to define exactly one constant, which is the constant that matches the file name.
- The *experimental* parser, in contrast, works as follows:
  - A reference is parsed just like it is with the `packwerk` parser, except definitions do not count as references.
  - Definitions are parsed directly from the file, rather than inferring them from file names.
  - We could consider `module Foo; class Bar; end; end` to define both `Foo` and `Foo::Bar`, since it opens up `Foo`. The approach the experimental parser takes is that any file defines a constant if it changes behavior within that constant. So for example, `module Foo; class Bar; end; end` actually defines nothing (since it does not change behavior). `module Foo; class Bar; def bar; end; end; end;` defines `Foo::Bar` (since it changes behavior within `Foo::Bar`), and `module Foo; def foo; end; class Bar; def bar; end; end; end;` defines both `Foo` and `Foo::Bar` (since it changes behavior within both).

## Usage Notes
- See usage with `packs --help`:
  - TLDR `packs -e update` and `packs -e check` OR use `experimental_parser: true` in your `packwerk.yml`.
- This is experimental API that could change!
- While the cache formats for the two parsers are the same, the packwerk resolver always caches an empty list of definitions. To switch between the two parsers and have expected results, you must clear the cache. You can do this with `packs delete-cache`.
- If you're unclear where `packs` thinks a constant is defined, you can use `packs -e list-definitions`

# Upcoming Developments + Limitations
- Over time, we'll want to refine how we handle monkey patches. Alternative implementations are:
  - We could consider *every* time a constant is opened up (i.e. a `class` or `module` keyword) to be "defining" a constant. This would mean that tons of files define the same constants. This is not a problem unless *different packs* define the same constant. This implementation would be very strict against monkey patches.
  - We could allow a monkey patch to be defined within `packwerk.yml`, so that it can be ignored as a definition. For example, if the root pack opens up `String`, we might have `String: config/initializers/string_extensions.rb` in our `packwerk.yml`.
- Right now, if multiple files define the same constant, we just choose the *first* one. This is not an ideal implementation at all. Instead, we should think about having constants be able to be defined by multiple files. Instead of having a "primary" definition, we can create one reference for each definition. For example, if `packs/a` and `packs/b` define `Foo`, then using `Foo` creates one reference to each of those packs.

