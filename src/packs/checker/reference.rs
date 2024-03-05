use std::path::Path;

use anyhow::{bail, Context};

use crate::packs::{
    constant_resolver::ConstantResolver, pack::Pack,
    parsing::UnresolvedReference, Configuration, PackSet, SourceLocation,
};

#[derive(Debug)]
pub struct Reference {
    pub constant_name: String,
    pub defining_pack_name: Option<String>,
    pub relative_defining_file: Option<String>,
    pub referencing_pack_name: String,
    pub relative_referencing_file: String,
    pub source_location: SourceLocation,
}

impl Reference {
    pub fn defining_pack<'a>(
        &self,
        pack_set: &'a PackSet,
    ) -> anyhow::Result<Option<&'a Pack>> {
        if let Some(name) = &self.defining_pack_name {
            Ok(Some(pack_set
                .for_pack(name)
                .context(format!(
                    "Reference#defining_pack_name is {}, but that pack is not found in pack set.",
                    &name
                ))?))
        } else {
            Ok(None)
        }
    }

    pub fn referencing_pack<'a>(
        &self,
        pack_set: &'a PackSet,
    ) -> anyhow::Result<&'a Pack> {
        pack_set.for_pack(&self.referencing_pack_name).
        context(format!("Reference#referencing_pack_name is {}, but that pack is not found in pack set.", 
        &self.referencing_pack_name))
    }
}

impl Reference {
    pub fn from_unresolved_reference(
        configuration: &Configuration,
        constant_resolver: &(dyn ConstantResolver + Send + Sync),
        unresolved_reference: &UnresolvedReference,
        referencing_file_path: &Path,
    ) -> anyhow::Result<Vec<Reference>> {
        let referencing_pack_name = match configuration
            .pack_set
            .for_file(referencing_file_path)?
            .map(|pack| pack.name.clone())
        {
            Some(pack_name) => pack_name,
            None => bail!(
                "Could not find pack for referencing file path: {}",
                &referencing_file_path.display()
            ),
        };

        let loc = &unresolved_reference.location;
        let source_location = SourceLocation {
            line: loc.start_row,
            column: loc.start_col,
        };

        let relative_referencing_file_path = referencing_file_path
            .strip_prefix(&configuration.absolute_root)
            .unwrap()
            .to_path_buf();

        let relative_referencing_file =
            relative_referencing_file_path.to_str().unwrap().to_string();

        let str_namespace_path: Vec<&str> = unresolved_reference
            .namespace_path
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        let maybe_constant_definition = constant_resolver
            .resolve(&unresolved_reference.name, &str_namespace_path);

        if let Some(constant_definitions) = &maybe_constant_definition {
            Ok(constant_definitions
                .iter()
                .map(move |constant| {
                    let absolute_path_of_definition =
                        &constant.absolute_path_of_definition;
                    let relative_defining_file = absolute_path_of_definition
                        .strip_prefix(&configuration.absolute_root)
                        .unwrap()
                        .to_path_buf()
                        .to_str()
                        .unwrap()
                        .to_string();

                    let defining_pack_name = configuration
                        .pack_set
                        .for_file(absolute_path_of_definition)?
                        .map(|pack| pack.name.clone());

                    let relative_defining_file = Some(relative_defining_file);
                    let constant_name = constant.fully_qualified_name.clone();

                    Ok(Reference {
                        constant_name,
                        defining_pack_name,
                        referencing_pack_name: referencing_pack_name.clone(),
                        relative_referencing_file: relative_referencing_file
                            .clone(),
                        source_location: source_location.clone(),
                        relative_defining_file,
                    })
                })
                .collect::<anyhow::Result<Vec<Reference>>>()?)
        } else {
            let defining_pack_name = None;
            let relative_defining_file = None;
            // Contant name is not known, so we'll just use the unresolved name for now
            let constant_name = unresolved_reference.name.clone();

            Ok(vec![Reference {
                constant_name,
                defining_pack_name,
                referencing_pack_name,
                relative_referencing_file,
                source_location,
                relative_defining_file,
            }])
        }
    }
}
