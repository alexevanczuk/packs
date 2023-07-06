# Experimental Parser Usage

TLDR:
- This allows `packs` to work in Ruby applications that are not compatible with `packwerk` (e.g. non-Rails and/or non-Zeitwerk apps)
- `packs --experimental-parser update` and `packs --experimental-parser check` OR use `experimental_parser: true` in your `packwerk.yml`.
- When switching between the `experimental` and `packwerk` parser, run `packs delete-cache` – the two caches are not compatible!
- `packwerk` infers constant definitions based on file names
- The `experimental` parser explicitly parses constant definitions from files
- There are some limitations still that might produce unexpected behavior. Please share your feedback!

## What's the difference?
Let's start with some example:
```ruby
# foo.rb
class Foo; end
```

```ruby
# foo/bar.rb
class Foo
  class Bar
  end
end
```

```ruby
# foo/baz.rb
class Foo
  class Baz
    def baz
    end
  end
end
```

```ruby
# foo/boo.rb
class Foo
  def foo
  end

  class Boo
    def boo
    end
  end
end
```

First, some context:
- Packs builds a graph of each package, the files within those packages, the constants (i.e. class, modules, or CONSTANTS) referenced within those files, and the constants defined within those files.
- The *packwerk* parser will parse files for references, but it has some quirks:
  - A *definition* counts as a reference. So `foo/bar.rb` includes references to both `Foo` and `Foo::Bar`. This means that if `Foo` is defined in another pack, it might show up as a violation.
  - Packwerk uses zeitwerk conventions (hence the name) to infer file definitions. So for example, `foo/bar.rb` defines `Foo::Bar`. It uses various Rails conventions (autoload paths, inflections, etc.) to infer what constants a path defines.
  - As a result of this, it has some limitations:
    - It cannot be used in non-Rails apps, or Rails apps that do not follow zeitwerk conventions (meaning it can't parse non-autoloaded code).
    - A file can only be considered to define exactly one constant, which is the constant that matches the file name.
- The *experimental* parser, in contrast, works as follows:
  - A reference is parsed just like it is with the `packwerk` parser, except definitions do not count as references.
  - Definitions are parsed directly from the file, rather than inferring them from file names.
  - We could consider `foo/bar.rb` to define both `Foo` and `Foo::Bar`, since it opens up `Foo`. However, this might cause very noisy results when multiple packs open up the same namespaces.
  - The approach the experimental parser takes is that any file defines a constant if it changes behavior within that constant. So for example, `foo/bar.rb` actually defines nothing (since it does not change behavior). `foo/baz.rb` defines `Foo::Baz` (since it changes behavior within `Foo::Baz`), and `foo/boo.rb` defines both `Foo` and `Foo::Boo` (since it changes behavior within both).

## Usage Notes
- See usage with `packs --help`:
  - TLDR `packs -e update` and `packs -e check` OR use `experimental_parser: true` in your `packwerk.yml`.
- This is experimental API that could change!
- While the cache formats for the two parsers are the same, the packwerk resolver always caches an empty list of definitions. To switch between the two parsers and have expected results, you must clear the cache. You can do this with `packs delete-cache`.
- If you're unclear where `packs` thinks a constant is defined, you can use `packs -e list-definitions`

# Upcoming Developments + Limitations
- There may be some definition constructs that are not properly parsed yet.
- There is currently no strong support for ignoring monkey patches. For example, if one pack monkey patches `String`, then every pack would get a violation against that pack every time they use `String`. I'm looking into allowing a pack to register its monkey patches, thereby ignoring them. Similarly, we could have a function like `list-monkey-patches` which lists all constants that are defined multiple times. 
  - We could actually consider pointing `packs` at the ruby standard library to get all constants defined by the standard library. We could hard-code this list in `packs` so that we could get a clear list of places where an app monkey patches the standard library
  - We could consider *every* time a constant is opened up (i.e. a `class` or `module` keyword) to be "defining" a constant. This would mean that tons of files define the same constants. This is not a problem unless *different packs* define the same constant. This implementation would be very strict against monkey patches.
  - We could allow a monkey patch to be defined within `packwerk.yml`, so that it can be ignored as a definition. For example, if the root pack opens up `String`, we might have `String: config/initializers/string_extensions.rb` in our `packwerk.yml`.
- Right now, if multiple files define the same constant, we just choose the *first* one. This is not an ideal implementation at all. Instead, we should think about having constants be able to be defined by multiple files. Instead of having a "primary" definition, we can create one reference for each definition. For example, if `packs/a` and `packs/b` define `Foo`, then using `Foo` creates one reference to each of those packs.

# Advantages
- Simpler – parsing files directly is conceptually simpler than inferring constants from file names based on zeitwerk conventions, which require handling of inflections, default namespaces, collapsed directories, and more. The implementation is simpler to maintain as well.
  - This makes the behavior easier to understand, too. In `packwerk`, a reference is also considered a definition.
- More applicable – allows `packs` to be used in non-Rails, non-Zeitwerk apps, such as gems. This also provides the basis of other interesting features, like detecting the use of specific gems in packages.
- Richer feature opportunities – provides platform for other possible features like monkey-patch detection.
