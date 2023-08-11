use std::{
    collections::HashSet,
    fs::File,
    hash::Hasher,
    io::Read,
    path::{Path, PathBuf},
};

use core::hash::Hash;
use serde::{Deserialize, Serialize};

use super::{
    checker::ViolationIdentifier,
    raw_pack::{self},
    PackageTodo,
};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub struct Pack {
    #[serde(skip_deserializing)]
    pub yml: PathBuf,
    #[serde(skip_deserializing)]
    pub name: String,
    #[serde(skip_deserializing)]
    pub relative_path: PathBuf,
    #[serde(default)]
    // I want to see if checkers and such can add their own deserialization
    // behavior to Pack via a trait or something? That would make extension simpler!
    pub dependencies: HashSet<String>,
    pub ignored_dependencies: HashSet<String>,
    pub ignored_private_constants: HashSet<String>,
    pub private_constants: HashSet<String>,
    pub package_todo: PackageTodo,
    pub visible_to: HashSet<String>,
    pub public_folder: PathBuf,
    pub layer: Option<String>,
    pub enforce_dependencies: CheckerSetting,
    pub enforce_privacy: CheckerSetting,
    pub enforce_visibility: CheckerSetting,
    pub enforce_architecture: CheckerSetting,
}

impl Hash for Pack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Pack {
    pub fn relative_yml(&self) -> PathBuf {
        self.relative_path.join("package.yml")
    }
}

#[derive(Debug, Default, PartialEq, Eq, Deserialize, Serialize, Clone)]
pub enum CheckerSetting {
    #[default]
    False,
    True,
    Strict,
}

impl CheckerSetting {
    /// Returns `true` if the checker setting is [`False`].
    ///
    /// [`False`]: CheckerSetting::False
    #[must_use]
    pub fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }
}

fn convert_raw_checker_setting(
    raw_checker_setting: &Option<String>,
) -> CheckerSetting {
    if let Some(raw_checker_setting) = raw_checker_setting {
        if raw_checker_setting == "strict" {
            CheckerSetting::Strict
        } else if raw_checker_setting == "true" {
            CheckerSetting::True
        } else if raw_checker_setting == "false" {
            CheckerSetting::False
        } else {
            panic!("Invalid checker setting: {}", raw_checker_setting);
        }
    } else {
        CheckerSetting::False
    }
}

impl Pack {
    pub fn all_violations(&self) -> Vec<ViolationIdentifier> {
        let mut violations = Vec::new();
        let violations_by_pack = &self.package_todo.violations_by_defining_pack;
        for (defining_pack_name, violation_groups) in violations_by_pack {
            for (constant_name, violation_group) in violation_groups {
                for violation_type in &violation_group.violation_types {
                    for file in &violation_group.files {
                        let identifier = ViolationIdentifier {
                            violation_type: violation_type.clone(),
                            file: file.clone(),
                            constant_name: constant_name.clone(),
                            referencing_pack_name: self.name.clone(),
                            defining_pack_name: defining_pack_name.clone(),
                        };

                        violations.push(identifier);
                    }
                }
            }
        }
        violations
    }

    pub fn from_path(
        package_yml_absolute_path: &Path,
        package_yml_relative_path: &Path,
    ) -> Pack {
        let mut relative_path = package_yml_relative_path
            .parent()
            .expect("Expected package to be in a parent directory")
            .to_owned();

        let mut name = relative_path
            .to_str()
            .expect("Non-unicode characters?")
            .to_owned();
        let yml = package_yml_absolute_path;

        // Handle the root pack
        if name == *"" {
            name = String::from(".");
            relative_path = PathBuf::from(".");
        };

        let raw_pack = raw_pack::from_path(yml);

        let absolute_path_to_package_todo = package_yml_absolute_path
            .parent()
            .unwrap()
            .join("package_todo.yml");

        let package_todo: PackageTodo = if absolute_path_to_package_todo
            .exists()
        {
            let mut package_todo_contents = String::new();
            let mut file = File::open(&absolute_path_to_package_todo)
                .expect("Failed to open the package_todo.yml file");
            file.read_to_string(&mut package_todo_contents)
                .expect("Could not read the package_todo.yml file");
            serde_yaml::from_str(&package_todo_contents).unwrap_or_else(|e| {

                panic!(
                    "Failed to deserialize the package_todo.yml file at {} with error {}",
                    absolute_path_to_package_todo.display(),
                    e
                )
            })
        } else {
            PackageTodo::default()
        };

        let dependencies = raw_pack.dependencies;
        let visible_to = raw_pack.visible_to.unwrap_or_default();
        let public_folder = relative_path.join(raw_pack.public_folder);
        let ignored_dependencies = raw_pack.ignored_dependencies;
        let ignored_private_constants = raw_pack.ignored_private_constants;
        let private_constants = raw_pack.private_constants;

        let enforce_dependencies =
            convert_raw_checker_setting(&raw_pack.enforce_dependencies);
        let enforce_privacy =
            convert_raw_checker_setting(&raw_pack.enforce_privacy);
        let enforce_visibility =
            convert_raw_checker_setting(&raw_pack.enforce_visibility);
        let enforce_architecture =
            convert_raw_checker_setting(&raw_pack.enforce_architecture);

        let layer = raw_pack.layer;

        let pack: Pack = Pack {
            yml: yml.to_path_buf(),
            name,
            relative_path,
            dependencies,
            package_todo,
            ignored_dependencies,
            ignored_private_constants,
            private_constants,
            visible_to,
            enforce_dependencies,
            enforce_privacy,
            enforce_visibility,
            enforce_architecture,
            public_folder,
            layer,
        };

        pack
    }
}
