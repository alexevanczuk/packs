use std::fs;

use super::{
    pack::{serialize_pack, Pack},
    Configuration,
};

pub(crate) fn lint_package_yml_files(configuration: &Configuration) {
    for pack in &configuration.pack_set.packs {
        let pack_yml = pack.yml.clone();
        let raw_pack = Pack::from_path(&pack_yml, &configuration.absolute_root);

        let linted_pack_yml = serialize_pack(&raw_pack);

        // Write the linted YAML content back to the file
        fs::write(&pack_yml, linted_pack_yml)
            .expect("Failed to write linted YAML to file");
    }
}
