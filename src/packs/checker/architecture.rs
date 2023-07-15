use super::{CheckerInterface, ViolationIdentifier};
use crate::packs::checker::Reference;
use crate::packs::pack::CheckerSetting;
use crate::packs::{Configuration, Violation};

#[derive(Default, Clone)]
pub struct Layers {
    pub layers: Vec<String>,
}

impl Layers {
    fn can_depend_on(
        &self,
        referencing_layer: &String,
        defining_layer: &String,
    ) -> bool {
        let referencing_layer_index = self
            .layers
            .iter()
            .position(|layer| layer == referencing_layer);

        let defining_layer_index =
            self.layers.iter().position(|layer| layer == defining_layer);

        match (referencing_layer_index, defining_layer_index) {
            (Some(referencing_layer_index), Some(defining_layer_index)) => {
                referencing_layer_index <= defining_layer_index
            }
            _ => {
                panic!(
                    "Could not find one of layer `{}` or layer `{}` in `packwerk.yml`",
                    referencing_layer, defining_layer
                )
            }
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
    ) -> Option<Violation> {
        let pack_set = &configuration.pack_set;

        let referencing_pack = &reference.referencing_pack(pack_set);

        let relative_defining_file = &reference.relative_defining_file;

        let referencing_pack_name = &referencing_pack.name;
        let defining_pack = &reference.defining_pack(pack_set);
        if defining_pack.is_none() {
            return None;
        }
        let defining_pack = defining_pack.unwrap();

        if referencing_pack.enforce_architecture.is_false() {
            return None;
        }

        let defining_pack_name = &defining_pack.name;

        if relative_defining_file.is_none() {
            return None;
        }

        if referencing_pack_name == defining_pack_name {
            return None;
        }

        match (&referencing_pack.layer, &defining_pack.layer) {
            (Some(referencing_layer), Some(defining_layer)) => {
                if self.layers.can_depend_on(referencing_layer, defining_layer)
                {
                    return None;
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
                    file,
                    constant_name: reference.constant_name.clone(),
                    referencing_pack_name: referencing_pack_name.clone(),
                    defining_pack_name: defining_pack_name.clone(),
                };

                Some(Violation {
                    message,
                    identifier,
                })
            }
            _ => None,
        }
    }

    fn is_strict_mode_violation(
        &self,
        violation: &ViolationIdentifier,
        configuration: &Configuration,
    ) -> bool {
        let referencing_pack = configuration
            .pack_set
            .for_pack(&violation.referencing_pack_name)
            .unwrap_or_else(|_| panic!("Violation refers to pack named {}, but it can't be found in the packset.", &violation.referencing_pack_name));

        referencing_pack.enforce_architecture == CheckerSetting::Strict
    }

    fn violation_type(&self) -> String {
        "architecture".to_owned()
    }
}

#[cfg(test)]
mod tests {

    use std::collections::{HashMap, HashSet};

    use crate::packs::{
        pack::{CheckerSetting, Pack},
        PackSet, SourceLocation,
    };

    use super::*;

    #[test]
    fn referencing_and_defining_pack_are_identical() {
        let checker = Checker {
            layers: Layers::default(),
        };

        let defining_pack = Pack {
            name: String::from("packs/foo"),
            enforce_visibility: CheckerSetting::True,
            ..Pack::default()
        };
        let referencing_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/foo/app/services/foo.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/bar/app/services/bar.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![defining_pack, referencing_pack]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };
        assert_eq!(None, checker.check(&reference, &configuration))
    }

    #[test]
    fn reference_is_an_architecture_violation() {
        let checker = Checker {
            layers: Layers {
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
            },
        };
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            layer: Some(String::from("product")),
            ..Pack::default()
        };
        let referencing_pack = Pack {
            name: String::from("packs/bar"),
            layer: Some(String::from("utilities")),
            enforce_architecture: CheckerSetting::True,
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/bar/app/services/bar.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/foo/app/services/foo.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![defining_pack, referencing_pack]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        let expected_violation = Violation {
            message: String::from("packs/bar/app/services/bar.rb:3:1\nArchitecture violation: `::Foo` belongs to `packs/foo` (whose layer is `product`) cannot be accessed from `packs/bar` (whose layer is `utilities`)"),
            identifier: ViolationIdentifier {
                violation_type: String::from("architecture"),
                file: String::from("packs/bar/app/services/bar.rb"),
                constant_name: String::from("::Foo"),
                referencing_pack_name: String::from("packs/bar"),
                defining_pack_name: String::from("packs/foo"),
            },
        };
        assert_eq!(
            expected_violation,
            checker.check(&reference, &configuration).unwrap()
        )
    }

    #[test]
    fn reference_is_not_an_architecture_violation() {
        let checker = Checker {
            layers: Layers {
                layers: vec![
                    String::from("product"),
                    String::from("utilities"),
                ],
            },
        };
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            layer: Some(String::from("utilities")),
            ..Pack::default()
        };
        let referencing_pack = Pack {
            name: String::from("packs/bar"),
            layer: Some(String::from("product")),
            enforce_architecture: CheckerSetting::True,
            ..Pack::default()
        };

        let reference = Reference {
            constant_name: String::from("::Foo"),
            defining_pack_name: Some(defining_pack.name.to_owned()),
            referencing_pack_name: referencing_pack.name.to_owned(),
            relative_referencing_file: String::from(
                "packs/bar/app/services/bar.rb",
            ),
            relative_defining_file: Some(String::from(
                "packs/foo/app/services/foo.rb",
            )),
            source_location: SourceLocation { line: 3, column: 1 },
        };

        let configuration = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![defining_pack, referencing_pack]),
                HashMap::new(),
            ),
            ..Configuration::default()
        };

        assert_eq!(None, checker.check(&reference, &configuration))
    }
}
