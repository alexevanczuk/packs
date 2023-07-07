use lib_ruby_parser::{nodes, Loc, Node};
use line_col::LineColLookup;

use crate::packs::parsing::Range;

#[derive(Debug)]
pub enum ParseError {
    Metaprogramming,
    // Add more variants as needed for different error cases
}

pub fn fetch_node_location(node: &nodes::Node) -> Result<&Loc, ParseError> {
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
