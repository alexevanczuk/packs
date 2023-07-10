use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::packs::{
    constant_resolver::{ConstantDefinition, ConstantResolver},
    ProcessedFile,
};

use self::constant_resolver::ExperimentalConstantResolver;

pub mod constant_resolver;

pub(crate) mod parser;

pub fn get_experimental_constant_resolver(
    absolute_root: &Path,
    processed_files: &Vec<ProcessedFile>,
    ignored_definitions: &HashMap<String, HashSet<PathBuf>>,
) -> Box<dyn ConstantResolver + Send + Sync> {
    let constants = processed_files
        .into_par_iter()
        .flat_map(|processed_file| {
            processed_file
                .definitions
                .iter()
                .map(|definition| {
                    let fully_qualified_name =
                        definition.fully_qualified_name.to_owned();
                    ConstantDefinition {
                        fully_qualified_name,
                        absolute_path_of_definition: processed_file
                            .absolute_path
                            .to_owned(),
                    }
                })
                .collect::<Vec<ConstantDefinition>>()
        })
        .collect::<Vec<ConstantDefinition>>();

    ExperimentalConstantResolver::create(
        constants,
        absolute_root,
        ignored_definitions,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::packs::parsing::ruby::experimental::parser::process_from_contents;
    use crate::packs::parsing::{ParsedDefinition, Range};
    use crate::packs::{ProcessedFile, UnresolvedReference};
    use pretty_assertions::assert_eq;

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

        let definitions = vec![ParsedDefinition {
            fully_qualified_name: String::from("::Foo"),
            location: Range {
                start_row: 1,
                start_col: 6,
                end_row: 1,
                end_col: 10,
            },
        }];

        let actual = process_from_contents(contents, &absolute_path);
        let expected = ProcessedFile {
            absolute_path,
            unresolved_references,
            definitions,
        };
        assert_eq!(expected, actual);
    }

    #[test]
    fn class_definition_some_body_with_class_method() {
        let contents: String = String::from(
            "\
class Foo
  def self.foo
  end
end
            ",
        );

        let absolute_path = PathBuf::from("path/to/file.rb");
        let unresolved_references = vec![];

        let definitions = vec![ParsedDefinition {
            fully_qualified_name: String::from("::Foo"),
            location: Range {
                start_row: 1,
                start_col: 6,
                end_row: 1,
                end_col: 10,
            },
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
