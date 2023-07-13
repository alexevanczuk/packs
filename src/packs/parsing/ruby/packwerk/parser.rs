use crate::packs::{
    parsing::{
        ruby::{
            namespace_calculator::possible_fully_qualified_constants,
            parse_utils::{
                fetch_const_const_name, fetch_const_name, fetch_node_location,
                get_constant_assignment_definition, get_definition_from,
                get_reference_from_active_record_association, loc_to_range,
            },
        },
        ParsedDefinition, Range, UnresolvedReference,
    },
    Configuration, ProcessedFile,
};
use lib_ruby_parser::{
    nodes, traverse::visitor::Visitor, Node, Parser, ParserOptions,
};
use line_col::LineColLookup;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SuperclassReference {
    pub name: String,
    pub namespace_path: Vec<String>,
}

struct ReferenceCollector<'a> {
    pub references: Vec<UnresolvedReference>,
    pub definitions: Vec<ParsedDefinition>,
    pub current_namespaces: Vec<String>,
    pub line_col_lookup: LineColLookup<'a>,
    pub in_superclass: bool,
    pub superclasses: Vec<SuperclassReference>,
    pub custom_associations: Vec<String>,
}

impl<'a> Visitor for ReferenceCollector<'a> {
    fn on_class(&mut self, node: &nodes::Class) {
        // We're not collecting definitions, so no need to visit the class definitioname);
        let namespace_result = fetch_const_name(&node.name);
        // For now, we simply exit and stop traversing if we encounter an error when fetching the constant name of a class
        // We can iterate on this if this is different than the packwerk implementation
        if namespace_result.is_err() {
            return;
        }

        let namespace = namespace_result.unwrap();

        if let Some(inner) = node.superclass.as_ref() {
            self.in_superclass = true;
            self.visit(inner);
            self.in_superclass = false;
        }
        let definition_loc = fetch_node_location(&node.name).unwrap();
        let location = loc_to_range(definition_loc, &self.line_col_lookup);

        let definition = get_definition_from(
            &namespace,
            &self.current_namespaces,
            &location,
        );

        let name = definition.fully_qualified_name.to_owned();
        let namespace_path = self.current_namespaces.to_owned();
        self.definitions.push(definition);

        // Packwerk also considers a definition to be a "reference"
        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location,
        });

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        if let Some(inner) = &node.body {
            self.visit(inner);
        }

        self.current_namespaces.pop();
        self.superclasses.pop();
    }

    fn on_send(&mut self, node: &nodes::Send) {
        let association_reference =
            get_reference_from_active_record_association(
                node,
                &self.current_namespaces,
                &self.line_col_lookup,
                &self.custom_associations,
            );

        if let Some(association_reference) = association_reference {
            self.references.push(association_reference);
        }

        lib_ruby_parser::traverse::visitor::visit_send(self, node);
    }

    fn on_casgn(&mut self, node: &nodes::Casgn) {
        let definition = get_constant_assignment_definition(
            node,
            self.current_namespaces.to_owned(),
            &self.line_col_lookup,
        );

        if let Some(definition) = definition {
            self.definitions.push(definition);
        }

        if let Some(v) = node.value.to_owned() {
            self.visit(&v);
        } else {
            // We don't handle constant assignments as part of a multi-assignment yet,
            // e.g. A, B = 1, 2
            // See the documentation for nodes::Casgn#value for more info.
        }
    }

    fn on_module(&mut self, node: &nodes::Module) {
        let namespace = fetch_const_name(&node.name)
            .expect("We expect no parse errors in class/module definitions");
        let definition_loc = fetch_node_location(&node.name).unwrap();
        let location = loc_to_range(definition_loc, &self.line_col_lookup);

        let definition = get_definition_from(
            &namespace,
            &self.current_namespaces,
            &location,
        );

        let name = definition.fully_qualified_name.to_owned();
        let namespace_path = self.current_namespaces.to_owned();
        self.definitions.push(definition);

        // Packwerk also considers a definition to be a "reference"
        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location,
        });

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        if let Some(inner) = &node.body {
            self.visit(inner);
        }

        self.current_namespaces.pop();
    }

    fn on_const(&mut self, node: &nodes::Const) {
        let Ok(name) = fetch_const_const_name(node) else { return };

        if self.in_superclass {
            self.superclasses.push(SuperclassReference {
                name: name.to_owned(),
                namespace_path: self.current_namespaces.to_owned(),
            })
        }
        // In packwerk, NodeHelpers.enclosing_namespace_path ignores
        // namespaces where a superclass OR namespace is the same as the current reference name
        let matching_superclass_option = self
            .superclasses
            .iter()
            .find(|superclass| superclass.name == name);

        let namespace_path =
            if let Some(matching_superclass) = matching_superclass_option {
                matching_superclass.namespace_path.to_owned()
            } else {
                self.current_namespaces
                    .clone()
                    .into_iter()
                    .filter(|namespace| {
                        namespace != &name
                            || self
                                .superclasses
                                .iter()
                                .any(|superclass| superclass.name == name)
                    })
                    .collect::<Vec<String>>()
            };

        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location: loc_to_range(&node.expression_l, &self.line_col_lookup),
        })
    }
}

pub(crate) fn process_from_path(
    path: &Path,
    configuration: &Configuration,
) -> ProcessedFile {
    let contents = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("Failed to read contents of {}", path.to_string_lossy())
    });

    process_from_contents(contents, path, configuration)
}

pub(crate) fn process_from_contents(
    contents: String,
    path: &Path,
    configuration: &Configuration,
) -> ProcessedFile {
    let options = ParserOptions {
        buffer_name: "".to_string(),
        ..Default::default()
    };

    let lookup = LineColLookup::new(&contents);
    let parser = Parser::new(contents.clone(), options);
    let parse_result = parser.do_parse();

    let ast_option: Option<Box<Node>> = parse_result.ast;

    let ast = match ast_option {
        Some(some_ast) => some_ast,
        None => {
            return ProcessedFile {
                absolute_path: path.to_owned(),
                unresolved_references: vec![],
                definitions: vec![],
            }
        }
    };

    let mut collector = ReferenceCollector {
        references: vec![],
        current_namespaces: vec![],
        definitions: vec![],
        line_col_lookup: lookup,
        in_superclass: false,
        superclasses: vec![],
        custom_associations: configuration.custom_associations.clone(),
    };

    collector.visit(&ast);

    let mut definition_to_location_map: HashMap<String, Range> = HashMap::new();

    for d in &collector.definitions {
        let parts: Vec<&str> = d.fully_qualified_name.split("::").collect();
        // We do this to handle nested constants, e.g.
        // class Foo::Bar
        // end
        for (index, _) in parts.iter().enumerate() {
            let combined = &parts[..=index].join("::");
            // If the map already contains the key, skip it.
            // This is helpful, e.g.
            // class Foo::Bar
            //  BAZ
            // end
            // The fully name for BAZ IS ::Foo::Bar::BAZ, so we do not want to overwrite
            // the definition location for ::Foo or ::Foo::Bar
            if !definition_to_location_map.contains_key(combined) {
                definition_to_location_map
                    .insert(combined.to_owned(), d.location.clone());
            }
        }
    }

    let unresolved_references = collector
        .references
        .into_iter()
        .filter(|r| {
            let mut should_ignore_local_reference = false;
            let namespace_path = r
                .namespace_path
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>();
            let possible_constants =
                possible_fully_qualified_constants(&namespace_path, &r.name);
            for constant_name in possible_constants {
                if let Some(location) = definition_to_location_map
                    .get(&constant_name)
                    .or(definition_to_location_map
                        .get(&format!("::{}", constant_name)))
                {
                    let reference_is_definition = location.start_row
                        == r.location.start_row
                        && location.start_col == r.location.start_col;
                    // In lib/packwerk/parsed_constant_definitions.rb, we don't count references when the reference is in the same place as the definition
                    // This is an idiosyncracy we are porting over here for behavioral alignment, although we might be doing some unnecessary work.
                    should_ignore_local_reference = !reference_is_definition;
                }
            }
            !should_ignore_local_reference
        })
        .collect();

    let absolute_path = path.to_owned();

    // The packwerk parser uses a ConstantResolver constructed by constants inferred from the file system
    // see zeitwerk_utils for more.
    // For a parser that uses parsed constants, see the experimental parser
    let definitions = vec![];

    ProcessedFile {
        absolute_path,
        unresolved_references,
        definitions,
    }
}
