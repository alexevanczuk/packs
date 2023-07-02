use crate::packs::{
    inflector_shim::to_class_case,
    parsing::{Definition, Range, UnresolvedReference},
    ProcessedFile,
};
use lib_ruby_parser::{
    nodes, traverse::visitor::Visitor, Loc, Node, Parser, ParserOptions,
};
use line_col::LineColLookup;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use crate::packs::parsing::ruby::namespace_calculator;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SuperclassReference {
    pub name: String,
    pub namespace_path: Vec<String>,
}

impl UnresolvedReference {
    fn possible_fully_qualified_constants(&self) -> Vec<String> {
        if self.name.starts_with("::") {
            return vec![self.name.to_owned()];
        }

        let mut possible_constants = vec![self.name.to_owned()];
        let module_nesting = namespace_calculator::calculate_module_nesting(
            &self.namespace_path,
        );
        for nesting in module_nesting {
            let possible_constant = format!("::{}::{}", nesting, self.name);
            possible_constants.push(possible_constant);
        }

        possible_constants
    }
}

struct ReferenceCollector<'a> {
    pub references: Vec<UnresolvedReference>,
    pub definitions: Vec<Definition>,
    pub current_namespaces: Vec<String>,
    pub line_col_lookup: LineColLookup<'a>,
    pub in_superclass: bool,
    pub superclasses: Vec<SuperclassReference>,
}

#[derive(Debug)]
enum ParseError {
    Metaprogramming,
    // Add more variants as needed for different error cases
}

fn fetch_node_location(node: &nodes::Node) -> Result<&Loc, ParseError> {
    match node {
        Node::Const(const_node) => Ok(&const_node.expression_l),
        node => {
            dbg!(node);
            panic!(
                "Cannot handle other node in get_constant_node_name: {:?}",
                node
            )
        }
    }
}

fn loc_to_range(loc: &Loc, lookup: &LineColLookup) -> Range {
    let (start_row, start_col) = lookup.get(loc.begin); // There's an off-by-one difference here with packwerk
    let (end_row, end_col) = lookup.get(loc.end);

    Range {
        start_row,
        start_col: start_col - 1,
        end_row,
        end_col,
    }
}
fn fetch_const_name(node: &nodes::Node) -> Result<String, ParseError> {
    match node {
        Node::Const(const_node) => Ok(fetch_const_const_name(const_node)?),
        Node::Cbase(_) => Ok(String::from("")),
        Node::Send(_) => Err(ParseError::Metaprogramming),
        Node::Lvar(_) => Err(ParseError::Metaprogramming),
        Node::Ivar(_) => Err(ParseError::Metaprogramming),
        Node::Self_(_) => Err(ParseError::Metaprogramming),
        node => {
            dbg!(node);
            panic!(
                "Cannot handle other node in get_constant_node_name: {:?}",
                node
            )
        }
    }
}

fn fetch_const_const_name(node: &nodes::Const) -> Result<String, ParseError> {
    match &node.scope {
        Some(s) => {
            let parent_namespace = fetch_const_name(s)?;
            Ok(format!("{}::{}", parent_namespace, node.name))
        }
        None => Ok(node.name.to_owned()),
    }
}

fn get_definition_from(
    current_nesting: &String,
    parent_nesting: &[String],
    location: &Range,
) -> Definition {
    let name = current_nesting.to_owned();

    let owned_namespace_path: Vec<String> = parent_nesting.to_vec();

    let fully_qualified_name = if !owned_namespace_path.is_empty() {
        let mut name_components = owned_namespace_path.clone();
        name_components.push(name);
        format!("::{}", name_components.join("::"))
    } else {
        format!("::{}", name)
    };

    Definition {
        fully_qualified_name,
        namespace_path: owned_namespace_path,
        location: location.to_owned(),
    }
}

// TODO: Combine with fetch_const_const_name
fn fetch_casgn_name(node: &nodes::Casgn) -> Result<String, ParseError> {
    match &node.scope {
        Some(s) => {
            let parent_namespace = fetch_const_name(s)?;
            Ok(format!("{}::{}", parent_namespace, node.name))
        }
        None => Ok(node.name.to_owned()),
    }
}

fn extract_class_name_from_kwargs(kwargs: &nodes::Kwargs) -> Option<String> {
    for pair_node in kwargs.pairs.iter() {
        if let Node::Pair(pair) = pair_node {
            if let Node::Sym(k) = *pair.key.to_owned() {
                if k.name.to_string_lossy() == *"class_name" {
                    if let Node::Str(v) = *pair.value.to_owned() {
                        return Some(v.value.to_string_lossy());
                    }
                }
            }
        }
    }

    None
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
            // dbg!("Visiting superclass!: {:?}", inner);
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

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        let name = definition.fully_qualified_name.to_owned();
        let namespace_path = definition.namespace_path.to_owned();
        self.definitions.push(definition);

        // Packwerk also considers a definition to be a "reference"
        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location,
        });

        if let Some(inner) = &node.body {
            self.visit(inner);
        }

        self.current_namespaces.pop();
        self.superclasses.pop();
    }

    fn on_send(&mut self, node: &nodes::Send) {
        // TODO: Read in args, process associations as a separate class
        // These can get complicated! e.g. we can specify a class name
        // dbg!(&node);
        if node.method_name == *"has_one"
            || node.method_name == *"has_many"
            || node.method_name == *"belongs_to"
            || node.method_name == *"has_and_belongs_to_many"
        {
            let first_arg: Option<&Node> = node.args.get(0);

            let mut name: Option<String> = None;
            for node in node.args.iter() {
                if let Node::Kwargs(kwargs) = node {
                    if let Some(found) = extract_class_name_from_kwargs(kwargs)
                    {
                        name = Some(found);
                    }
                }
            }

            if let Some(Node::Sym(d)) = first_arg {
                if name.is_none() {
                    // We singularize here because by convention Rails will singularize the class name as declared via a symbol,
                    // e.g. `has_many :companies` will look for a class named `Company`, not `Companies`
                    name = Some(to_class_case(
                        &d.name.to_string_lossy(),
                        true,
                        &HashSet::new(), // todo: pass in acronyms here
                    ));
                }
            }

            // let unwrapped_name = name.unwrap_or_else(|| {
            //     panic!("Could not find class name for association {:?}", &node,)
            // });
            // Later we should probably handle these cases!
            if name.is_some() {
                let unwrapped_name = name.unwrap_or_else(|| {
                    panic!(
                        "Could not find class name for association {:?}",
                        &node,
                    )
                });

                self.references.push(UnresolvedReference {
                    name: unwrapped_name,
                    namespace_path: self.current_namespaces.to_owned(),
                    location: loc_to_range(
                        &node.expression_l,
                        &self.line_col_lookup,
                    ),
                })
            }
        }

        lib_ruby_parser::traverse::visitor::visit_send(self, node);
    }

    fn on_casgn(&mut self, node: &nodes::Casgn) {
        let name_result = fetch_casgn_name(node);
        if name_result.is_err() {
            return;
        }

        // TODO: This can be extracted from on_class
        let name = name_result.unwrap();
        let fully_qualified_name = if !self.current_namespaces.is_empty() {
            let mut name_components = self.current_namespaces.clone();
            name_components.push(name);
            format!("::{}", name_components.join("::"))
        } else {
            format!("::{}", name)
        };

        self.definitions.push(Definition {
            fully_qualified_name,
            namespace_path: self.current_namespaces.to_owned(),
            location: loc_to_range(&node.expression_l, &self.line_col_lookup),
        });

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

        // Note – is there a way to use lifetime specifiers to get rid of this and
        // just keep current namespaces as a vector of string references or something else
        // more efficient?
        self.current_namespaces.push(namespace);

        let name = definition.fully_qualified_name.to_owned();
        let namespace_path = definition.namespace_path.to_owned();
        self.definitions.push(definition);

        // Packwerk also considers a definition to be a "reference"
        self.references.push(UnresolvedReference {
            name,
            namespace_path,
            location,
        });

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

pub(crate) fn process_from_path(path: &Path) -> ProcessedFile {
    let contents = fs::read_to_string(path).unwrap_or_else(|_| {
        panic!("Failed to read contents of {}", path.to_string_lossy())
    });

    process_from_contents(contents, path)
}

pub(crate) fn process_from_contents(
    contents: String,
    path: &Path,
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
            let possible_constants = r.possible_fully_qualified_constants();
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
                    if reference_is_definition {
                        should_ignore_local_reference = false
                    } else {
                        should_ignore_local_reference = true
                    }
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
