use anyhow::Context;

use crate::packs::{
    checker::reference::Reference, pack::write_pack_to_disk,
    reference_extractor::get_all_references_and_sigils,
};

use super::{pack::Pack, Configuration};
use std::collections::HashSet;

/// Finds references to the provided constant and updates the associated packs to include the defining pack as a dependency.
pub fn update_dependencies_for_constant(
    configuration: &Configuration,
    constant_name: &str,
) -> anyhow::Result<usize> {
    let (all_references, _sigils) = get_all_references_and_sigils(
        configuration,
        &configuration.included_files,
    )?;
    if let Some((defining_pack_name, reference_pack_names_set)) =
        find_defining_and_referencing_packs(&all_references, constant_name)
    {
        let packs_for_update = find_pack_names_for_update(
            configuration,
            &defining_pack_name,
            &reference_pack_names_set,
        )?;
        let defining_pack = configuration
            .pack_set
            .for_pack(&defining_pack_name)
            .context("Could not find the defining pack")?;

        for pack in packs_for_update.iter() {
            let cloned_pack = pack.add_dependency(defining_pack);
            write_pack_to_disk(&cloned_pack)?;
        }

        Ok(packs_for_update.len())
    } else {
        Ok(0)
    }
}

fn find_pack_names_for_update<'a>(
    configuration: &'a Configuration,
    defining_pack_name: &'a str,
    reference_pack_names_set: &'a HashSet<String>,
) -> anyhow::Result<Vec<&'a Pack>> {
    let packs_for_update: Vec<&Pack> = reference_pack_names_set
        .iter()
        .filter_map(|referencing_pack_name| {
            let referencing_pack = configuration
                .pack_set
                .for_pack(referencing_pack_name)
                .context("Could not find the referencing pack")
                .ok()?;
            if referencing_pack.dependencies.contains(defining_pack_name) {
                None
            } else {
                Some(referencing_pack)
            }
        })
        .collect();
    Ok(packs_for_update)
}

fn find_defining_and_referencing_packs(
    all_references: &[Reference],
    constant_name: &str,
) -> Option<(String, HashSet<String>)> {
    let mut defining_pack_name_option: Option<&str> = None;
    let reference_pack_names_set: HashSet<&String> = all_references
        .iter()
        .filter_map(|reference| {
            if reference.constant_name == constant_name {
                if let Some(defining_pack_name) = &reference.defining_pack_name
                {
                    if defining_pack_name != &reference.referencing_pack_name {
                        defining_pack_name_option
                            .get_or_insert(defining_pack_name);
                        return Some(&reference.referencing_pack_name);
                    }
                }
            }
            None
        })
        .collect();
    defining_pack_name_option.map(|defining_pack_name| {
        (
            defining_pack_name.to_string(),
            reference_pack_names_set
                .iter()
                .map(|s| s.to_string())
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::packs::{PackSet, SourceLocation};

    fn example_references() -> Vec<Reference> {
        vec![
            Reference {
                constant_name: String::from("::Bar::BarChild"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/foo"),
                relative_referencing_file: String::from(
                    "packs/foo/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            },
            Reference {
                constant_name: String::from("::Bar::BarChild"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/bar"),
                relative_referencing_file: String::from(
                    "packs/bar/app/services/foo.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation { line: 3, column: 1 },
            },
            Reference {
                constant_name: String::from("::BarChild"),
                defining_pack_name: Some(String::from("packs/diff_bar")),
                referencing_pack_name: String::from("packs/baz"),
                relative_referencing_file: String::from(
                    "packs/baz/app/services/baz.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/diff_bar/app/api/diff_bar.rb",
                )),
                source_location: SourceLocation {
                    line: 33,
                    column: 1,
                },
            },
            Reference {
                constant_name: String::from("::Bar"),
                defining_pack_name: Some(String::from("packs/bar")),
                referencing_pack_name: String::from("packs/bizz"),
                relative_referencing_file: String::from(
                    "packs/bizz/app/services/baz.rb",
                ),
                relative_defining_file: Some(String::from(
                    "packs/bar/app/api/bar.rb",
                )),
                source_location: SourceLocation {
                    line: 53,
                    column: 1,
                },
            },
        ]
    }

    #[test]
    fn test_find_defining_and_referencing_packs() {
        let references = example_references();
        let (defining_pack_name, reference_pack_names_set) =
            find_defining_and_referencing_packs(&references, "::Bar::BarChild")
                .unwrap();
        assert_eq!(defining_pack_name, "packs/bar");
        assert_eq!(reference_pack_names_set.len(), 1);
        assert!(reference_pack_names_set.contains("packs/foo"));
    }

    #[test]
    fn test_find_defining_and_referencing_packs_when_constant_not_found() {
        let references = example_references();
        let result =
            find_defining_and_referencing_packs(&references, "NonExistent");
        assert!(result.is_none());
    }

    fn example_configuration() -> Configuration {
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };
        let referencing_pack_bar = Pack {
            name: String::from("packs/bar"),
            ..Pack::default()
        };
        let referencing_pack_baz = Pack {
            name: String::from("packs/baz"),
            dependencies: HashSet::from_iter(vec![String::from("packs/foo")]),
            ..Pack::default()
        };

        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };
        Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack_bar,
                    referencing_pack_baz,
                ]),
                HashMap::new(),
            )
            .unwrap(),
            ..Configuration::default()
        }
    }

    #[test]
    fn test_find_pack_names_for_update() {
        let configuration = example_configuration();

        let reference_pack_names_set = HashSet::from_iter(vec![
            String::from("packs/bar"),
            String::from("packs/baz"),
            String::from("packs/does-not-exist"),
        ]);

        let packs_for_update = find_pack_names_for_update(
            &configuration,
            "packs/foo",
            &reference_pack_names_set,
        );
        let packs_for_update = packs_for_update.unwrap();
        assert_eq!(packs_for_update.len(), 1);
        assert_eq!(packs_for_update[0].name, "packs/bar");
    }
}
