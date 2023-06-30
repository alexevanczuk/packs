use std::{collections::HashSet, path::Path};

use regex::Regex;

// Load in config/initializers/inflections.rb
// For any inflections in there, add them to the acronyms vector
// An inflection takes the form of "inflect.acronym 'API'", so "API" would be the acronym here
// This is a bit of a hack, but it's the easiest way to get the inflections loaded in
// TODO: Figure out a better way to do this
pub(crate) fn get_acronyms_from_disk(absolute_root: &Path) -> HashSet<String> {
    let mut acronyms: HashSet<String> = HashSet::new();

    let inflections_path =
        absolute_root.join("config/initializers/inflections.rb");
    if inflections_path.exists() {
        let inflections_file =
            std::fs::read_to_string(inflections_path).unwrap();
        let inflections_lines = inflections_file.lines();
        for line in inflections_lines {
            if line.contains(".acronym") {
                let re = Regex::new(r#"['\\"]"#).unwrap();
                let acronym = re.split(line).nth(1).unwrap();
                acronyms.insert(acronym.to_string());
            }
        }
    }

    acronyms
}
