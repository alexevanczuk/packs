use std::collections::HashMap;

use crate::packs::{
    pack::{CheckerSetting, Pack},
    Configuration,
};

use super::{
    output_helper::print_reference_location, reference::Reference, Violation,
    ViolationIdentifier,
};

pub struct PackChecker<'a> {
    pub configuration: &'a Configuration,
    pub referencing_pack: &'a Pack,
    pub defining_pack: Option<&'a Pack>,
    pub violation_type: ViolationType,
    pub reference: &'a Reference,
}

enum ViolationDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy)]
pub enum ViolationType {
    Dependency,
    FolderPrivacy,
    Layer,
    Privacy,
    Visibility,
}

impl From<&str> for ViolationType {
    fn from(s: &str) -> Self {
        match s {
            "dependency" => ViolationType::Dependency,
            "folder_privacy" => ViolationType::FolderPrivacy,
            "layer" => ViolationType::Layer,
            "privacy" => ViolationType::Privacy,
            "visibility" => ViolationType::Visibility,
            _ => panic!("unknown violation type: {}", s),
        }
    }
}

impl From<ViolationType> for &str {
    fn from(violation_type: ViolationType) -> &'static str {
        match violation_type {
            ViolationType::Dependency => "dependency",
            ViolationType::FolderPrivacy => "folder_privacy",
            ViolationType::Layer => "layer",
            ViolationType::Privacy => "privacy",
            ViolationType::Visibility => "visibility",
        }
    }
}

impl<'a> PackChecker<'a> {
    pub fn new(
        configuration: &'a Configuration,
        reference: &'a Reference,
        violation_type: &str,
    ) -> anyhow::Result<Self> {
        let pack_set = &configuration.pack_set;
        Ok(Self {
            configuration,
            referencing_pack: reference.referencing_pack(pack_set)?,
            defining_pack: reference.defining_pack(pack_set)?,
            violation_type: ViolationType::from(violation_type),
            reference,
        })
    }

    fn violation_direction(&self) -> ViolationDirection {
        match self.violation_type {
            ViolationType::Dependency | ViolationType::Layer => {
                ViolationDirection::Outgoing
            }
            ViolationType::Privacy
            | ViolationType::FolderPrivacy
            | ViolationType::Visibility => ViolationDirection::Incoming,
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
        match self.violation_type {
            ViolationType::Dependency => self
                .checker_setting_for(&self.rules_pack().enforce_dependencies),
            ViolationType::FolderPrivacy => {
                self.rules_pack().enforce_folder_privacy()
            }
            ViolationType::Layer => {
                self.checker_setting_for(&self.rules_pack().enforce_layers)
            }
            ViolationType::Privacy => {
                self.checker_setting_for(&self.rules_pack().enforce_privacy)
            }
            ViolationType::Visibility => {
                self.checker_setting_for(&self.rules_pack().enforce_visibility)
            }
        }
    }

    fn violation_globally_disabled(&self) -> bool {
        match self.violation_type {
            ViolationType::Dependency => {
                self.configuration.disable_enforce_dependencies
            }
            ViolationType::FolderPrivacy => {
                self.configuration.disable_enforce_folder_privacy
            }
            ViolationType::Layer => self.configuration.disable_enforce_layers,
            ViolationType::Privacy => {
                self.configuration.disable_enforce_privacy
            }
            ViolationType::Visibility => {
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
            .is_ignored(file_path, self.violation_type.into())
    }

    pub fn violation(&self, msg: &str) -> anyhow::Result<Option<Violation>> {
        Ok(Some(Violation {
            message: self.interpolate_violation_message(msg),
            identifier: self.violation_identifier(),
        }))
    }

    pub fn violation_identifier(&self) -> ViolationIdentifier {
        let violation_type: &str = self.violation_type.into();
        ViolationIdentifier {
            violation_type: violation_type.to_string(),
            strict: self.is_strict(),
            file: self.reference.relative_referencing_file.clone(),
            constant_name: self.reference.constant_name.clone(),
            referencing_pack_name: self.referencing_pack.name.clone(),
            defining_pack_name: self.defining_pack.unwrap().name.clone(),
        }
    }

    fn interpolate_violation_message(&self, msg: &str) -> String {
        let mut map = HashMap::new();
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

        let mut interpolated_msg = msg.to_string();
        for (key, value) in &map {
            interpolated_msg = interpolated_msg.replace(key, value);
        }
        interpolated_msg
    }
}
