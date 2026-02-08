use std::collections::HashSet;

use lib_ruby_parser::{nodes, Loc, Node};
use line_col::LineColLookup;

use crate::packs::{
    parsing::{ParsedDefinition, Range, UnresolvedReference},
    Sigil,
};

use super::inflector_shim::to_class_case;

#[derive(Debug)]
pub enum ParseError {
    Metaprogramming,
    // Add more variants as needed for different error cases
}

pub fn fetch_node_location(node: &nodes::Node) -> Result<&Loc, ParseError> {
    match node {
        Node::Const(const_node) => Ok(&const_node.expression_l),
        node => {
            panic!(
                "Cannot handle other node in get_constant_node_name: {:?}",
                node
            )
        }
    }
}

pub fn get_definition_from(
    current_nesting: &String,
    parent_nesting: &[String],
    location: &Range,
) -> ParsedDefinition {
    let name = current_nesting.to_owned();

    let owned_namespace_path: Vec<String> = parent_nesting.to_vec();

    let fully_qualified_name = if !owned_namespace_path.is_empty() {
        let mut name_components = owned_namespace_path;
        name_components.push(name);
        format!("::{}", name_components.join("::"))
    } else {
        format!("::{}", name)
    };

    ParsedDefinition {
        fully_qualified_name,
        location: location.to_owned(),
    }
}

pub fn loc_to_range(loc: &Loc, lookup: &LineColLookup) -> Range {
    let (start_row, start_col) = lookup.get(loc.begin); // There's an off-by-one difference here with packwerk
    let (end_row, end_col) = lookup.get(loc.end);

    Range {
        start_row,
        start_col: start_col - 1,
        end_row,
        end_col,
    }
}

pub fn fetch_const_name(node: &nodes::Node) -> Result<String, ParseError> {
    match node {
        Node::Const(const_node) => Ok(fetch_const_const_name(const_node)?),
        Node::Cbase(_) => Ok(String::from("")),
        Node::Send(_) => Err(ParseError::Metaprogramming),
        Node::Lvar(_) => Err(ParseError::Metaprogramming),
        Node::Ivar(_) => Err(ParseError::Metaprogramming),
        Node::Self_(_) => Err(ParseError::Metaprogramming),
        _node => Err(ParseError::Metaprogramming),
    }
}

pub fn fetch_const_const_name(
    node: &nodes::Const,
) -> Result<String, ParseError> {
    match &node.scope {
        Some(s) => {
            let parent_namespace = fetch_const_name(s)?;
            Ok(format!("{}::{}", parent_namespace, node.name))
        }
        None => Ok(node.name.to_owned()),
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

const ASSOCIATION_METHOD_NAMES: [&str; 4] = [
    "has_one",
    "has_many",
    "belongs_to",
    "has_and_belongs_to_many",
];

pub fn get_reference_from_active_record_association(
    node: &nodes::Send,
    current_namespaces: &[String],
    line_col_lookup: &LineColLookup,
    custom_associations: &[String],
) -> Option<UnresolvedReference> {
    // TODO: Read in args, process associations as a separate class
    // These can get complicated! e.g. we can specify a class name
    let combined_associations: Vec<String> = custom_associations
        .iter()
        .map(|s| s.to_owned())
        .chain(ASSOCIATION_METHOD_NAMES.iter().copied().map(String::from))
        .collect();

    let is_association = combined_associations
        .iter()
        .any(|association_method| node.method_name == *association_method);

    if is_association {
        let first_arg: Option<&Node> = node.args.first();

        let mut name: Option<String> = None;
        for node in node.args.iter() {
            if let Node::Kwargs(kwargs) = node {
                if let Some(found) = extract_class_name_from_kwargs(kwargs) {
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
                panic!("Could not find class name for association {:?}", &node,)
            });

            Some(UnresolvedReference {
                name: unwrapped_name,
                namespace_path: current_namespaces.to_owned(),
                location: loc_to_range(&node.expression_l, line_col_lookup),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn extract_class_name_from_kwargs(kwargs: &nodes::Kwargs) -> Option<String> {
    for pair_node in kwargs.pairs.iter() {
        if let Node::Pair(pair) = pair_node {
            if let Node::Sym(k) = *pair.key.to_owned() {
                if k.name.to_string_lossy() == *"class_name" {
                    // Handle string literal: class_name: "Foo::Bar"
                    if let Node::Str(v) = *pair.value.to_owned() {
                        return Some(v.value.to_string_lossy());
                    }
                    // Handle constant with .name: class_name: Foo::Bar.name
                    if let Node::Send(send) = *pair.value.to_owned() {
                        if send.method_name == "name" {
                            if let Some(recv) = send.recv {
                                if let Ok(const_name) = fetch_const_name(&recv)
                                {
                                    return Some(const_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn get_constant_assignment_definition(
    node: &nodes::Casgn,
    current_namespaces: Vec<String>,
    line_col_lookup: &LineColLookup,
) -> Option<ParsedDefinition> {
    let name_result = fetch_casgn_name(node);
    if name_result.is_err() {
        return None;
    }

    let name = name_result.unwrap();
    let fully_qualified_name = if !current_namespaces.is_empty() {
        let mut name_components = current_namespaces;
        name_components.push(name);
        format!("::{}", name_components.join("::"))
    } else {
        format!("::{}", name)
    };

    Some(ParsedDefinition {
        fully_qualified_name,
        location: loc_to_range(&node.expression_l, line_col_lookup),
    })
}

pub fn extract_sigils_from_contents(contents: &str) -> Vec<Sigil> {
    let mut sigils: Vec<Sigil> = Vec::new();

    // Hardcoded to public, but later we can make this a convention like `pack_*: true`, if we find it more generally useful
    contents.lines().take(5).for_each(|line| {
        if line.contains("pack_public: true") {
            sigils.push(Sigil {
                name: "public".to_string(),
                value: true,
            });
        }
    });

    sigils
}
