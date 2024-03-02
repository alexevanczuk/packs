use std::{collections::HashSet, path::PathBuf};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tracing::debug;

use crate::packs::{
    get_experimental_constant_resolver, get_zeitwerk_constant_resolver,
    process_files_with_cache, ProcessedFile,
};

use super::{checker::reference::Reference, Configuration};

pub(crate) fn get_all_references(
    configuration: &Configuration,
    absolute_paths: &HashSet<PathBuf>,
) -> anyhow::Result<Vec<Reference>> {
    let cache = configuration.get_cache();

    debug!("Getting unresolved references (using cache if possible)");

    let (constant_resolver, processed_files_to_check) = if configuration
        .experimental_parser
    {
        // The experimental parser needs *all* processed files to get definitions
        let all_processed_files: Vec<ProcessedFile> = process_files_with_cache(
            &configuration.included_files,
            cache,
            configuration,
        )?;

        let constant_resolver = get_experimental_constant_resolver(
            &configuration.absolute_root,
            &all_processed_files,
            &configuration.ignored_definitions,
        );

        let processed_files_to_check = all_processed_files
            .into_iter()
            .filter(|processed_file| {
                absolute_paths.contains(&processed_file.absolute_path)
            })
            .collect();

        (constant_resolver, processed_files_to_check)
    } else {
        let processed_files: Vec<ProcessedFile> =
            process_files_with_cache(absolute_paths, cache, configuration)?;

        // The zeitwerk constant resolver doesn't look at processed files to get definitions
        let constant_resolver = get_zeitwerk_constant_resolver(
            &configuration.pack_set,
            &configuration.constant_resolver_configuration(),
        );

        (constant_resolver, processed_files)
    };

    debug!("Turning unresolved references into fully qualified references");
    let references: anyhow::Result<Vec<Reference>> = processed_files_to_check
        .par_iter()
        .try_fold(
            Vec::new,
            // Start with an empty vector for each thread
            |mut acc, processed_file| {
                // Try to fold results within a thread
                for unresolved_ref in &processed_file.unresolved_references {
                    let mut refs = Reference::from_unresolved_reference(
                        configuration,
                        constant_resolver.as_ref(),
                        unresolved_ref,
                        &processed_file.absolute_path,
                    )?;
                    acc.append(&mut refs); // Collect references, return error if any
                }
                Ok(acc)
            },
        )
        .try_reduce(
            Vec::new, // Start with an empty vector for the reduction
            |mut acc, mut vec| {
                // Try to reduce results across threads
                acc.append(&mut vec); // Combine vectors, no error expected here
                Ok(acc)
            },
        );
    debug!("Finished turning unresolved references into fully qualified references");

    references
}
