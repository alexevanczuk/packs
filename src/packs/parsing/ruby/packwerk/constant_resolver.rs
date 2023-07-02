use serde::{Deserialize, Serialize};
use tracing::debug;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct ConstantResolver {
    pub fully_qualified_constant_to_constant_map: HashMap<String, Constant>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub fully_qualified_name: String,
    pub absolute_path_of_definition: PathBuf,
}

#[allow(unused_variables)]
impl ConstantResolver {
    pub fn create(
        absolute_root: &Path,
        constants: Vec<Constant>,
    ) -> ConstantResolver {
        debug!("Building constant resolver");

        debug!("Building constant resolver from constants vector");

        let mut fully_qualified_constant_to_constant_map: HashMap<
            String,
            Constant,
        > = HashMap::new();

        // TODO: Do this in parallel?
        for constant in constants {
            let fully_qualified_constant_name =
                constant.fully_qualified_name.clone();

            let existing_constant = fully_qualified_constant_to_constant_map
                .get(&fully_qualified_constant_name);

            if let Some(existing_constant) = existing_constant {
                panic!(
                    "Found two constants with the same name: {:?} and {:?}",
                    existing_constant, constant
                );
            } else {
                fully_qualified_constant_to_constant_map
                    .insert(fully_qualified_constant_name, constant);
            }
        }

        debug!("Finished building constant resolver");

        ConstantResolver {
            fully_qualified_constant_to_constant_map,
        }
    }

    pub fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[&str],
    ) -> Option<Constant> {
        // If the fully_or_partially_qualified_constant is prefixed with ::, the namespace path is technically empty, since it's a global reference
        let (namespace_path, const_name) =
            if fully_or_partially_qualified_constant.starts_with("::") {
                let const_name = fully_or_partially_qualified_constant
                    .trim_start_matches("::");
                let namespace_path: &[&str] = &[];
                (namespace_path, const_name)
            } else {
                (namespace_path, fully_or_partially_qualified_constant)
            };

        self.resolve_constant(const_name, namespace_path, const_name)
    }
    fn resolve_constant<'a>(
        &'a self,
        const_name: &'a str,
        current_namespace_path: &'a [&str],
        original_name: &'a str,
    ) -> Option<Constant> {
        let constant = self.resolve_traversing_namespace_path(
            const_name,
            current_namespace_path,
        );
        match constant {
            (Some(namespace), Some(absolute_path_of_definition)) => {
                let mut fully_qualified_name_vec = vec![""];
                fully_qualified_name_vec.extend(namespace);
                fully_qualified_name_vec.push(original_name);
                let fully_qualified_name_guess =
                    fully_qualified_name_vec.join("::");

                Some(Constant {
                    fully_qualified_name: fully_qualified_name_guess,
                    absolute_path_of_definition: absolute_path_of_definition
                        .to_owned(),
                })
            }
            (None, None) => {
                // If we couldn't find a match, it's possible the constant is defined within its parent namespace and not within its own file.
                // For example, `Boo` above could be defined in `foo/bar.rb` as:
                // module Foo
                //   module Bar
                //     class Boo
                //     end
                //   end
                // end
                // Therefore, we take the given const_name, remove the last part of the fully qualified name, and try again.
                // In this case, we'd try to resolve `::Foo::Bar` instead of `::Foo::Bar::Boo`
                let split_const = const_name.split("::").collect::<Vec<&str>>();
                if split_const.len() <= 1 {
                    return None;
                }
                let parent_constant =
                    split_const[0..=split_const.len() - 2].join("::");
                self.resolve_constant(
                    &parent_constant,
                    current_namespace_path,
                    original_name,
                )
            }
            _ => {
                todo!()
            }
        }
    }

    // Example for namespace_path: ['Foo', 'Bar', 'Baz']
    // If the const_name is 'Boo',
    // it could refer to any of the following:
    // ::Foo::Bar::Baz::Boo
    // ::Foo::Bar::Boo
    // ::Foo::Boo
    // ::Boo
    // We need to check each of these possibilities in order, and return the first one that exists
    // If none of them exist, return None
    fn resolve_traversing_namespace_path<'a>(
        &'a self,
        const_name: &'a str,
        current_namespace_path: &'a [&str],
    ) -> (Option<&'a [&str]>, Option<&'a PathBuf>) {
        let mut fully_qualified_name_guess_vec =
            current_namespace_path.to_vec();
        fully_qualified_name_guess_vec.push(const_name);

        let fully_qualified_name_guess =
            fully_qualified_name_guess_vec.join("::");

        if let Some(constant) =
            self.constant_for_fully_qualified_name(&fully_qualified_name_guess)
        {
            (
                Some(current_namespace_path),
                Some(&constant.absolute_path_of_definition),
            )
        } else {
            // In this case, we couldn't find a constant with the given name under the given namespace.
            // However, it's possible the constant is defined within the parent namespace.
            let split_result = current_namespace_path.split_last();
            match split_result {
                Some((_last, parent_namespace)) => {
                    let vec = parent_namespace;
                    let (namespace, absolute_path_of_definition) =
                        self.resolve_traversing_namespace_path(const_name, vec);
                    (namespace, absolute_path_of_definition)
                }
                None => (None, None),
            }
        }
    }

    fn constant_for_fully_qualified_name(
        &self,
        fully_qualified_name: &String,
    ) -> Option<&Constant> {
        if let Some(constant) = self
            .fully_qualified_constant_to_constant_map
            .get(fully_qualified_name)
        {
            return Some(constant);
        }

        None
    }
}
