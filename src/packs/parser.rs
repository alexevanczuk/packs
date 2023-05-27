use glob::glob;
use lib_ruby_parser::{nodes, traverse::visitor::Visitor, Node, Parser, ParserOptions};
use rayon::prelude::*;
use std::{fs, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct Reference {
    name: String,
    // class Foo
    //   module Bar
    //     class Baz
    //       puts Module.nesting.inspect
    //     end
    //   end
    // end
    // # outputs: [Foo::Bar::Baz, Foo::Bar, Foo]
    module_nesting: Vec<String>,
}

enum VisitorState {
    DefiningConstant,
    ReferencingConstant,
    Initial,
}
struct ReferenceCollector {
    pub references: Vec<Reference>,
    pub current_namespaces: Vec<String>,
    pub state: VisitorState,
}

fn get_constant_node_name(name: &Node) -> String {
    match name {
        Node::Const(node) => fetch_const_const_name(node),
        other => panic!("Cannot handle other node in get_constant_node_name"),
    }
}

impl Visitor for ReferenceCollector {
    fn on_class(&mut self, node: &nodes::Class) {
        // We're not collecting definitions, so no need to visit the class definition
        // self.visit(&node.name);
        let namespace = get_constant_node_name(&node.name);
        // We're not visiting super classes either
        // if let Some(inner) = node.superclass.as_ref() {
        //     self.visit(inner);
        // }
        if let Some(inner) = node.body.as_ref() {
            let class_or_module_name = match node.name {
                Node::Const(c) => fetch_const_node(&c),
                _other => todo!(),
            };
            self.current_namespaces.push(node.name);
            self.visit(inner);
        }
    }
    fn on_const(&mut self, node: &nodes::Const) {
        // match self.state {
        //     VisitorState::DefiningConstant => {

        //     },
        //     VisitorState::ReferencingConstant => todo!(),
        //     VisitorState::Initial => todo!(),
        // }

        self.references.push(Reference {
            name: fetch_const_node(node),
            module_nesting: vec![],
        })

        // if let Some(parent_const_node) = node.scope {
        //     match *parent_const_node {
        //         Node::Const(parent_const) => {
        //             // self.state = ReferencingConstant
        //             format!("{}::{}", visitor::visit_const(self, node))
        //         }
        //         _other => node.name,
        //     }
        // } else {
        //     node.name
        // }
    }
}

fn fetch_const_node(node: &nodes::Const) -> String {
    if let Some(scope) = &node.scope {
        format!("{}::{}", fetch_const_scope_name(scope), node.name)
    } else {
        node.name.to_owned()
    }
}

fn fetch_const_scope_name(scope: &nodes::Node) -> String {
    match scope {
        Node::Cbase(_) | Node::Self_(_) | Node::Send(_) | Node::Lvar(_) | Node::Ivar(_) => "".to_owned(),
        Node::Const(node) => fetch_const_node(node),
        other => panic!("Don't know how to fetch const name from {:?}", other),
    }
}

fn fetch_const_name(const_node: &Node) -> String {
    match const_node {
        Node::Const(nodes::Const { name, .. }) => name.to_owned(),
        other => panic!("Don't know how to fetch const name from {:?}", other),
    }
}

pub fn get_references(absolute_root: PathBuf) -> Vec<Reference> {
    // Later this can come from config
    let pattern = absolute_root.join("packs/**/*.rb");

    let x = glob(pattern.to_str().unwrap())
        .expect("Failed to read glob pattern")
        .par_bridge() // Parallel iterator
        .flat_map(|entry| match entry {
            Ok(path) => extract_from_path(path),
            Err(e) => {
                println!("{:?}", e);
                panic!("blah");
            }
        })
        .collect();
    x
}

fn extract_from_path(path: PathBuf) -> Vec<Reference> {
    // TODO: This can be a debug statement instead of a print
    // println!("Now parsing {:?}", path);
    let contents = fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read contents of {}", path.to_string_lossy()));

    extract_from_contents(contents)
}

fn extract_from_contents(contents: String) -> Vec<Reference> {
    let options = ParserOptions {
        buffer_name: "".to_string(),
        ..Default::default()
    };
    let parser = Parser::new(contents, options);
    let _ret = parser.do_parse();
    let ast = *_ret.ast.expect("No AST found!");

    dbg!(ast.clone());
    let mut collector = ReferenceCollector {
        references: vec![],
        current_namespaces: vec![],
        state: VisitorState::Initial,
    };
    // extract_from_ast(ast, vec![])
    collector.visit(&ast);
    collector.references
}

// fn extract_from_ast(ast: Node, current_module_nesting: Vec<String>) -> Vec<Reference> {
//     match ast {
//         Node::Class(class) => {
//             let body = *class.body.expect("no body on class node");
//             let class_name_node = *class.name;
//             walk_class_or_module_nodes(body, class_name_node, current_module_nesting)
//         }
//         Node::Const(n) => {
//             let fully_qualified_const_reference = unstack_constant_node(n);
//             // In this ruby file:
//             // class Foo
//             //   class Bar
//             //     Baz
//             //   end
//             // end
//             // "Foo" and "Bar" are in a local definition block, but Baz is not.
//             //
//             // In this ruby file:
//             // class Foo::Bar
//             //   Baz
//             // end
//             // "Foo" and "Foo::Bar" are in a local definition block, but Baz is not.
//             if false {
//                 vec![]
//             } else {
//                 vec![Reference {
//                     name: fully_qualified_const_reference,
//                     module_nesting: current_module_nesting,
//                 }]
//             }
//         }
//         Node::Module(module) => {
//             let body = *module.body.expect("no body on class node");
//             let class_name_node = *module.name;
//             walk_class_or_module_nodes(body, class_name_node, current_module_nesting)
//         }
//         Node::Array(arr) => arr
//             .elements
//             .into_iter()
//             .flat_map(|n| extract_from_ast(n, current_module_nesting.clone()))
//             .collect(),

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial_case() {
        let contents: String = String::from("Foo");
        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Foo"),
                module_nesting: vec![]
            }]
        );
    }

    #[test]
    fn test_nested_constant() {
        let contents: String = String::from("Foo::Bar");
        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Foo::Bar"),
                module_nesting: vec![]
            }]
        );
    }

    #[test]
    fn test_deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz");
        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Foo::Bar::Baz"),
                module_nesting: vec![]
            }]
        );
    }

    #[test]
    fn test_very_deeply_nested_constant() {
        let contents: String = String::from("Foo::Bar::Baz::Boo");
        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Foo::Bar::Baz::Boo"),
                module_nesting: vec![]
            }]
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

        assert_eq!(extract_from_contents(contents), vec![]);
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Bar"),
                module_nesting: vec![String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Baz"),
                module_nesting: vec![String::from("Foo::Bar"), String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Boo"),
                module_nesting: vec![String::from("Foo::Bar::Baz"), String::from("Foo::Bar"), String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Bar"),
                module_nesting: vec![String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Baz"),
                module_nesting: vec![String::from("Foo::Bar"), String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Boo"),
                module_nesting: vec![String::from("Foo::Bar::Baz"), String::from("Foo::Bar"), String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Boo"),
                module_nesting: vec![String::from("Foo::Bar::Baz"), String::from("Foo::Bar"), String::from("Foo")]
            }]
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
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Baz"),
                module_nesting: vec![String::from("Foo::Bar")],
            }]
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_compact_style_with_nesting_class_definition_constant() {
        let contents: String = String::from(
            "\
            class Foo::Bar
                module Baz
                    Baz
                end
            end
        ",
        );

        assert_eq!(
            extract_from_contents(contents),
            vec![Reference {
                name: String::from("Baz"),
                module_nesting: vec![String::from("Foo::Bar::Baz"), String::from("Foo::Bar")],
            }]
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_constant() {
        let contents: String = String::from("[Foo]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references.get(0).expect("There should be a reference at index 0");
        assert_eq!(
            *reference,
            Reference {
                name: String::from("Foo"),
                module_nesting: vec![]
            }
        );
    }
    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_multiple_constants() {
        let contents: String = String::from("[Foo, Bar]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 2);
        let reference1 = references.get(0).expect("There should be a reference at index 0");
        assert_eq!(
            *reference1,
            Reference {
                name: String::from("Foo"),
                module_nesting: vec![]
            }
        );
        let reference2 = references.get(1).expect("There should be a reference at index 1");
        assert_eq!(
            *reference2,
            Reference {
                name: String::from("Bar"),
                module_nesting: vec![]
            }
        );
    }

    #[test]
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Style/ClassAndModuleChildren
    fn test_array_of_nested_constant() {
        let contents: String = String::from("[Baz::Boo]");
        let references = extract_from_contents(contents);
        assert_eq!(references.len(), 1);
        let reference = references.get(0).expect("There should be a reference at index 0");
        assert_eq!(
            *reference,
            Reference {
                name: String::from("Baz::Boo"),
                module_nesting: vec![]
            }
        );
    }
}
