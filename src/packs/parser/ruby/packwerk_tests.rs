#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_case() {
        let contents: String = String::from("Foo");
        assert_eq!(
            vec![Reference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 4
                }
            }],
            extract_from_contents(contents)
        );
    }

    #[test]
    fn test_nested_constant() {
        let contents: String = String::from("Foo::Bar");
        assert_eq!(
            vec![Reference {
                name: String::from("Foo::Bar"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 9
                }
            }],
            extract_from_contents(contents)
        );
    }

    #[test]
    fn test_deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz");
        assert_eq!(
            vec![Reference {
                name: String::from("Foo::Bar::Baz"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 14
                }
            }],
            extract_from_contents(contents)
        );
    }

    #[test]
    fn test_very_deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz::Boo");
        assert_eq!(
            vec![Reference {
                name: String::from("Foo::Bar::Baz::Boo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 19
                }
            }],
            extract_from_contents(contents)
        );
    }

    #[test]
    fn test_class_definition() {
        let contents: String = String::from(
            "\
class Foo
end
            ",
        );

        assert_eq!(
            vec![Reference {
                name: String::from("::Foo"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 10
                }
            }],
            extract_from_contents(contents)
        );
    }

    #[test]
    fn test_class_namespaced_constant() {
        let contents: String = String::from(
            "\
class Foo
  Bar
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Bar"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 6
                }
            },
            *extract_from_contents(contents).get(1).unwrap()
        );
    }

    #[test]
    fn test_deeply_class_namespaced_constant() {
        let contents: String = String::from(
            "\
class Foo
  class Bar
    Baz
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Baz"),
                namespace_path: vec![String::from("Foo"), String::from("Bar")],
                location: Range {
                    start_row: 3,
                    start_col: 4,
                    end_row: 3,
                    end_col: 8
                }
            },
            *extract_from_contents(contents).get(2).unwrap()
        );
    }

    #[test]
    fn test_very_deeply_class_namespaced_constant() {
        let contents: String = String::from(
            "\
class Foo
  class Bar
    class Baz
      Boo
    end
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Boo"),
                namespace_path: vec![
                    String::from("Foo"),
                    String::from("Bar"),
                    String::from("Baz")
                ],
                location: Range {
                    start_row: 4,
                    start_col: 6,
                    end_row: 4,
                    end_col: 10
                }
            },
            *extract_from_contents(contents).get(3).unwrap()
        );
    }

    #[test]
    fn test_module_namespaced_constant() {
        let contents: String = String::from(
            "\
module Foo
  Bar
end
        ",
        );

        assert_eq!(
            vec![
                Reference {
                    name: String::from("::Foo"),
                    namespace_path: vec![String::from("Foo")],
                    location: Range {
                        start_row: 1,
                        start_col: 7,
                        end_row: 1,
                        end_col: 11
                    }
                },
                Reference {
                    name: String::from("Bar"),
                    namespace_path: vec![String::from("Foo")],
                    location: Range {
                        start_row: 2,
                        start_col: 2,
                        end_row: 2,
                        end_col: 6
                    }
                }
            ],
            extract_from_contents(contents),
        );
    }

    #[test]
    fn test_deeply_module_namespaced_constant() {
        let contents: String = String::from(
            "\
module Foo
  module Bar
    Baz
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Baz"),
                namespace_path: vec![String::from("Foo"), String::from("Bar")],
                location: Range {
                    start_row: 3,
                    start_col: 4,
                    end_row: 3,
                    end_col: 8
                }
            },
            *extract_from_contents(contents).get(2).unwrap()
        );
    }

    #[test]
    fn test_very_deeply_module_namespaced_constant() {
        let contents: String = String::from(
            "\
module Foo
  module Bar
    module Baz
      Boo
    end
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Boo"),
                namespace_path: vec![
                    String::from("Foo"),
                    String::from("Bar"),
                    String::from("Baz")
                ],
                location: Range {
                    start_row: 4,
                    start_col: 6,
                    end_row: 4,
                    end_col: 10
                }
            },
            *extract_from_contents(contents).get(3).unwrap()
        );
    }

    #[test]
    fn test_mixed_namespaced_constant() {
        let contents: String = String::from(
            "\
class Foo
  module Bar
    class Baz
      Boo
    end
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Boo"),
                namespace_path: vec![
                    String::from("Foo"),
                    String::from("Bar"),
                    String::from("Baz")
                ],
                location: Range {
                    start_row: 4,
                    start_col: 6,
                    end_row: 4,
                    end_col: 10
                },
            },
            *extract_from_contents(contents).get(3).unwrap()
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_compact_style_class_definition_constant() {
        let contents: String = String::from(
            "\
class Foo::Bar
  Baz
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("Baz"),
                namespace_path: vec![String::from("Foo::Bar")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 6
                }
            },
            *extract_from_contents(contents).get(1).unwrap(),
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_compact_style_with_module_constant() {
        let contents: String = String::from(
            "\
class Foo::Bar
  module Baz
  end
end
        ",
        );

        assert_eq!(
            Reference {
                name: String::from("::Foo::Bar::Baz"),
                namespace_path: vec![
                    String::from("Foo::Bar"),
                    String::from("Baz")
                ],
                location: Range {
                    start_row: 2,
                    start_col: 9,
                    end_row: 2,
                    end_col: 13
                }
            },
            *extract_from_contents(contents).get(1).unwrap()
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_constant() {
        let contents: String = String::from("[Foo]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 1,
                    end_row: 1,
                    end_col: 5
                }
            },
            *reference
        );
    }
    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_multiple_constants() {
        let contents: String = String::from("[Foo, Bar]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let reference1 = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 1,
                    end_row: 1,
                    end_col: 5
                }
            },
            *reference1
        );
        let reference2 = references
            .get(1)
            .expect("There should be a reference at index 1");
        assert_eq!(
            Reference {
                name: String::from("Bar"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 10
                }
            },
            *reference2,
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_nested_constant() {
        let contents: String = String::from("[Baz::Boo]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("Baz::Boo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 1,
                    end_row: 1,
                    end_col: 10
                }
            },
            *reference,
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_globally_referenced_constant() {
        let contents: String = String::from("::Foo");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("::Foo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 6
                }
            },
            *reference,
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_metaprogrammatically_referenced_constant() {
        let contents: String = String::from("described_class::Foo");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 0);
    }

    #[test]
    fn test_ignore_local_constant() {
        let contents: String = String::from(
            "\
class Foo
  BAR = 1
  def use_bar
    puts BAR
  end
end
        ",
        );

        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("::Foo"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 10
                }
            }]
        )
    }

    #[test]
    fn test_ignore_local_constant_under_nested_module() {
        let contents: String = String::from(
            "\
class Foo
  class Baz
    BAR = 1
    def use_bar
      puts BAR
    end
  end
end
        ",
        );

        assert_eq!(
            extract_from_contents(contents),
            vec![
                Reference {
                    name: String::from("::Foo"),
                    namespace_path: vec![String::from("Foo"),],
                    location: Range {
                        start_row: 1,
                        start_col: 6,
                        end_row: 1,
                        end_col: 10
                    }
                },
                Reference {
                    name: String::from("::Foo::Baz"),
                    namespace_path: vec![
                        String::from("Foo"),
                        String::from("Baz")
                    ],
                    location: Range {
                        start_row: 2,
                        start_col: 8,
                        end_row: 2,
                        end_col: 12
                    }
                }
            ]
        );
    }

    #[test]
    fn test_super_classes_are_references() {
        let contents: String = String::from(
            "\
class Foo < Bar
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("Bar"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 12,
                    end_row: 1,
                    end_col: 16
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_compact_nested_classes_are_references() {
        let contents: String = String::from(
            "\
class Foo::Bar
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let first_reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("::Foo::Bar"),
                namespace_path: vec![String::from("Foo::Bar")],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 15
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_regular_nested_classes_are_references() {
        let contents: String = String::from(
            "\
class Foo
  class Bar
  end
end
        ",
        );

        let references: Vec<Reference> = extract_from_contents(contents);
        assert_eq!(
            references,
            vec![
                Reference {
                    name: String::from("::Foo"),
                    namespace_path: vec![String::from("Foo")],
                    location: Range {
                        start_row: 1,
                        start_col: 6,
                        end_row: 1,
                        end_col: 10
                    }
                },
                Reference {
                    name: String::from("::Foo::Bar"),
                    namespace_path: vec![
                        String::from("Foo"),
                        String::from("Bar")
                    ],
                    location: Range {
                        start_row: 2,
                        start_col: 8,
                        end_row: 2,
                        end_col: 12
                    }
                }
            ]
        );
    }
    #[test]
    fn test_const_assignments_are_references() {
        let contents: String = String::from(
            "\
FOO = BAR
",
        );
        let references: Vec<Reference> = extract_from_contents(contents);

        assert_eq!(references.len(), 1);
        let first_reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("BAR"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 10
                }
            },
            *first_reference
        )
    }

    #[test]
    fn test_has_one_association() {
        let contents: String = String::from(
            "\
class Foo
  has_one :some_user_model
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("SomeUserModel"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 27
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_has_one_association_with_class_name() {
        let contents: String = String::from(
            "\
class Foo
  has_one :some_user_model, class_name: 'User'
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("User"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 47
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_has_many_association() {
        let contents: String = String::from(
            "\
class Foo
  has_many :some_user_models
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("SomeUserModel"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 29
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_has_many_association_with_custom_inflection() {
        let contents: String = String::from(
            "\
class Foo
  has_many :my_statuses
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("MyStatus"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 24
                }
            },
            *first_reference,
        );
    }
    #[test]
    fn test_has_many_association_with_custom_inflection_2() {
        let contents: String = String::from(
            "\
class Foo
  has_many :my_leaves
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("MyLeave"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 22
                }
            },
            *first_reference,
        );
    }

    #[test]
    #[ignore]
    fn test_has_many_association_with_class_name_after_block() {
        let contents: String = String::from(
            "\
class Foo
  has_one :bar, -> { my_scope }, as: :owner, class_name: 'bar_class'
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(1)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("MyLeave"),
                namespace_path: vec![String::from("Foo")],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 22
                }
            },
            *first_reference,
        );
    }

    #[test]
    fn test_it_uses_the_namespace_of_inherited_class_when_referencing_inherited_class(
    ) {
        let contents: String = String::from(
            "\
class Foo < Bar
  Bar
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 3);
        let reference = references
            .get(2)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("Bar"),
                namespace_path: vec![],
                location: Range {
                    start_row: 2,
                    start_col: 2,
                    end_row: 2,
                    end_col: 6
                }
            },
            *reference,
        );
    }

    #[test]
    fn test_it_ignores_locally_defined_nested_constants() {
        let contents: String = String::from(
            "\
class Foo
  class Bar
    Foo::Bar
  end
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let first_reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        let second_reference = references
            .get(1)
            .expect("There should be a reference at index 0");

        assert_eq!(first_reference.name, String::from("::Foo"));
        assert_eq!(second_reference.name, String::from("::Foo::Bar"));
    }

    #[test]
    fn test_it_ignores_references_to_subsets_of_locally_defined_nested_constants(
    ) {
        let contents: String = String::from(
            "\
class Foo::Bar
  Foo
end
        ",
        );

        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references
            .get(0)
            .expect("There should be a reference at index 0");
        assert_eq!(
            Reference {
                name: String::from("::Foo::Bar"),
                namespace_path: vec![String::from("Foo::Bar")],
                location: Range {
                    start_row: 1,
                    start_col: 6,
                    end_row: 1,
                    end_col: 15
                }
            },
            *reference,
        );
    }
}
