use super::{CheckerInterface, ValidatorInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::pack::{CheckerSetting, Pack};
use crate::packs::{Configuration, Violation};
use anyhow::{bail, Result};

#[derive(Default, Debug, Clone)]
pub struct Layers {
    pub layers: Vec<String>,
    pub using_deprecated_keys: bool,
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

    fn pack_enforces_layers<'a>(&self, pack: &'a Pack) -> &'a CheckerSetting {
        let checker_setting = match self.using_deprecated_keys {
            true => &pack.enforce_architecture,
            false => &pack.enforce_layers,
        };
        match checker_setting {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    fn inconsistent_enforce_key_error(&self, pack: &Pack) -> Option<String> {
        if self.using_deprecated_keys && pack.enforce_layers.is_some() {
            return Some(format!("Unknown 'enforce_layers' specified in '{}'. Did you mean 'enforce_architecture'?", 
            &pack.relative_yml().to_string_lossy()));
        } else if !self.using_deprecated_keys
            && pack.enforce_architecture.is_some()
        {
            return Some(format!("Unknown 'enforce_architecture' specified in '{}'. Did you mean 'enforce_layers'?",
            &pack.relative_yml().to_string_lossy()));
        }
        None
    }

    fn enforce_key(&self) -> String {
        match self.using_deprecated_keys {
            true => "enforce_architecture".to_string(),
            false => "enforce_layers".to_string(),
        }
    }

    fn violation_type(&self) -> String {
        match self.using_deprecated_keys {
            true => "architecture".to_string(),
            false => "layer".to_string(),
        }
    }

    fn violation_name(&self) -> String {
        match self.using_deprecated_keys {
            true => "Architecture".to_string(),
            false => "Layer".to_string(),
        }
    }
}

impl Checker {
    fn validate_pack(&self, pack: &Pack) -> Option<String> {
        if let Some(error_message) =
            self.layers.inconsistent_enforce_key_error(pack)
        {
            return Some(error_message);
        }
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
                        "'layer' must be specified in '{}' because `{}` is true or strict.",
                        pack.relative_yml().to_string_lossy(),
                        self.layers.enforce_key(),
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
        let pack_set = &configuration.pack_set;

        let referencing_pack = &reference.referencing_pack(pack_set)?;

        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack(pack_set)?;
        if defining_pack.is_none() {
            return Ok(None);
        }
        let defining_pack = defining_pack.unwrap();

        if self
            .layers
            .pack_enforces_layers(referencing_pack)
            .is_false()
        {
            return Ok(None);
        }

        let defining_pack_name = &defining_pack.name;

        let relative_defining_file = match relative_defining_file {
            Some(file) => file,
            None => return Ok(None),
        };

        if referencing_pack_name == defining_pack_name {
            return Ok(None);
        }

        match (&referencing_pack.layer, &defining_pack.layer) {
            (Some(referencing_layer), Some(defining_layer)) => {
                if self
                    .layers
                    .can_depend_on(referencing_layer, defining_layer)?
                {
                    return Ok(None);
                }

                if referencing_pack.is_ignored(
                    relative_defining_file,
                    &self.violation_type(),
                )? {
                    return Ok(None);
                }

                let message = format!(
                    "{}:{}:{}\n{} violation: `{}` belongs to `{}` (whose layer is `{}`) cannot be accessed from `{}` (whose layer is `{}`)",
                    reference.relative_referencing_file,
                    reference.source_location.line,
                    reference.source_location.column,
                    self.layers.violation_name(),
                    reference.constant_name,
                    defining_pack_name,
                    defining_layer,
                    referencing_pack_name,
                    referencing_layer,
                );

                let violation_type = self.layers.violation_type();
                let file = reference.relative_referencing_file.clone();
                let identifier = ViolationIdentifier {
                    violation_type,
                    strict: self
                        .layers
                        .pack_enforces_layers(referencing_pack)
                        .is_strict(),
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

    fn checker_with_layers(deprecated: bool) -> Checker {
        Checker {
            layers: Layers {
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
                using_deprecated_keys: deprecated,
            },
        }
    }

    #[test]
    fn referencing_and_defining_pack_are_identical() -> anyhow::Result<()> {
        let pack = Pack {
            name: "packs/foo".to_owned(),
            enforce_architecture: Some(CheckerSetting::True),
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
            ..Default::default()
        };
        test_check(&checker_with_layers(true), &mut test_checker)
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
                enforce_layers: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nLayer violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "layer".to_string(), false)),
            ..Default::default()
        };
        test_check(&checker_with_layers(false), &mut test_checker)
    }

    #[test]
    fn reference_is_an_architecture_violation_with_deprecated_keys(
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
                enforce_architecture: Some(CheckerSetting::True),
                layer: Some("utilities".to_string()),
                ..default_referencing_pack()
            },
            expected_violation: Some(build_expected_violation(
                "packs/foo/app/services/foo.rb:3:1\nArchitecture violation: `::Bar` belongs to `packs/bar` (whose layer is `product`) cannot be accessed from `packs/foo` (whose layer is `utilities`)".to_string(), 
                "architecture".to_string(), false)),
            ..Default::default()
        };
        test_check(&checker_with_layers(true), &mut test_checker)
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
            ..Default::default()
        };
        test_check(&checker_with_layers(false), &mut test_checker)
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
        test_check(&checker_with_layers(false), &mut test_checker)
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
        test_check(&checker_with_layers(false), &mut test_checker)
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
                }]),
                ..default_referencing_pack()
            },
            ..Default::default()
        };
        test_check(&checker_with_layers(false), &mut test_checker)
    }

    fn validate_layers(
        config_layers: Vec<String>,
        using_deprecated_keys: bool,
        deprecated_enforcement: bool,
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
        if deprecated_enforcement {
            test_pack.enforce_architecture = package_enforce_layer;
        } else {
            test_pack.enforce_layers = package_enforce_layer;
        }
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
                using_deprecated_keys,
            },
        };
        checker.validate(&configuration)
    }

    #[test]
    fn validate_layers_strict_true() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("product")),
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("product")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_false_none() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            None,
            Some(CheckerSetting::False),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            None,
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_false_none_but_layer_specified() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("product")),
            Some(CheckerSetting::False),
        );
        assert_eq!(result, None);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("product")),
            None,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn validate_layers_true_strict_with_no_layer() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            None,
            Some(CheckerSetting::True),
        );
        assert_eq!(result, Some(vec![String::from("'layer' must be specified in 'packs/foo/package.yml/package.yml' because `enforce_layers` is true or strict.")]));

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            None,
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, Some(vec![String::from("'layer' must be specified in 'packs/foo/package.yml/package.yml' because `enforce_layers` is true or strict.")]));
    }

    #[test]
    fn validate_layers_when_inconsistent_enforce_key() {
        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            true,
            Some(String::from("product")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, Some(vec![String::from("Unknown 'enforce_architecture' specified in 'packs/foo/package.yml/package.yml'. Did you mean 'enforce_layers'?")]));

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            true,
            false,
            Some(String::from("product")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, Some(vec![String::from("Unknown 'enforce_layers' specified in 'packs/foo/package.yml/package.yml'. Did you mean 'enforce_architecture'?")]));
    }

    #[test]
    fn validate_layers_with_not_found_layer() {
        let expected_error = Some(vec![String::from("Invalid 'layer' option in 'packs/foo/package.yml/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`")]);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("not defined")),
            Some(CheckerSetting::True),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("not defined")),
            Some(CheckerSetting::Strict),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
            Some(String::from("not defined")),
            Some(CheckerSetting::False),
        );
        assert_eq!(result, expected_error);

        let result = validate_layers(
            vec![String::from("product"), String::from("utilities")],
            false,
            false,
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
                using_deprecated_keys: false,
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
                using_deprecated_keys: true,
            },
        };

        let error = checker.validate(&configuration);
        assert!(error.is_some());
        let mut errors = error.unwrap();
        errors.sort();

        let expected_errors = vec![
            "'layer' must be specified in 'packs/baz/package.yml' because `enforce_architecture` is true or strict.".to_string(), 
            "Invalid 'layer' option in 'packs/bar/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`".to_string(), 
            "Invalid 'layer' option in 'packs/foo/package.yml'. `layer` must be one of the layers defined in `packwerk.yml`".to_string()
        ];
        assert_eq!(errors, expected_errors);
    }
}
