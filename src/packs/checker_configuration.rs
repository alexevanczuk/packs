#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CheckerType {
    Dependency,
    Privacy,
    FolderPrivacy,
    Layer,
    Visibility,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CheckerConfiguration {
    pub checker_type: CheckerType,
    pub override_error_template: Option<String>,
}

const DEFAULT_DEPENDENCY_TEMPLATE: &str = "{{reference_location}}Dependency violation: `{{constant_name}}` belongs to `{{defining_pack_name}}`, but `{{referencing_pack_relative_yml}}` does not specify a dependency on `{{defining_pack_name}}`.";
const DEFAULT_FOLDER_PRIVACY_TEMPLATE: &str = "{{reference_location}}{{violation_name}} violation: `{{constant_name}}` belongs to `{{defining_pack_name}}`, which is private to `{{referencing_pack_name}}` as it is not a sibling pack or parent pack.";
const DEFAULT_LAYER_TEMPLATE: &str = "{{reference_location}}Layer violation: `{{constant_name}}` belongs to `{{defining_pack_name}}` (whose layer is `{{defining_layer}}`) cannot be accessed from `{{referencing_pack_name}}` (whose layer is `{{referencing_layer}}`)";
const DEFAULT_VISIBILITY_TEMPLATE: &str = "{{reference_location}}Visibility violation: `{{constant_name}}` belongs to `{{defining_pack_name}}`, which is not visible to `{{referencing_pack_name}}`";
const DEFAULT_PRIVACY_TEMPLATE: &str = "{{reference_location}}Privacy violation: `{{constant_name}}` is private to `{{defining_pack_name}}`, but referenced from `{{referencing_pack_name}}`";

impl CheckerConfiguration {
    pub fn new(checker_type: CheckerType) -> Self {
        Self {
            checker_type,
            override_error_template: None,
        }
    }

    pub fn pretty_checker_name(&self) -> String {
        self.checker_name()
            .split('_')
            .map(CheckerConfiguration::capitalize)
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn capitalize(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    pub fn checker_name(&self) -> String {
        self.default_checker_name()
    }

    pub fn checker_error_template(&self) -> String {
        self.override_error_template
            .clone()
            .unwrap_or_else(|| self.default_error_template())
    }

    fn default_error_template(&self) -> String {
        match self.checker_type {
            CheckerType::Dependency => DEFAULT_DEPENDENCY_TEMPLATE.into(),
            CheckerType::FolderPrivacy => {
                DEFAULT_FOLDER_PRIVACY_TEMPLATE.into()
            }
            CheckerType::Layer => DEFAULT_LAYER_TEMPLATE.into(),
            CheckerType::Visibility => DEFAULT_VISIBILITY_TEMPLATE.into(),
            CheckerType::Privacy => DEFAULT_PRIVACY_TEMPLATE.into(),
        }
    }

    fn default_checker_name(&self) -> String {
        match self.checker_type {
            CheckerType::Dependency => "dependency".into(),
            CheckerType::FolderPrivacy => "folder_privacy".into(),
            CheckerType::Layer => "layer".into(),
            CheckerType::Visibility => "visibility".into(),
            CheckerType::Privacy => "privacy".into(),
        }
    }
}
