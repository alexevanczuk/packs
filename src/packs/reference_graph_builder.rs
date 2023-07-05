use super::{
    checker::Reference,
    parsing::ruby::packwerk::constant_resolver::ConstantResolver,
};

pub struct ReferenceGraph {
    pub references: Vec<Reference>,
    pub constant_resolver: ConstantResolver,
}
