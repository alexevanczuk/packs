use tracing::debug;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::packs::{
    constant_resolver::{ConstantDefinition, ConstantResolver},
    parsing::ruby::namespace_calculator::combine_namespace_with_constant_name,
};

#[derive(Default, Debug)]
pub struct ExperimentalConstantResolver {
    pub fully_qualified_constant_name_to_constant_definition_map:
        HashMap<String, Vec<ConstantDefinition>>,
}

impl ConstantResolver for ExperimentalConstantResolver {
    fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[&str],
    ) -> Option<Vec<ConstantDefinition>> {
        // If the fully_or_partially_qualified_constant is prefixed with ::, the namespace path is technically empty, since it's a global reference
        let (namespace_path, const_name) =
            if fully_or_partially_qualified_constant.starts_with("::") {
                // `resolve_constant` will add a leading :: before it makes a guess at the fully qualified name
                // so we remove it here and represent it as a relative constant with no namespace path
                let const_name = fully_or_partially_qualified_constant
                    .strip_prefix("::")
                    .unwrap();
                let namespace_path: &[&str] = &[];
                (namespace_path, const_name)
            } else {
                (namespace_path, fully_or_partially_qualified_constant)
            };

        Some(self.resolve_traversing_namespace_path(const_name, namespace_path))
    }

    fn fully_qualified_constant_name_to_constant_definition_map(
        &self,
    ) -> &HashMap<String, Vec<ConstantDefinition>> {
        &self.fully_qualified_constant_name_to_constant_definition_map
    }
}

impl ExperimentalConstantResolver {
    pub fn create(
        constants: Vec<ConstantDefinition>,
        absolute_root: &Path,
        ignored_definitions: &HashMap<String, HashSet<PathBuf>>,
    ) -> Box<dyn ConstantResolver + Send + Sync> {
        debug!("Building constant resolver from constants vector");

        let mut fully_qualified_constant_to_constant_map: HashMap<
            String,
            Vec<ConstantDefinition>,
        > = HashMap::new();

        for constant in constants {
            if let Some(definition_location) =
                ignored_definitions.get(&constant.fully_qualified_name)
            {
                let relative_path = constant
                    .absolute_path_of_definition
                    .strip_prefix(absolute_root)
                    .unwrap();

                if definition_location.contains(relative_path) {
                    debug!(
                        "Ignoring definition of {:?} from {:?}",
                        constant.fully_qualified_name,
                        constant.absolute_path_of_definition
                    );
                    continue;
                }
            }

            let fully_qualified_constant_name =
                constant.fully_qualified_name.clone();

            let existing_definitions = fully_qualified_constant_to_constant_map
                .get(&fully_qualified_constant_name);

            if let Some(existing_definitions) = existing_definitions {
                let mut new_definitions = existing_definitions.clone();
                new_definitions.push(constant);
                fully_qualified_constant_to_constant_map
                    .insert(fully_qualified_constant_name, new_definitions);
            } else {
                fully_qualified_constant_to_constant_map
                    .insert(fully_qualified_constant_name, vec![constant]);
            }
        }

        debug!("Finished building constant resolver");

        Box::new(ExperimentalConstantResolver {
            fully_qualified_constant_name_to_constant_definition_map:
                fully_qualified_constant_to_constant_map,
        })
    }

    // In Ruby, say we have this code:
    //
    // module Foo
    //   module Bar
    //     module Baz
    //       Boo
    //     end
    //   end
    // end
    //
    // The `current_namespace_path` here is: ['Foo', 'Bar', 'Baz']
    // The `const_name` here is: `Boo`
    // Ruby constant resolution rules dictate that `Boo` coudl refer to any of the following,
    // in this specific order:
    //
    // ::Foo::Bar::Baz::Boo
    // ::Foo::Bar::Boo
    // ::Foo::Boo
    // ::Boo
    //
    // We need to check each of these possibilities in order, and return the first one that exists
    // If none of them exist, return None
    fn resolve_traversing_namespace_path<'a>(
        &'a self,
        const_name: &'a str,
        current_namespace_path: &'a [&str],
    ) -> Vec<ConstantDefinition> {
        let fully_qualified_name_guess = combine_namespace_with_constant_name(
            current_namespace_path,
            const_name,
        );

        let definitions =
            self.constant_for_fully_qualified_name(&fully_qualified_name_guess);

        if !definitions.is_empty() {
            definitions
        } else {
            // In this case, we couldn't find a constant with the given name under the given namespace.
            // However, it's possible the constant is defined within the parent namespace.
            let split_result = current_namespace_path.split_last();
            match split_result {
                Some((_last, parent_namespace)) => self
                    .resolve_traversing_namespace_path(
                        const_name,
                        parent_namespace,
                    ),
                None => vec![],
            }
        }
    }

    fn constant_for_fully_qualified_name(
        &self,
        fully_qualified_name: &String,
    ) -> Vec<ConstantDefinition> {
        let ret = self
            .fully_qualified_constant_name_to_constant_definition_map
            .get(fully_qualified_name);

        match ret {
            Some(constant_definitions) => constant_definitions.to_owned(),
            None => vec![],
        }
    }
}
