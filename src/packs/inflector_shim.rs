use regex::Regex;
use ruby_inflector::{
    case::{to_case_camel_like, CamelOptions},
    Inflector,
};

// See https://github.com/whatisinternet/Inflector/pull/87
// Note that as of the PR that adds this comment, we are now using https://github.com/alexevanczuk/ruby_inflector,
// so that we have an easier time making this inflector more specific to ruby applications (for now)
pub fn to_class_case(s: &str, should_singularize: bool) -> String {
    let options = CamelOptions {
        new_word: true,
        last_char: ' ',
        first_word: false,
        injectable_char: ' ',
        has_seperator: false,
        inverted: false,
    };

    let mut class_name = if should_singularize {
        s.to_class_case()
    } else {
        to_case_camel_like(s, options)
    };

    // let mut class_name = s.to_class_case();
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

    if class_name.contains("Leafe") {
        let re = Regex::new("Leafe").unwrap();
        class_name = re.replace_all(&class_name, "Leave").to_string();
    }

    class_name
}

// Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial() {
        let actual = to_class_case("my_string", false);
        let expected = "MyString";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_digits() {
        let actual = to_class_case("my_string_401k_thing", false);
        let expected = "MyString401KThing";
        assert_eq!(expected, actual);
    }
}
