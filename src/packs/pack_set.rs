use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use itertools::Itertools;

use super::{checker::ViolationIdentifier, pack::Pack};

#[derive(Default, Debug)]
pub struct PackSet {
    pub packs: Vec<Pack>,
    indexed_packs: HashMap<String, Pack>,
    owning_pack_name_for_file: HashMap<PathBuf, String>,
    // For now, we keep track of all violations so that we can diff them and only
    // present the ones that are not recorded.
    // Eventually, we'll need to rewrite these to disk, in which case we'll need
    // more info in these Violations to aggregate them into package_todo.yml files.
    // We will also likely want to have an optimization that only rewrites the files
    // that have different violations.
    pub all_violations: HashSet<ViolationIdentifier>,
}

impl PackSet {
    pub fn build(
        packs: HashSet<Pack>,
        owning_package_yml_for_file: HashMap<PathBuf, PathBuf>,
    ) -> PackSet {
        let packs: Vec<Pack> = packs
            .into_iter()
            .sorted_by(|packa, packb| {
                Ord::cmp(&packb.name.len(), &packa.name.len())
                    .then_with(|| packa.name.cmp(&packb.name))
            })
            .collect();
        let mut indexed_packs_by_name: HashMap<String, Pack> = HashMap::new();
        let mut indexed_packs_by_yml: HashMap<PathBuf, String> = HashMap::new();

        let mut all_violations = HashSet::new();
        for pack in &packs {
            indexed_packs_by_name.insert(pack.name.clone(), pack.clone());
            indexed_packs_by_yml.insert(pack.yml.clone(), pack.name.clone());
            for violation_identifier in pack.all_violations() {
                all_violations.insert(violation_identifier);
            }
        }

        let mut owning_pack_name_for_file: HashMap<PathBuf, String> =
            HashMap::new();

        for (file, package_yml) in owning_package_yml_for_file {
            if let Some(pack_name) = indexed_packs_by_yml.get(&package_yml) {
                owning_pack_name_for_file.insert(file, pack_name.clone());
            }
        }

        let indexed_packs = indexed_packs_by_name;

        PackSet {
            indexed_packs,
            packs,
            all_violations,
            owning_pack_name_for_file,
        }
    }

    pub fn for_file(&self, absolute_file_path: &Path) -> Option<&Pack> {
        self.owning_pack_name_for_file.get(absolute_file_path).map(
            |pack_name| {
                let error_closure = |_| {
                    panic!(
                        "Walking the directory identified that the following file belongs to {}, but that pack cannot be found in the packset:\n{}",
                        pack_name,
                        &absolute_file_path.display(),
                    )
                };
                self.for_pack(pack_name).unwrap_or_else(error_closure)
            },
        )
    }

    pub fn for_pack(&self, pack_name: &str) -> Result<&Pack, &'static str> {
        if let Some(pack) = self.indexed_packs.get(pack_name) {
            Ok(pack)
        } else {
            Err("No pack found.")
        }
    }
}
