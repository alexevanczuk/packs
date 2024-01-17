use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConstantDefinition {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
}

#[derive(Debug)]
pub struct ConstantResolverConfiguration<'a> {
    pub absolute_root: &'a PathBuf,
    pub cache_directory: &'a PathBuf,
    pub cache_enabled: bool,
    pub autoload_roots: &'a HashMap<PathBuf, String>,
}

pub trait ConstantResolver {
    fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[&str],
    ) -> Option<Vec<ConstantDefinition>>;

    fn fully_qualified_constant_name_to_constant_definition_map(
        &self,
    ) -> &HashMap<String, Vec<ConstantDefinition>>;
}
