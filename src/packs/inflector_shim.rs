use inflector::cases::classcase::to_class_case as buggy_to_class_case;
use regex::Regex;

// See https://github.com/whatisinternet/Inflector/pull/87
pub fn to_class_case(s: &str) -> String {
    let mut class_name = buggy_to_class_case(s);
    if class_name.contains("Statu") {
        let re = Regex::new("Statuse$").unwrap();
        class_name = re.replace_all(&class_name, "Status").to_string();
        let re = Regex::new("Statu$").unwrap();

        class_name = re.replace_all(&class_name, "Status").to_string();

        let re = Regex::new("Statuss").unwrap();
        re.replace_all(&class_name, "Status").to_string();
    }

    if class_name.contains("Daum") {
        let re = Regex::new("Daum").unwrap();
        class_name = re.replace_all(&class_name, "Datum").to_string();
    }

    if class_name.contains("Lefe") {
        let re = Regex::new("Lefe").unwrap();
        class_name = re.replace_all(&class_name, "Leave").to_string();
    }

    class_name
}
