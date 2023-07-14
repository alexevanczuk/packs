# Experimental Parser Usage

# TLDR:
## Why
- This allows `packs` to work in Ruby applications that are not compatible with `packwerk` (e.g. non-Rails and/or non-Zeitwerk apps)
- See usage with `packs --help`:
  - `packs -e update` and `packs -e check` OR use `experimental_parser: true` in your `packwerk.yml`.
- `packwerk` infers constant definitions based on file names
- The `experimental` parser explicitly parses constant definitions from files
- There are some limitations still that might produce unexpected behavior. Please share your feedback!
- This is experimental API that could change!

# Other Usage
## Ignoring definitions
You may want to ignore a definition, such as if there is a monkey patch that you do not want considered or any other issue.

You can configure this in `packwerk.yml` like so:
```yml
ignored_definitions:
  ::String:
    - lib/monkey_patches.rb
```

## Finding multiple definitions
With the experimental parser, a reference to a constant defined in N places produces N references.

To find these constants defined in multiple locations, you can run:
`packs -e list-definitions --ambiguous`

# What's the difference?
Here are some example definitions which I'll refer to below:
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
  - The approach the experimental parser takes is that any file defines a constant if it changes behavior within that constant. So for example, `foo/bar.rb` actually defines nothing (since it does not change behavior). `foo/baz.rb` defines `Foo::Baz` (since it changes behavior within `Foo::Baz`), and `foo/boo.rb` defines both `Foo` and `Foo::Boo` (since it changes behavior within both).

# Limitations
- There may be some definition constructs that are not properly parsed yet.

# Alternative Implementations
- We could consider *every* time a constant is opened up (i.e. a `class` or `module` keyword) to be "defining" a constant. This would mean that tons of files define the same constants. This is not a problem unless *different packs* define the same constant. This implementation would be very strict against monkey patches.
- We might force the user to define a primary definition when a constant is defined in multiple places

# Advantages
- Simpler – parsing files directly is conceptually simpler than inferring constants from file names based on zeitwerk conventions, which require handling of inflections, default namespaces, collapsed directories, and more. The implementation is simpler to maintain as well.
  - This makes the behavior easier to understand, too. In `packwerk`, a reference is also considered a definition.
- More applicable – allows `packs` to be used in non-Rails, non-Zeitwerk apps, such as gems. This also provides the basis of other interesting features, like detecting the use of specific gems in packages.
- Richer feature opportunities – provides platform for other possible features like monkey-patch detection.
