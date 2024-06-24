use std::path::Path;

pub fn packs_bin_name() -> String {
    if Path::new("bin/pks").exists() {
        "bin/pks".to_string()
    } else {
        "packs".to_string()
    }
}
