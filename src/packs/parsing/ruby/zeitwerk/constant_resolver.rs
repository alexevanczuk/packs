use tracing::debug;

use std::collections::HashMap;

use crate::packs::{
    constant_resolver::{ConstantDefinition, ConstantResolver},
    parsing::ruby::namespace_calculator::combine_namespace_with_constant_name,
};

#[derive(Default, Debug)]
pub struct ZeitwerkConstantResolver {
    pub fully_qualified_constant_name_to_constant_definition_map:
        HashMap<String, Vec<ConstantDefinition>>,
}

impl ConstantResolver for ZeitwerkConstantResolver {
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

        self.resolve_constant(const_name, namespace_path, const_name)
    }

    fn fully_qualified_constant_name_to_constant_definition_map(
        &self,
    ) -> &HashMap<String, Vec<ConstantDefinition>> {
        &self.fully_qualified_constant_name_to_constant_definition_map
    }
}

impl ZeitwerkConstantResolver {
    pub fn create(
        constants: Vec<ConstantDefinition>,
    ) -> Box<dyn ConstantResolver + Send + Sync> {
        debug!("Building constant resolver from constants vector");

        let mut fully_qualified_constant_to_constant_map: HashMap<
            String,
            Vec<ConstantDefinition>,
        > = HashMap::new();

        // TODO: Do this in parallel?
        for constant in constants {
            let fully_qualified_constant_name =
                constant.fully_qualified_name.clone();

            let existing_constant = fully_qualified_constant_to_constant_map
                .get(&fully_qualified_constant_name);

            if let Some(existing_constant) = existing_constant {
                // TODO: This still needs to be handled more elegantly. For now, we just panic.
                // Probably, we should have the HashMap have a Vec<Constant> instead of a single Constant, and then we can add to the Vec.
                // Then, when we create references, we can create one reference to each unique pack that defines the constant.

                // Later, we can allow the checkers to skip over constants where it's pointing at a pack that defines it as an ignored_monkeypatch: path/to/definition.rb
                // We should be sure to validate that ignored_monkeypatch paths match the absolute_path_to_definition of the constant.
                panic!(
                    "Found two constants with the same name: {:?} and {:?}",
                    existing_constant, constant
                );
            } else {
                fully_qualified_constant_to_constant_map
                    .insert(fully_qualified_constant_name, vec![constant]);
            }
        }

        debug!("Finished building constant resolver");

        Box::new(ZeitwerkConstantResolver {
            fully_qualified_constant_name_to_constant_definition_map:
                fully_qualified_constant_to_constant_map,
        })
    }

    fn resolve_constant<'a>(
        &'a self,
        const_name: &'a str,
        current_namespace_path: &'a [&str],
        original_name: &'a str,
    ) -> Option<Vec<ConstantDefinition>> {
        let constant = self.resolve_traversing_namespace_path(
            const_name,
            current_namespace_path,
            original_name,
        );
        match constant {
            Some(definition) => Some(vec![definition]),
            None => {
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
        }
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
        original_name: &'a str,
    ) -> Option<ConstantDefinition> {
        let fully_qualified_name_guess = combine_namespace_with_constant_name(
            current_namespace_path,
            const_name,
        );

        if let Some(constant) =
            self.constant_for_fully_qualified_name(&fully_qualified_name_guess)
        {
            // Since the ContantResolver might say that some constant Foo::Bar::Baz is defined in Foo::Bar,
            // we want to return a ConstantDefinition that has the fully qualified name of the constant we're looking for.
            // In this case, we want to return a ConstantDefinition with the fully qualified name of Foo::Bar::Baz
            // even though the ConstantDefinition we found has the fully qualified name of Foo::Bar
            // The ConstantResolver from the experimental parser does not need to do this, so we might be better off
            // having a separate ConstantResolver for that implementation
            let fully_qualified_name = combine_namespace_with_constant_name(
                current_namespace_path,
                original_name,
            );

            let absolute_path_of_definition =
                constant.absolute_path_of_definition.to_owned();
            Some(ConstantDefinition {
                fully_qualified_name,
                absolute_path_of_definition,
            })
        } else {
            // In this case, we couldn't find a constant with the given name under the given namespace.
            // However, it's possible the constant is defined within the parent namespace.
            let split_result = current_namespace_path.split_last();
            match split_result {
                Some((_last, parent_namespace)) => self
                    .resolve_traversing_namespace_path(
                        const_name,
                        parent_namespace,
                        original_name,
                    ),
                None => None,
            }
        }
    }

    fn constant_for_fully_qualified_name(
        &self,
        fully_qualified_name: &String,
    ) -> Option<&ConstantDefinition> {
        if let Some(definitions) = self
            .fully_qualified_constant_name_to_constant_definition_map
            .get(fully_qualified_name)
        {
            return Some(definitions.first().unwrap());
        }

        None
    }
}
