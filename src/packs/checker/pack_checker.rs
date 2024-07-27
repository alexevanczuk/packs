use std::collections::HashMap;

use crate::packs::{
    checker_configuration::{CheckerConfiguration, CheckerType},
    pack::{CheckerSetting, Pack},
    Configuration,
};

use super::{
    output_helper::print_reference_location, reference::Reference, Violation,
    ViolationIdentifier,
};

pub struct PackChecker<'a> {
    pub configuration: &'a Configuration,
    pub checker_configuration: &'a CheckerConfiguration,
    pub checker_type: CheckerType,
    pub referencing_pack: &'a Pack,
    pub defining_pack: Option<&'a Pack>,
    pub reference: &'a Reference,
}

#[derive(Debug, PartialEq)]
enum ViolationDirection {
    Incoming,
    Outgoing,
}

impl<'a> PackChecker<'a> {
    pub fn new(
        configuration: &'a Configuration,
        checker_type: CheckerType,
        reference: &'a Reference,
    ) -> anyhow::Result<Self> {
        let pack_set = &configuration.pack_set;
        let checker_configuration =
            &configuration.checker_configuration[&checker_type];
        Ok(Self {
            configuration,
            checker_configuration,
            checker_type,
            referencing_pack: reference.referencing_pack(pack_set)?,
            defining_pack: reference.defining_pack(pack_set)?,
            reference,
        })
    }

    fn violation_direction(&self) -> ViolationDirection {
        match self.checker_type {
            CheckerType::Dependency | CheckerType::Layer => {
                ViolationDirection::Outgoing
            }
            CheckerType::Privacy
            | CheckerType::FolderPrivacy
            | CheckerType::Visibility => ViolationDirection::Incoming,
        }
    }

    pub fn checkable(&self) -> anyhow::Result<bool> {
        if self.defining_pack.is_none() {
            return Ok(false);
        }
        if self.defining_pack_name() == self.referencing_pack_name() {
            return Ok(false);
        }
        if self.rules_checker_setting().is_false() {
            return Ok(false);
        }
        if self.violation_globally_disabled() {
            return Ok(false);
        }
        if self.is_ignored()? {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn is_strict(&self) -> bool {
        self.rules_checker_setting().is_strict()
    }

    fn defining_pack_name(&self) -> &str {
        &self.defining_pack.as_ref().unwrap().name
    }

    fn referencing_pack_name(&self) -> &str {
        &self.referencing_pack.name
    }

    fn rules_checker_setting(&self) -> &CheckerSetting {
        match self.checker_type {
            CheckerType::Dependency => self
                .checker_setting_for(&self.rules_pack().enforce_dependencies),
            CheckerType::FolderPrivacy => {
                self.rules_pack().enforce_folder_privacy()
            }
            CheckerType::Layer => {
                self.checker_setting_for(&self.rules_pack().enforce_layers)
            }
            CheckerType::Privacy => {
                self.checker_setting_for(&self.rules_pack().enforce_privacy)
            }
            CheckerType::Visibility => {
                self.checker_setting_for(&self.rules_pack().enforce_visibility)
            }
        }
    }

    fn violation_globally_disabled(&self) -> bool {
        match self.checker_type {
            CheckerType::Dependency => {
                self.configuration.disable_enforce_dependencies
            }
            CheckerType::FolderPrivacy => {
                self.configuration.disable_enforce_folder_privacy
            }
            CheckerType::Layer => self.configuration.disable_enforce_layers,
            CheckerType::Privacy => self.configuration.disable_enforce_privacy,
            CheckerType::Visibility => {
                self.configuration.disable_enforce_visibility
            }
        }
    }

    fn checker_setting_for(
        &self,
        checker_setting: &'a Option<CheckerSetting>,
    ) -> &'a CheckerSetting {
        match checker_setting {
            Some(setting) => setting,
            None => &CheckerSetting::False,
        }
    }

    fn rules_pack(&self) -> &Pack {
        match self.violation_direction() {
            ViolationDirection::Outgoing => self.referencing_pack,
            ViolationDirection::Incoming => {
                self.defining_pack.as_ref().unwrap()
            }
        }
    }

    fn is_ignored(&self) -> anyhow::Result<bool> {
        let file_path = match self.violation_direction() {
            ViolationDirection::Incoming => {
                &self.reference.relative_referencing_file
            }
            ViolationDirection::Outgoing => {
                self.reference.relative_defining_file.as_ref().unwrap()
            }
        };
        self.rules_pack()
            .is_ignored(file_path, &self.checker_configuration.checker_name())
    }

    pub fn violation(
        &self,
        extra_template_fields: Option<HashMap<&str, String>>,
    ) -> anyhow::Result<Option<Violation>> {
        Ok(Some(Violation {
            message: self.interpolate_violation_message(extra_template_fields),
            identifier: self.violation_identifier(),
        }))
    }

    pub fn violation_identifier(&self) -> ViolationIdentifier {
        let violation_type: &str = &self.checker_configuration.checker_name();
        ViolationIdentifier {
            violation_type: violation_type.to_string(),
            strict: self.is_strict(),
            file: self.reference.relative_referencing_file.clone(),
            constant_name: self.reference.constant_name.clone(),
            referencing_pack_name: self.referencing_pack.name.clone(),
            defining_pack_name: self.defining_pack.unwrap().name.clone(),
        }
    }

    fn interpolate_violation_message(
        &self,
        extra_template_fields: Option<HashMap<&str, String>>,
    ) -> String {
        let mut map = extra_template_fields.unwrap_or_default();
        map.insert(
            "{{violation_name}}",
            self.checker_configuration.pretty_checker_name(),
        );
        map.insert(
            "{{referencing_pack_name}}",
            self.referencing_pack.name.clone(),
        );
        map.insert(
            "{{defining_pack_name}}",
            self.defining_pack.unwrap().name.clone(),
        );
        map.insert("{{constant_name}}", self.reference.constant_name.clone());
        map.insert(
            "{{reference_location}}",
            print_reference_location(self.reference),
        );
        map.insert(
            "{{referencing_pack_relative_yml}}",
            self.referencing_pack
                .relative_yml()
                .to_string_lossy()
                .to_string(),
        );

        let mut interpolated_msg =
            self.checker_configuration.checker_error_template();
        for (key, value) in &map {
            interpolated_msg = interpolated_msg.replace(key, value);
        }
        interpolated_msg
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::packs::{PackSet, SourceLocation};

    use super::*;

    fn build_config_refer() -> (Configuration, Reference) {
        let root_pack = Pack {
            name: String::from("."),
            ..Pack::default()
        };
        let defining_pack = Pack {
            name: String::from("packs/foo"),
            ..Pack::default()
        };
        let referencing_pack_bar = Pack {
            name: String::from("packs/bar"),
            ..Pack::default()
        };
        let referencing_pack_baz = Pack {
            name: String::from("packs/baz"),
            dependencies: HashSet::from_iter(vec![String::from("packs/foo")]),
            ..Pack::default()
        };
        let mut checker_map = HashMap::new();
        checker_map.insert(
            CheckerType::Dependency,
            CheckerConfiguration::new(CheckerType::Dependency),
        );
        checker_map.insert(
            CheckerType::Privacy,
            CheckerConfiguration::new(CheckerType::Privacy),
        );
        checker_map.insert(
            CheckerType::FolderPrivacy,
            CheckerConfiguration::new(CheckerType::FolderPrivacy),
        );
        checker_map.insert(
            CheckerType::Visibility,
            CheckerConfiguration::new(CheckerType::Visibility),
        );
        checker_map.insert(
            CheckerType::Layer,
            CheckerConfiguration::new(CheckerType::Layer),
        );

        let config = Configuration {
            pack_set: PackSet::build(
                HashSet::from_iter(vec![
                    root_pack,
                    defining_pack,
                    referencing_pack_bar,
                    referencing_pack_baz,
                ]),
                HashMap::new(),
            )
            .unwrap(),
            checker_configuration: checker_map,
            ..Configuration::default()
        };

        let refer = Reference {
            constant_name: "Foo".into(),
            defining_pack_name: Some("packs/foo".into()),
            relative_defining_file: None,
            referencing_pack_name: "packs/baz".into(),
            relative_referencing_file: "packs/baz/public/baz.rb".into(),
            source_location: SourceLocation {
                line: 3usize,
                column: 4usize,
            },
        };
        (config, refer)
    }

    #[test]
    fn folder_privacy_test() -> anyhow::Result<()> {
        let (config, refer) = build_config_refer();
        let checker =
            PackChecker::new(&config, CheckerType::FolderPrivacy, &refer)?;

        assert_eq!(checker.violation_direction(), ViolationDirection::Incoming);
        assert!(!checker.is_strict());
        assert_eq!(checker.defining_pack_name(), "packs/foo".to_string());
        assert_eq!(checker.referencing_pack_name(), "packs/baz".to_string());
        assert_eq!(checker.rules_checker_setting(), &CheckerSetting::False);
        assert!(!checker.violation_globally_disabled());
        let expected_violation_message: String = "\u{1b}[36mpacks/baz/public/baz.rb\u{1b}[0m:3:4\nFolder Privacy violation: `Foo` belongs to `packs/foo`, which is private to `packs/baz` as it is not a sibling pack or parent pack.".into();
        assert_eq!(
            checker.interpolate_violation_message(None),
            expected_violation_message
        );

        let mut config = config;
        let mut privacy_checker =
            CheckerConfiguration::new(CheckerType::FolderPrivacy);
        privacy_checker.override_error_template =
            Some("{{violation_name}}".into());
        config
            .checker_configuration
            .insert(CheckerType::FolderPrivacy, privacy_checker);
        let checker =
            PackChecker::new(&config, CheckerType::FolderPrivacy, &refer)?;
        assert_eq!(
            checker.interpolate_violation_message(None),
            "Folder Privacy".to_string()
        );

        Ok(())
    }

    #[test]
    fn privacy_test() -> anyhow::Result<()> {
        let (config, refer) = build_config_refer();
        let checker = PackChecker::new(&config, CheckerType::Privacy, &refer)?;

        assert_eq!(checker.violation_direction(), ViolationDirection::Incoming);
        let expected_violation_message: String = "\u{1b}[36mpacks/baz/public/baz.rb\u{1b}[0m:3:4\nPrivacy violation: `Foo` is private to `packs/foo`, but referenced from `packs/baz`".into();
        assert_eq!(
            checker.interpolate_violation_message(None),
            expected_violation_message
        );

        Ok(())
    }

    #[test]
    fn dependency_test() -> anyhow::Result<()> {
        let (config, refer) = build_config_refer();
        let checker =
            PackChecker::new(&config, CheckerType::Dependency, &refer)?;

        assert_eq!(checker.violation_direction(), ViolationDirection::Outgoing);
        let expected_violation_message: String = "\u{1b}[36mpacks/baz/public/baz.rb\u{1b}[0m:3:4\nDependency violation: `Foo` belongs to `packs/foo`, but `package.yml` does not specify a dependency on `packs/foo`.".into();
        assert_eq!(
            checker.interpolate_violation_message(None),
            expected_violation_message
        );

        Ok(())
    }

    #[test]
    fn layer_test() -> anyhow::Result<()> {
        let (config, refer) = build_config_refer();
        let checker = PackChecker::new(&config, CheckerType::Layer, &refer)?;

        assert_eq!(checker.violation_direction(), ViolationDirection::Outgoing);
        let expected_violation_message: String = "\u{1b}[36mpacks/baz/public/baz.rb\u{1b}[0m:3:4\nLayer violation: `Foo` belongs to `packs/foo` (whose layer is `{{defining_layer}}`) cannot be accessed from `packs/baz` (whose layer is `{{referencing_layer}}`)".into();
        assert_eq!(
            checker.interpolate_violation_message(None),
            expected_violation_message
        );

        Ok(())
    }

    #[test]
    fn visibility_test() -> anyhow::Result<()> {
        let (config, refer) = build_config_refer();
        let checker =
            PackChecker::new(&config, CheckerType::Visibility, &refer)?;

        assert_eq!(checker.violation_direction(), ViolationDirection::Incoming);
        let expected_violation_message: String = "\u{1b}[36mpacks/baz/public/baz.rb\u{1b}[0m:3:4\nVisibility violation: `Foo` belongs to `packs/foo`, which is not visible to `packs/baz`".into();
        assert_eq!(
            checker.interpolate_violation_message(None),
            expected_violation_message
        );

        Ok(())
    }
}
