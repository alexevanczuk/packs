use anyhow::{bail, Result};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use itertools::Itertools;

use super::{checker::ViolationIdentifier, pack::Pack, Configuration};

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

#[derive(Debug)]
pub struct PackDependency<'a> {
    // from_pack has a package.yml dependency on to_pack
    pub from_pack: &'a Pack,
    pub to_pack: &'a Pack,
}

impl PackSet {
    pub fn build(
        packs: HashSet<Pack>,
        owning_package_yml_for_file: HashMap<PathBuf, PathBuf>,
    ) -> anyhow::Result<PackSet> {
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

        if !indexed_packs.contains_key(".") {
            bail!("No root pack found. First double check a root pack exists (a package.yml file in the application root). Secondly, double check your packwerk.yml `package_paths` includes the root pack by using command packs list-packs.");
        }

        Ok(PackSet {
            indexed_packs,
            packs,
            all_violations,
            owning_pack_name_for_file,
        })
    }

    pub fn for_file(
        &self,
        absolute_file_path: &Path,
    ) -> anyhow::Result<Option<&Pack>> {
        self.owning_pack_name_for_file
            .get(absolute_file_path)
            .map(|pack_name| self.for_pack(pack_name))
            .transpose()
            .map_err(|_| {
                anyhow::Error::msg(format!(
                    "Walking the directory identified that the following file belongs to a pack, but that pack cannot be found in the packset:\n{}",
                    absolute_file_path.display()
                ))
            })
    }

    pub fn for_pack(&self, pack_name: &str) -> Result<&Pack> {
        // Trim trailing slash on pack_name.
        // Since often the input arg here comes from the command line,
        // a command line auto-completer may add a trailing slash.
        let pack_name = pack_name.trim_end_matches('/');
        if let Some(pack) = self.indexed_packs.get(pack_name) {
            Ok(pack)
        } else {
            bail!("No pack found '{}'", pack_name)
        }
    }

    // Returns all of the package dependencies in the pack set.
    pub fn all_pack_dependencies<'a>(
        &'a self,
        configuration: &'a Configuration,
    ) -> Result<Vec<PackDependency>> {
        let mut pack_refs: Vec<PackDependency> = Vec::new();
        for from_pack in &configuration.pack_set.packs {
            for dependency_pack_name in &from_pack.dependencies {
                match configuration.pack_set.for_pack(dependency_pack_name) {
                    Ok(to_pack) => {
                        pack_refs.push(PackDependency { from_pack, to_pack })
                    }
                    Err(_) => {
                        bail!("{} has '{}' in its dependencies, but that pack cannot be found. Try `packs list-packs` to debug.",
                               from_pack.yml.to_string_lossy(),
                               dependency_pack_name);
                    }
                }
            }
        }
        Ok(pack_refs)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::packs::pack::Pack;

    use super::PackSet;

    fn example_pack_set() -> PackSet {
        let foo_pack = Pack {
            name: "packs/foo".to_string(),
            ..Pack::default()
        };
        let root_pack = Pack {
            name: ".".to_string(),
            ..Pack::default()
        };
        let mut packs = HashSet::new();
        packs.insert(foo_pack);
        packs.insert(root_pack);
        PackSet::build(packs, HashMap::new()).unwrap()
    }

    #[test]
    fn from_pack() {
        let pack_set = example_pack_set();
        let actual_pack = pack_set.for_pack("packs/foo");
        assert!(actual_pack.is_ok());
    }

    #[test]
    fn from_pack_with_slash() {
        let pack_set = example_pack_set();
        let actual_pack = pack_set.for_pack("packs/foo/");
        assert!(actual_pack.is_ok());
    }
}
