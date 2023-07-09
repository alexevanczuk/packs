use std::path::Path;

use crate::packs::{
    constant_resolver::ConstantResolver, pack::Pack,
    parsing::UnresolvedReference, Configuration, SourceLocation,
};

#[derive(Debug)]
pub struct Reference<'a> {
    pub constant_name: String,
    pub defining_pack: Option<&'a Pack>,
    pub relative_defining_file: Option<String>,
    pub referencing_pack: &'a Pack,
    pub relative_referencing_file: String,
    pub source_location: SourceLocation,
}

impl<'a> Reference<'a> {
    pub fn from_unresolved_reference(
        configuration: &'a Configuration,
        constant_resolver: &'a (dyn ConstantResolver + Send + Sync),
        unresolved_reference: &UnresolvedReference,
        referencing_file_path: &Path,
    ) -> Vec<Reference<'a>> {
        let referencing_pack = configuration
            .pack_set
            .for_file(referencing_file_path)
            .unwrap_or_else(|| {
                panic!(
                    "Could not find pack for referencing file path: {}",
                    &referencing_file_path.display()
                )
            });

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

        let (defining_pack, relative_defining_file, constant_name) =
            if let Some(constant) = &maybe_constant_definition {
                let absolute_path_of_definition =
                    &constant.absolute_path_of_definition;
                let relative_defining_file = absolute_path_of_definition
                    .strip_prefix(&configuration.absolute_root)
                    .unwrap()
                    .to_path_buf()
                    .to_str()
                    .unwrap()
                    .to_string();

                let defining_pack = configuration
                    .pack_set
                    .for_file(absolute_path_of_definition);

                let relative_defining_file = Some(relative_defining_file);
                let constant_name = constant.fully_qualified_name.clone();

                (defining_pack, relative_defining_file, constant_name)
            } else {
                let defining_pack = None;
                let relative_defining_file = None;
                // Contant name is not known, so we'll just use the unresolved name for now
                let constant_name = unresolved_reference.name.clone();

                (defining_pack, relative_defining_file, constant_name)
            };

        vec![Reference {
            constant_name,
            defining_pack,
            referencing_pack,
            relative_referencing_file,
            source_location,
            relative_defining_file,
        }]
    }
}
