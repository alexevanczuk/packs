pub(crate) mod parser;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::packs::parsing::ruby::experimental::parser::process_from_contents;
    use crate::packs::parsing::{Definition, Range};
    use crate::packs::{ProcessedFile, UnresolvedReference};

    #[test]
    fn trivial_case() {
        let contents: String = String::from("Foo");

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![UnresolvedReference {
            name: String::from("Foo"),
            namespace_path: vec![],
            location: Range {
                start_row: 1,
                start_col: 0,
                end_row: 1,
                end_col: 4,
            },
        }];

        let definitions = vec![];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn nested_constant() {
        let contents: String = String::from("Foo::Bar");

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![UnresolvedReference {
            name: String::from("Foo::Bar"),
            namespace_path: vec![],
            location: Range {
                start_row: 1,
                start_col: 0,
                end_row: 1,
                end_col: 9,
            },
        }];

        let definitions = vec![];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz");

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![UnresolvedReference {
            name: String::from("Foo::Bar::Baz"),
            namespace_path: vec![],
            location: Range {
                start_row: 1,
                start_col: 0,
                end_row: 1,
                end_col: 14,
            },
        }];

        let definitions = vec![];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn very_deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz::Boo");

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![UnresolvedReference {
            name: String::from("Foo::Bar::Baz::Boo"),
            namespace_path: vec![],
            location: Range {
                start_row: 1,
                start_col: 0,
                end_row: 1,
                end_col: 19,
            },
        }];

        let definitions = vec![];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn class_definition_no_body() {
        let contents: String = String::from(
            "\
class Foo
end
            ",
        );

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![];

        let definitions = vec![];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn class_definition_some_body() {
        let contents: String = String::from(
            "\
class Foo
  def foo
  end
end
            ",
        );

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![];

        let definitions = vec![Definition {
            fully_qualified_name: String::from("Foo"),
            location: Range {
                start_row: 1,
                start_col: 0,
                end_row: 4,
                end_col: 3,
            },
            namespace_path: vec![],
        }];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }
}
