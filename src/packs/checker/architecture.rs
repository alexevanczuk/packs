use super::{CheckerInterface, ValidatorInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::pack::Pack;
use crate::packs::{Configuration, Violation};
use anyhow::{bail, Result};

#[derive(Default, Clone)]
pub struct Layers {
    pub layers: Vec<String>,
}

impl Layers {
    fn can_depend_on(
        &self,
        referencing_layer: &String,
        defining_layer: &String,
    ) -> Result<bool> {
        let referencing_layer_index = self
            .layers
            .iter()
            .position(|layer| layer == referencing_layer);

        let defining_layer_index =
            self.layers.iter().position(|layer| layer == defining_layer);

        match (referencing_layer_index, defining_layer_index) {
            (Some(referencing_layer_index), Some(defining_layer_index)) => {
                Ok(referencing_layer_index <= defining_layer_index)
            }
            _ => {
                bail!("Could not find one of layer `{}` or layer `{}` in `packwerk.yml`",
                    referencing_layer, defining_layer)
            }
        }
    }
}

impl ValidatorInterface for Checker {
    fn validate(&self, configuration: &Configuration) -> Option<Vec<String>> {
        let mut error_messages: Vec<String> = vec![];
        match configuration.pack_set.all_pack_dependencies(configuration) {
            Ok(dependencies) => {
                for pack_dependency in dependencies {
                    let (from_pack, to_pack) =
                        (pack_dependency.from_pack, pack_dependency.to_pack);
                    match dependency_permitted(
                        configuration,
                        from_pack,
                        to_pack,
                    ) {
                        Ok(true) => continue,
                        Ok(false) => {
                            let error_message = format!(
                                "Invalid 'dependencies' in '{}/package.yml'. '{}/package.yml' has a layer type of '{},' which cannot rely on '{},' which has a layer type of '{}.' `architecture_layers` can be found in packwerk.yml",
                                from_pack.relative_path.display(),
                                from_pack.relative_path.display(),
                                from_pack.layer.clone().unwrap(),
                                to_pack.name,
                                to_pack.layer.clone().unwrap(),
                            );
                            error_messages.push(error_message);
                        }
                        Err(error) => {
                            error_messages.push(error.to_string());
                            return Some(error_messages);
                        }
                    }
                }
            }
            Err(error) => {
                error_messages.push(error.to_string());
            }
        }

        if error_messages.is_empty() {
            None
        } else {
            Some(error_messages)
        }
    }
}

fn dependency_permitted(
    configuration: &Configuration,
    from_pack: &Pack,
    to_pack: &Pack,
) -> Result<bool> {
    if from_pack.enforce_architecture().is_false() {
        return Ok(true);
    }

    let (from_pack_layer, to_pack_layer) = (&from_pack.layer, &to_pack.layer);

    if from_pack_layer.is_none() || to_pack_layer.is_none() {
        return Ok(true);
    }

    let (from_pack_layer, to_pack_layer) = (
        from_pack_layer.as_ref().unwrap(),
        to_pack_layer.as_ref().unwrap(),
    );

    configuration
        .layers
        .can_depend_on(from_pack_layer, to_pack_layer)
}

pub struct Checker {
    pub layers: Layers,
}

impl CheckerInterface for Checker {
    fn check(
        &self,
        reference: &Reference,
        configuration: &Configuration,
    ) -> anyhow::Result<Option<Violation>> {
        let pack_set = &configuration.pack_set;

        let referencing_pack = &reference.referencing_pack(pack_set)?;

        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack(pack_set)?;
        if defining_pack.is_none() {
            return Ok(None);
        }
        let defining_pack = defining_pack.unwrap();

        if referencing_pack.enforce_architecture().is_false() {
            return Ok(None);
        }

        let defining_pack_name = &defining_pack.name;

        if relative_defining_file.is_none() {
            return Ok(None);
        }

        if referencing_pack_name == defining_pack_name {
            return Ok(None);
        }

        match (&referencing_pack.layer, &defining_pack.layer) {
            (Some(referencing_layer), Some(defining_layer)) => {
                if self
                    .layers
                    .can_depend_on(referencing_layer, defining_layer)
                    .unwrap()
                {
                    return Ok(None);
                }

                let message = format!(
                    "{}:{}:{}\nArchitecture violation: `{}` belongs to `{}` (whose layer is `{}`) cannot be accessed from `{}` (whose layer is `{}`)",
                    reference.relative_referencing_file,
                    reference.source_location.line,
                    reference.source_location.column,
                    reference.constant_name,
                    defining_pack_name,
                    defining_layer,
                    referencing_pack_name,
                    referencing_layer,
                );

                let violation_type = String::from("architecture");
                let file = reference.relative_referencing_file.clone();
                let identifier = ViolationIdentifier {
                    violation_type,
                    strict: referencing_pack.enforce_architecture().is_strict(),
                    file,
                    constant_name: reference.constant_name.clone(),
                    referencing_pack_name: referencing_pack_name.clone(),
                    defining_pack_name: defining_pack_name.clone(),
                };

                Ok(Some(Violation {
                    message,
                    identifier,
                }))
            }
            _ => Ok(None),
        }
    }

    fn violation_type(&self) -> String {
        "architecture".to_owned()
    }
}

#[cfg(test)]
mod tests {

    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use crate::packs::checker::common_test::tests::{
        build_expected_violation, default_defining_pack,
        default_referencing_pack, test_check, TestChecker,
    };
    use crate::packs::{
        configuration,
        pack::{CheckerSetting, Pack},
        PackSet,
    };

    use super::*;

    fn checker_with_layers() -> Checker {
        Checker {
            layers: Layers {
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
            },
        }
    }

    #[test]
    fn referencing_and_defining_pack_are_identical() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
               layer: Some("product".to_string()),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/foo".to_owned(),
                enforce_architecture: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nArchitecture violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "architecture".to_string(), false)),
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn reference_is_an_architecture_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
               layer: Some("product".to_string()),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/foo".to_owned(),
                enforce_architecture: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nArchitecture violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "architecture".to_string(), false)),
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn reference_is_a_strict_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
               layer: Some("product".to_string()),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/foo".to_owned(),
                enforce_architecture: Some(CheckerSetting::Strict),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nArchitecture violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "architecture".to_string(), true)),
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn reference_is_an_architecture_violation_but_not_enforced(
    ) -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                layer: Some("product".to_string()),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/foo".to_owned(),
                enforce_architecture: Some(CheckerSetting::False),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn reference_is_not_an_architecture_violation() -> anyhow::Result<()> {
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(Pack {
                name: "packs/bar".to_owned(),
                layer: Some("utilities".to_string()),
                ..default_defining_pack()
            }),
            referencing_pack: Pack {
                name: "packs/foo".to_owned(),
                enforce_architecture: Some(CheckerSetting::False),
                layer: Some("product".to_string()),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    struct ArchitectureTestCase {
        from_pack_name: String,
        from_pack_layer: Option<String>,
        from_pack_dependencies: HashSet<String>,
        from_pack_enforce_architecture: Option<CheckerSetting>,
        to_pack_name: String,
        to_pack_layer: Option<String>,
        layers: Vec<String>,
        expected_result: bool,
    }

    impl Default for ArchitectureTestCase {
        fn default() -> Self {
            ArchitectureTestCase {
                from_pack_name: String::from("packs/foo"),
                from_pack_layer: Some(String::from("utilities")),
                from_pack_enforce_architecture: Some(CheckerSetting::True),
                from_pack_dependencies: HashSet::from_iter(vec![String::from(
                    "packs/bar",
                )]),
                to_pack_name: String::from("packs/bar"),
                to_pack_layer: Some(String::from("product")),
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
                expected_result: false,
            }
        }
    }
    fn package_yml_architecture_test(test_case: ArchitectureTestCase) {
        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };

        let from_pack = Pack {
            name: test_case.from_pack_name,
            layer: test_case.from_pack_layer,
            enforce_architecture: test_case.from_pack_enforce_architecture,
            dependencies: test_case.from_pack_dependencies,
            ..Pack::default()
        };
        let to_pack = Pack {
            name: test_case.to_pack_name,
            layer: test_case.to_pack_layer,
            ..Pack::default()
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    from_pack.clone(),
                    to_pack.clone(),
                ]),
                HashMap::new(),
            )
            .unwrap(),
            layers: Layers {
                layers: test_case.layers,
            },
            ..Configuration::default()
        };

        let result = dependency_permitted(&configuration, &from_pack, &to_pack);
        assert_eq!(result.unwrap(), test_case.expected_result);
    }

    #[test]
    fn package_yml_dependency_not_permitted() {
        let test_case = ArchitectureTestCase::default();
        package_yml_architecture_test(test_case);
    }

    #[test]
    fn package_yml_dependency_permitted_violation_not_enforced() {
        let test_case = ArchitectureTestCase {
            from_pack_enforce_architecture: Some(CheckerSetting::False),
            expected_result: true,
            ..ArchitectureTestCase::default()
        };
        package_yml_architecture_test(test_case);
    }

    #[test]
    fn package_yml_dependency_permitted_violation_no_from_layer() {
        let test_case = ArchitectureTestCase {
            from_pack_layer: None,
            expected_result: true,
            ..ArchitectureTestCase::default()
        };
        package_yml_architecture_test(test_case);
    }

    #[test]
    fn package_yml_dependency_permitted_violation_no_to_layer() {
        let test_case = ArchitectureTestCase {
            to_pack_layer: None,
            expected_result: true,
            ..ArchitectureTestCase::default()
        };
        package_yml_architecture_test(test_case);
    }

    #[test]
    fn package_yml_dependency_permitted_violation_valid_layer() {
        let test_case = ArchitectureTestCase {
            expected_result: true,
            layers: vec![String::from("utilities"), String::from("product")],
            ..ArchitectureTestCase::default()
        };
        package_yml_architecture_test(test_case);
    }

    #[test]
    fn test_validate_with_architecture_violations() {
        let configuration = configuration::get(
            PathBuf::from(
                "tests/fixtures/app_with_architecture_violations_in_yml",
            )
            .canonicalize()
            .expect("Could not canonicalize path")
            .as_path(),
        )
        .unwrap();
        let checker = Checker {
            layers: Layers {
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
            },
        };

        let error = checker.validate(&configuration);
        let expected_message = vec![
            String::from("Invalid 'dependencies' in 'packs/baz/package.yml'. 'packs/baz/package.yml' has a layer type of 'technical_services,' which cannot rely on 'packs/bar,' which has a layer type of 'admin.' `architecture_layers` can be found in packwerk.yml"),
            String::from( "Invalid 'dependencies' in 'packs/foo/package.yml'. 'packs/foo/package.yml' has a layer type of 'product,' which cannot rely on 'packs/bar,' which has a layer type of 'admin.' `architecture_layers` can be found in packwerk.yml")
        ];
        assert_eq!(error, Some(expected_message));
    }
}
