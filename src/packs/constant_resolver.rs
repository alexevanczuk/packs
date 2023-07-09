use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConstantDefinition {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
}

pub trait ConstantResolver {
    fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[&str],
    ) -> Option<ConstantDefinition>;
}
