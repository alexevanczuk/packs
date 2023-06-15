use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use itertools::Itertools;

use super::Pack;

#[derive(Default)]
pub struct PackSet {
    pub packs: Vec<Pack>,
    indexed_packs: HashMap<String, Pack>,
    for_file_cache: chashmap::CHashMap<PathBuf, Option<String>>,
}

impl PackSet {
    pub fn build(packs: HashSet<Pack>) -> PackSet {
        let packs: Vec<Pack> = packs
            .into_iter()
            .sorted_by(|packa, packb| {
                Ord::cmp(&packb.name.len(), &packa.name.len())
                    .then_with(|| packa.name.cmp(&packb.name))
            })
            .collect();
        let mut indexed_packs: HashMap<String, Pack> = HashMap::new();
        for pack in &packs {
            indexed_packs.insert(pack.name.clone(), pack.clone());
        }

        PackSet {
            indexed_packs,
            packs,
            for_file_cache: chashmap::CHashMap::new(),
        }
    }

    pub fn for_file(&self, absolute_file_path: &Path) -> Option<String> {
        if self.for_file_cache.contains_key(absolute_file_path) {
            self.for_file_cache.get(absolute_file_path).unwrap().clone()
        } else {
            let defining_pack_name = self.for_file_uncached(absolute_file_path);
            self.for_file_cache.insert(
                absolute_file_path.to_path_buf(),
                defining_pack_name.clone(),
            );
            defining_pack_name
        }
    }

    pub fn for_file_uncached(
        &self,
        absolute_file_path: &Path,
    ) -> Option<String> {
        for pack in &self.packs {
            if absolute_file_path.starts_with(pack.yml.parent().unwrap()) {
                return Some(pack.name.clone());
            }
        }

        None
    }

    pub fn for_pack(&self, pack_name: &str) -> &Pack {
        self.indexed_packs.get(pack_name).unwrap()
    }
}
