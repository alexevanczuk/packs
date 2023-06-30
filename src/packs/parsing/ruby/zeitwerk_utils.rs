use std::path::PathBuf;

use tracing::debug;

use crate::packs::{file_utils::process_glob_pattern, Pack};

pub(crate) fn get_autoload_paths(packs: &Vec<Pack>) -> Vec<PathBuf> {
    let mut autoload_paths: Vec<PathBuf> = Vec::new();

    debug!(
        target: "perf_events",
        "Getting autoload paths"
    );

    for pack in packs {
        // App paths
        let app_paths = pack.yml.parent().unwrap().join("app").join("*");
        let app_glob_pattern = app_paths.to_str().unwrap();
        process_glob_pattern(app_glob_pattern, &mut autoload_paths);

        // Concerns paths
        let concerns_paths = pack
            .yml
            .parent()
            .unwrap()
            .join("app")
            .join("*")
            .join("concerns");
        let concerns_glob_pattern = concerns_paths.to_str().unwrap();

        process_glob_pattern(concerns_glob_pattern, &mut autoload_paths);
    }

    debug!(
        target: "perf_events",
        "Finished getting autoload paths"
    );

    autoload_paths
}
