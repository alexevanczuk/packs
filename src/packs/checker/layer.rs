use super::output_helper::print_reference_location;
use super::pack_checker::PackChecker;
use super::{CheckerInterface, ValidatorInterface};
use crate::packs::checker::Reference;
use crate::packs::pack::{CheckerSetting, Pack};
use crate::packs::{Configuration, Violation};
use anyhow::{bail, Result};

#[derive(Default, Debug, Clone)]
pub struct Layers {
    pub layers: Vec<String>,
}

const VIOLATION_TYPE: &str = "layer";
const VIOLATION_NAME: &str = "Layer";

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

    fn pack_enforces_layers<'a>(&self, pack: &'a Pack) -> &'a CheckerSetting {
        match &pack.enforce_layers {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    fn violation_type(&self) -> String {
        VIOLATION_TYPE.to_string()
    }

    fn violation_name(&self) -> String {
        VIOLATION_NAME.to_string()
    }
}

impl Checker {
    fn validate_pack(&self, pack: &Pack) -> Option<String> {
        match &pack.layer {
            Some(layer) => {
                if self.layers.layers.contains(layer) {
                    None
                } else {
                    Some(format!(
                        "Invalid 'layer' option in '{}'. `layer` must be one of the layers defined in `packwerk.yml`",
                        &pack.relative_yml().to_string_lossy()
                    ))
                }
            }
            None => match self.layers.pack_enforces_layers(pack) {
                CheckerSetting::False => None,
                _ => {
                    Some(format!(
                        "'layer' must be specified in '{}' because `enforce_layers` is true or strict.",
                        pack.relative_yml().to_string_lossy(),
                    ))
                }
            },
        }
    }
}

impl ValidatorInterface for Checker {
    fn validate(&self, configuration: &Configuration) -> Option<Vec<String>> {
        let mut error_messages: Vec<String> = vec![];

        for pack in &configuration.pack_set.packs {
            if let Some(error_message) = self.validate_pack(pack) {
                error_messages.push(error_message);
            }
        }

        if error_messages.is_empty() {
            None
        } else {
            Some(error_messages)
        }
    }
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
        let pack_checker =
            PackChecker::new(configuration, reference, &self.violation_type())?;
        if !pack_checker.checkable()? {
            return Ok(None);
        }
        let defining_pack = pack_checker.defining_pack.unwrap();

        match (&pack_checker.referencing_pack.layer, &defining_pack.layer) {
            (Some(referencing_layer), Some(defining_layer)) => {
                if self
                    .layers
                    .can_depend_on(referencing_layer, defining_layer)?
                {
                    return Ok(None);
                }

                let loc = print_reference_location(reference);

                let message = format!(
                    "{}{} violation: `{}` belongs to `{}` (whose layer is `{}`) cannot be accessed from `{}` (whose layer is `{}`)",
                    loc,
                    self.layers.violation_name(),
                    reference.constant_name,
                    defining_pack.name,
                    defining_layer,
                    pack_checker.referencing_pack.name,
                    referencing_layer,
                );

                Ok(Some(Violation {
                    message,
                    identifier: pack_checker.violation_identifier(),
                }))
            }
            _ => Ok(None),
        }
    }

    fn violation_type(&self) -> String {
        self.layers.violation_type()
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
    use crate::packs::pack::EnforcementGlobsIgnore;
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
        let pack = Pack {
            name: "packs/foo".to_owned(),
            enforce_layers: Some(CheckerSetting::True),
            layer: Some("utilities".to_string()),
            ..default_referencing_pack()
        };
        let mut test_checker = TestChecker {
            reference: None,
            configuration: None,
            referenced_constant_name: Some(String::from("::Bar")),
            defining_pack: Some(pack.clone()),
            referencing_pack: pack,
            expected_violation: None,
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn reference_is_a_layer_violation() -> anyhow::Result<()> {
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
                enforce_layers: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nLayer violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "layer".to_string(), false)),
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
                enforce_layers: Some(CheckerSetting::Strict),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nLayer violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "layer".to_string(), true)),
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
                enforce_layers: Some(CheckerSetting::False),
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
                enforce_layers: Some(CheckerSetting::False),
                layer: Some("product".to_string()),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    #[test]
    fn test_with_enforcement_globs_ignore() -> anyhow::Result<()> {
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
                enforce_layers: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                enforcement_globs_ignore: Some(vec![EnforcementGlobsIgnore {
                    enforcements: ["layer"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    ignores: ["packs/bar/**"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    reason: "deprecated".to_string(),
                }]),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&checker_with_layers(), &mut test_checker)
    }

    fn validate_layers(
        config_layers: Vec<String>,
        package_layer: Option<String>,
        package_enforce_layer: Option<CheckerSetting>,
    ) -> Option<Vec<String>> {
        let root_pack = Pack {
            name: String::from("."),
            layer: None,
            ..Pack::default()
        };
        let mut test_pack = Pack {
            name: String::from("packs/foo"),
            relative_path: PathBuf::from("packs/foo/package.yml"),
            layer: package_layer,
            ..Pack::default()
        };

        test_pack.enforce_layers = package_enforce_layer;
        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![root_pack, test_pack]),
                HashMap::new(),
            )
            .unwrap(),
            ..Configuration::default()
        };
        let checker = Checker {
            layers: Layers {
                layers: config_layers,
            },
        };
        checker.validate(&configuration)
    }

    #[test]
    fn validate_layers_strict_true() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("product")),
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("product")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_false_none() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            None,
            Some(CheckerSetting::False),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            None,
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_false_none_but_layer_specified() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("product")),
            Some(CheckerSetting::False),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("product")),
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_true_strict_with_no_layer() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            None,
            Some(CheckerSetting::True),
        );
        assert_eq!(result, Some(vec![String::from("'layer' must be specified in 'packs/foo/package.yml/package.yml' because `enforce_layers` is true or strict.")]));

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            None,
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, Some(vec![String::from("'layer' must be specified in 'packs/foo/package.yml/package.yml' because `enforce_layers` is true or strict.")]));
    }

    #[test]
    fn validate_layers_with_not_found_layer() {
        let expected_error = Some(vec![String::from("Invalid 'layer' option in 'packs/foo/package.yml/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`")]);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("not defined")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("not defined")),
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("not defined")),
            Some(CheckerSetting::False),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            Some(String::from("not defined")),
            None,
        );
        assert_eq!(result, expected_error);
    }

    #[test]
    fn test_validate_with_layer_violations() {
        let configuration = configuration::get(
            PathBuf::from("tests/fixtures/app_with_layer_violations_in_yml")
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
        assert!(error.is_some());
        let mut errors = error.unwrap();
        errors.sort();

        let expected_errors = vec![
            "'layer' must be specified in 'packs/baz/package.yml' because `enforce_layers` is true or strict.".to_string(), 
            "Invalid 'layer' option in 'packs/bar/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`".to_string(), 
            "Invalid 'layer' option in 'packs/foo/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`".to_string()
        ];
        assert_eq!(errors, expected_errors);
    }
}
