use core::fmt;
use std::{
    collections::HashMap,
    fs::File,
    io::BufRead,
    path::{Path, PathBuf},
};

use error_stack::{Context, Result};

use jwalk::WalkDir;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use regex::Regex;
use tracing::{debug, instrument};

use crate::error_stack_ext::IntoContext;
use glob_match::glob_match;

pub struct Project {
    pub base_path: PathBuf,
    pub files: Vec<ProjectFile>,
    pub packages: Vec<Package>,
    pub vendored_gems: Vec<VendoredGem>,
    pub teams: Vec<Team>,
    pub codeowners_file: String,
}

#[derive(Clone)]
pub struct VendoredGem {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Debug)]
pub struct ProjectFile {
    pub owner: Option<String>,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct Team {
    pub path: PathBuf,
    pub name: String,
    pub github_team: String,
    pub owned_globs: Vec<String>,
    pub owned_gems: Vec<String>,
    pub avoid_ownership: bool,
}

pub struct Package {
    pub path: PathBuf,
    pub package_type: PackageType,
    pub owner: String,
}

impl Package {
    pub fn package_root(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum PackageType {
    Ruby,
    Javascript,
}

mod deserializers {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Metadata {
        pub owner: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct JavascriptPackage {
        pub metadata: Option<Metadata>,
    }

    #[derive(Deserialize)]
    pub struct RubyPackage {
        pub owner: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct Github {
        pub team: String,
        #[serde(default = "bool_false")]
        pub do_not_add_to_codeowners_file: bool,
    }

    #[derive(Deserialize)]
    pub struct Ruby {
        #[serde(default = "empty_string_vec")]
        pub owned_gems: Vec<String>,
    }

    #[derive(Deserialize)]
    pub struct Team {
        pub name: String,
        pub github: Github,
        pub ruby: Option<Ruby>,

        #[serde(default = "empty_string_vec")]
        pub owned_globs: Vec<String>,
    }

    fn empty_string_vec() -> Vec<String> {
        Vec::new()
    }

    fn bool_false() -> bool {
        false
    }
}

#[derive(Debug)]
pub enum Error {
    Io,
    SerdeYaml,
    SerdeJson,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io => fmt.write_str("Error::Io"),
            Error::SerdeYaml => fmt.write_str("Error::SerdeYaml"),
            Error::SerdeJson => fmt.write_str("Error::SerdeJson"),
        }
    }
}

impl Context for Error {}

impl Project {
    // #[instrument(level = "debug", skip_all)]
    // pub fn build(base_path: &Path, codeowners_file_path: &Path, config: &Config) -> Result<Self, Error> {
    //     debug!(base_path = base_path.to_str(), "scanning project");

    //     let mut owned_file_paths: Vec<PathBuf> = Vec::new();
    //     let mut packages: Vec<Package> = Vec::new();
    //     let mut teams: Vec<Team> = Vec::new();
    //     let mut vendored_gems: Vec<VendoredGem> = Vec::new();

    //     for entry in WalkDir::new(base_path) {
    //         let entry = entry.into_context(Error::Io)?;

    //         let absolute_path = entry.path();
    //         let relative_path = absolute_path.strip_prefix(base_path).into_context(Error::Io)?.to_owned();

    //         if entry.file_type().is_dir() {
    //             if relative_path.parent() == Some(Path::new(&config.vendored_gems_path)) {
    //                 let file_name = relative_path.file_name().expect("expected a file_name");
    //                 vendored_gems.push(VendoredGem {
    //                     path: absolute_path,
    //                     name: file_name.to_string_lossy().to_string(),
    //                 })
    //             }

    //             continue;
    //         }

    //         let file_name = relative_path.file_name().expect("expected a file_name");

    //         if file_name.eq_ignore_ascii_case("package.yml") && matches_globs(relative_path.parent().unwrap(), &config.ruby_package_paths) {
    //             if let Some(owner) = ruby_package_owner(&absolute_path)? {
    //                 packages.push(Package {
    //                     path: relative_path.clone(),
    //                     owner,
    //                     package_type: PackageType::Ruby,
    //                 })
    //             }
    //         }

    //         if file_name.eq_ignore_ascii_case("package.json")
    //             && matches_globs(relative_path.parent().unwrap(), &config.javascript_package_paths)
    //         {
    //             if let Some(owner) = javascript_package_owner(&absolute_path)? {
    //                 packages.push(Package {
    //                     path: relative_path.clone(),
    //                     owner,
    //                     package_type: PackageType::Javascript,
    //                 })
    //             }
    //         }

    //         if matches_globs(&relative_path, &config.team_file_glob) {
    //             let file = File::open(&absolute_path).into_context(Error::Io)?;
    //             let deserializer: deserializers::Team = serde_yaml::from_reader(file).into_context(Error::SerdeYaml)?;

    //             teams.push(Team {
    //                 path: absolute_path.clone(),
    //                 name: deserializer.name,
    //                 github_team: deserializer.github.team,
    //                 owned_globs: deserializer.owned_globs,
    //                 owned_gems: deserializer.ruby.map(|ruby| ruby.owned_gems).unwrap_or(Vec::new()),
    //                 avoid_ownership: deserializer.github.do_not_add_to_codeowners_file,
    //             })
    //         }

    //         if matches_globs(&relative_path, &config.owned_globs) && !matches_globs(&relative_path, &config.unowned_globs) {
    //             owned_file_paths.push(absolute_path)
    //         }
    //     }

    //     debug!(
    //         owned_file_paths = owned_file_paths.len(),
    //         packages = packages.len(),
    //         teams = teams.len(),
    //         vendored_gems = vendored_gems.len(),
    //         "finished scanning project",
    //     );

    //     let codeowners_file: String = if codeowners_file_path.exists() {
    //         std::fs::read_to_string(codeowners_file_path).into_context(Error::Io)?
    //     } else {
    //         "".to_owned()
    //     };

    //     let owned_files = owned_files(owned_file_paths);

    //     Ok(Project {
    //         base_path: base_path.to_owned(),
    //         files: owned_files,
    //         vendored_gems,
    //         teams,
    //         packages,
    //         codeowners_file,
    //     })
    // }

    pub fn relative_path<'a>(&'a self, absolute_path: &'a Path) -> &'a Path {
        absolute_path
            .strip_prefix(&self.base_path)
            .expect("Could not generate relative path")
    }

    pub fn team_by_name(&self) -> HashMap<String, Team> {
        let mut result: HashMap<String, Team> = HashMap::new();

        for team in &self.teams {
            result.insert(team.name.clone(), team.clone());
        }

        result
    }

    pub fn vendored_gem_by_name(&self) -> HashMap<String, VendoredGem> {
        let mut result: HashMap<String, VendoredGem> = HashMap::new();

        for vendored_gem in &self.vendored_gems {
            result.insert(vendored_gem.name.clone(), vendored_gem.clone());
        }

        result
    }
}

#[instrument(level = "debug", skip_all)]
fn owned_files(owned_file_paths: Vec<PathBuf>) -> Vec<ProjectFile> {
    let regexp = Regex::new(r#"^(?:#|//) @team (.*)$"#).expect("error compiling regular expression");

    debug!("opening files to read ownership annotation");

    owned_file_paths
        .into_par_iter()
        .map(|path| {
            let file = File::open(&path).unwrap_or_else(|_| panic!("Couldn't open {}", path.to_string_lossy()));
            let first_line = std::io::BufReader::new(file).lines().next().transpose();
            let first_line = first_line.expect("error reading first line");

            if first_line.is_none() {
                return ProjectFile { path, owner: None };
            }

            if let Some(first_line) = first_line {
                let capture = regexp.captures(&first_line);

                if let Some(capture) = capture {
                    let first_capture = capture.get(1);

                    if let Some(first_capture) = first_capture {
                        return ProjectFile {
                            path,
                            owner: Some(first_capture.as_str().to_string()),
                        };
                    }
                }
            }

            ProjectFile { path, owner: None }
        })
        .collect()
}

// fn matches_globs(path: &Path, globs: &[String]) -> bool {
//     globs.iter().any(|glob| glob_match(glob, path.to_str().unwrap()))
// }
