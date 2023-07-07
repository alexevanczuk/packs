use lib_ruby_parser::{nodes, Loc, Node};

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
