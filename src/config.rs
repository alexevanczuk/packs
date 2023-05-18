use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub include: Vec<String>,

    pub exclude: Vec<String>,

    #[serde(default =  "**/")]
    pub package_paths: Vec<String>,

    #[serde(alias = "js_package_paths")]
    pub javascript_package_paths: Vec<String>,

    #[serde(default = "team_file_glob")]
    pub team_file_glob: Vec<String>,
    pub unowned_globs: Vec<String>,

    #[serde(alias = "unbuilt_gems_path")]
    pub vendored_gems_path: String,
}

#[derive(Deserialize, Debug)]
pub struct RubyPackageConfig {
    #[serde(alias = "pack_paths")]
    pub ruby_package_paths: Vec<String>,
}

fn ruby_package_paths() -> Vec<String> {
    vec!["packs/**/*".to_owned(), "components/**".to_owned()]
}

fn team_file_glob() -> Vec<String> {
    vec!["config/teams/**/*.yml".to_owned()]
}
