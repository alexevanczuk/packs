use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

use tracing::debug;

use crate::packs::{
    constant_resolver::ConstantDefinition, file_utils::glob_ruby_files_in_dirs,
    get_experimental_constant_resolver, process_files_with_cache,
    ProcessedFile,
};

use super::Configuration;

pub fn expose_monkey_patches(
    configuration: &Configuration,
    rubydir: &PathBuf,
    gemdir: &PathBuf,
) -> String {
    let mut lines_to_print: Vec<String> = vec![];
    if !configuration.experimental_parser {
        panic!("This command is only supported with the experimental parser! `packs help` for more info.")
    }

    debug!("Globbing out rubydir and gemdir");

    let other_files_to_parse = glob_ruby_files_in_dirs(vec![rubydir, gemdir]);
    let mut included_files = configuration.included_files.clone();

    included_files.extend(other_files_to_parse);

    let processed_files: Vec<ProcessedFile> = process_files_with_cache(
        &included_files,
        configuration.get_cache(),
        configuration,
    );

    let constant_resolver = get_experimental_constant_resolver(
        &configuration.absolute_root,
        &processed_files,
        &configuration.ignored_definitions,
    );

    let constant_definition_map = constant_resolver
        .fully_qualified_constant_name_to_constant_definition_map();

    let mut ruby_monkey_patches: Vec<&ConstantDefinition> = vec![];
    let mut gem_monkey_patches: Vec<&ConstantDefinition> = vec![];
    let mut app_monkey_patches: Vec<&ConstantDefinition> = vec![];
    let mut definitions_by_gem: HashMap<String, String> = HashMap::new();

    constant_definition_map
        .iter()
        .for_each(|(_name, definitions)| {
            let mut stdlib_defs: Vec<&ConstantDefinition> = vec![];
            let mut gem_defs: Vec<&ConstantDefinition> = vec![];
            let mut app_defs: Vec<&ConstantDefinition> = vec![];
            definitions.iter().for_each(|d| {
                let path = &d.absolute_path_of_definition;
                if path.starts_with(rubydir) {
                    stdlib_defs.push(d)
                } else if path.starts_with(gemdir) {
                    gem_defs.push(d)
                } else {
                    app_defs.push(d)
                }
            });

            let stdlib_definitions_count = stdlib_defs.len();
            let gem_definitions_count = gem_defs.len();

            if gem_definitions_count > 0 {
                for def in gem_defs {
                    let relative_path = def
                        .absolute_path_of_definition
                        .strip_prefix(gemdir)
                        .unwrap();
                    let gem_name = relative_path
                        .components()
                        .next()
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap();

                    definitions_by_gem.insert(
                        def.fully_qualified_name.clone(),
                        gem_name.to_owned(),
                    );
                }
            }

            let app_definitions_count = app_defs.len();
            match (
                stdlib_definitions_count,
                gem_definitions_count,
                app_definitions_count,
            ) {
                (1.., 1.., 0) => {
                    // skip: gems monkey patching ruby
                }
                (1.., 0, 1..) => ruby_monkey_patches.extend(app_defs),
                // This one is also gems monkey patching ruby, but we skip that info for now
                (1.., 1.., 1..) => ruby_monkey_patches.extend(app_defs),
                (0.., 1.., 1..) => gem_monkey_patches.extend(app_defs),
                (0.., 0.., 2..) => app_monkey_patches.extend(app_defs),
                (_, _, _) => {
                    // skip
                }
            }
        });

    let constant_definition_sorting_function =
        |a: &&ConstantDefinition, b: &&ConstantDefinition| {
            // Compare by fully_qualified_name first
            match a.fully_qualified_name.cmp(&b.fully_qualified_name) {
                // If fully_qualified_name is the same, compare by absolute_path_of_definition
                Ordering::Equal => a
                    .absolute_path_of_definition
                    .cmp(&b.absolute_path_of_definition),
                // Otherwise, sort by fully_qualified_name
                other => other,
            }
        };

    lines_to_print.push("The following is a list of constants that are redefined by your app.\n".to_owned());
    lines_to_print.push("# Ruby Standard Library".to_owned());
    lines_to_print.push(format!("These monkey patches redefine behavior in the Ruby standard library (as determined by parsing the contents of `{}`):", rubydir.display()));

    ruby_monkey_patches.sort_by(constant_definition_sorting_function);
    gem_monkey_patches.sort_by(constant_definition_sorting_function);
    app_monkey_patches.sort_by(constant_definition_sorting_function);

    for definition in ruby_monkey_patches {
        lines_to_print.push(get_redefinition_line_to_print(
            &configuration.absolute_root,
            definition,
        ));
    }

    lines_to_print.push("\n# Gems".to_owned());
    lines_to_print.push(format!("These monkey patches redefine behavior in gems your app depends on (as determined by parsing the contents of `{}`):", gemdir.display()));
    for definition in gem_monkey_patches {
        let relative_path = definition
            .absolute_path_of_definition
            .strip_prefix(&configuration.absolute_root)
            .unwrap();

        let gem_name = definitions_by_gem
            .get(&definition.fully_qualified_name)
            .unwrap();

        lines_to_print.push(format!(
            "{} (from gem `{}`) is redefined at {}",
            definition.fully_qualified_name,
            gem_name,
            relative_path.display()
        ))
    }

    lines_to_print.push("\n# Application".to_owned());
    lines_to_print.push("These monkey patches redefine behavior in a pack within your app (as determined by parsing your app's packs):".to_owned());
    for definition in app_monkey_patches {
        lines_to_print.push(get_redefinition_line_to_print(
            &configuration.absolute_root,
            definition,
        ));
    }

    lines_to_print.join("\n")
}

fn get_redefinition_line_to_print(
    absolute_root: &PathBuf,
    definition: &ConstantDefinition,
) -> String {
    let relative_path = definition
        .absolute_path_of_definition
        .strip_prefix(absolute_root)
        .unwrap();
    format!(
        "{} is redefined at {}",
        definition.fully_qualified_name,
        relative_path.display(),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::packs::configuration;

    use super::expose_monkey_patches;

    #[test]
    fn test_expose_monkey_patches() {
        let expected_message = String::from("\
The following is a list of constants that are redefined by your app.

# Ruby Standard Library
These monkey patches redefine behavior in the Ruby standard library (as determined by parsing the contents of `tests/fixtures/app_with_monkey_patches/rubydir_stub`):
::Date is redefined at config/initializers/string_and_date_extensions.rb
::String is redefined at config/initializers/string_and_date_extensions.rb

# Gems
These monkey patches redefine behavior in gems your app depends on (as determined by parsing the contents of `tests/fixtures/app_with_monkey_patches/gemdir_stub`):
::Rails (from gem `rails`) is redefined at config/initializers/rails_monkeypatch.rb

# Application
These monkey patches redefine behavior in a pack within your app (as determined by parsing your app's packs):
::Foo is redefined at packs/foo/app/models/foo.rb
::Foo is redefined at packs/foo/app/services/foo.rb
::SomeRootClass is redefined at app/models/some_root_class.rb
::SomeRootClass is redefined at app/services/some_root_class.rb"
      );

        let mut configuration = configuration::get(&PathBuf::from(
            "tests/fixtures/app_with_monkey_patches",
        ));
        configuration.experimental_parser = true;
        let actual_message = expose_monkey_patches(
            &configuration,
            &PathBuf::from(
                "tests/fixtures/app_with_monkey_patches/rubydir_stub",
            ),
            &PathBuf::from(
                "tests/fixtures/app_with_monkey_patches/gemdir_stub",
            ),
        );

        assert_eq!(expected_message, actual_message);
    }
}
