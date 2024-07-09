use std::collections::{HashMap, HashSet};

use regex::Regex;
use ruby_inflector::case::{
    to_case_camel_like, to_class_case as to_class_case_original, CamelOptions,
};

// This is a list of plural to singular words that are not handled by the inflector
// The plural words are
const CLASS_CASE_TO_SINGULAR: [(&str, &str); 4] = [
    ("Censuse", "Census"),
    ("Leafe", "Leave"),
    ("Lefe", "Leave"),
    ("Daum", "Datum"),
];

// See https://github.com/whatisinternet/Inflector/pull/87
// Note that as of the PR that adds this comment, we are now using https://github.com/alexevanczuk/ruby_inflector,
// so that we have an easier time making this inflector more specific to ruby applications (for now)
pub fn to_class_case(
    s: &str,
    should_singularize: bool,
    acronyms: &HashSet<String>,
) -> String {
    let options = CamelOptions {
        new_word: true,
        last_char: ' ',
        first_word: false,
        injectable_char: ' ',
        has_seperator: false,
        inverted: false,
    };

    let mut class_name = if should_singularize {
        to_class_case_original(s, acronyms)
    } else {
        to_case_camel_like(s, options, acronyms)
    };

    if class_name.contains("Statu") {
        let re = Regex::new("Statuse$").unwrap();
        class_name = re.replace_all(&class_name, "Status").to_string();
        let re = Regex::new("Statu$").unwrap();

        class_name = re.replace_all(&class_name, "Status").to_string();

        let re = Regex::new("Statuss").unwrap();
        re.replace_all(&class_name, "Status").to_string();
    }

    CLASS_CASE_TO_SINGULAR
        .into_iter()
        .for_each(|(plural, singular)| {
            if class_name.contains(plural) {
                let re = Regex::new(plural).unwrap();
                class_name = re.replace_all(&class_name, singular).to_string();
            }
        });

    class_name
}

pub fn camelize(s: &str, acronyms: &HashSet<String>) -> String {
    // Meant to emulate https://github.com/rails/rails/blob/e88857bbb9d4e1dd64555c34541301870de4a45b/activesupport/lib/active_support/inflector/methods.rb#L69
    //
    // def camelize(term, uppercase_first_letter = true)
    //   string = term.to_s
    //   # String#camelize takes a symbol (:upper or :lower), so here we also support :lower to keep the methods consistent.
    //   if !uppercase_first_letter || uppercase_first_letter == :lower
    //     string = string.sub(inflections.acronyms_camelize_regex) { |match| match.downcase! || match }
    //   else
    //     string = string.sub(/^[a-z\d]*/) { |match| inflections.acronyms[match] || match.capitalize! || match }
    //   end
    //   string.gsub!(/(?:_|(\/))([a-z\d]*)/i) do
    //     word = $2
    //     substituted = inflections.acronyms[word] || word.capitalize! || word
    //     $1 ? "::#{substituted}" : substituted
    //   end
    //   string
    // end

    let lowercase_acronyms_to_originals = acronyms
        .iter()
        .map(|acronym| (acronym.to_lowercase(), acronym))
        .collect::<HashMap<String, &String>>();

    let mut new_string = s.to_string();
    // Replace the beginning of the word, matched with lowercase letters, with either a matching inflection or a capitalized version of the word
    let re = Regex::new("^[a-z\\d]*").unwrap();
    new_string = re
        .replace(&new_string, |caps: &regex::Captures| {
            let word = caps.get(0).unwrap().as_str();
            if lowercase_acronyms_to_originals.contains_key(word) {
                lowercase_acronyms_to_originals[word].to_string()
            } else {
                capitalize(word)
            }
        })
        .to_mut()
        .to_string();

    let re = Regex::new("(?:_|(/))([a-z\\d]*)").unwrap();

    new_string = re
        .replace_all(&new_string, |caps: &regex::Captures| {
            let matched_slash = caps.get(1);
            let word = caps.get(2).unwrap().as_str();
            let capitalized_word =
                if lowercase_acronyms_to_originals.contains_key(word) {
                    lowercase_acronyms_to_originals[word].to_string()
                } else {
                    capitalize(word)
                };

            if matched_slash.is_some() {
                format!("::{}", capitalized_word)
            } else {
                capitalized_word
            }
        })
        .to_mut()
        .to_string();

    new_string
}

/// Capitalizes the first character in s.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivial() {
        let actual = to_class_case("my_string", false, &HashSet::new());
        let expected = "MyString";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_digits() {
        let actual =
            to_class_case("my_string_401k_thing", false, &HashSet::new());
        let expected = "MyString401kThing";
        assert_eq!(expected, actual);
    }

    #[test]
    fn fn_test_camelizing_case_retained() {
        let mut acronyms = HashSet::new();
        acronyms.insert(String::from("FacTory"));

        let actual = camelize("my_factory", &acronyms);
        let expected = "MyFacTory";
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_to_class_case() {
        let tests = vec![
            ("my_string", false, "MyString"),
            ("censuses", true, "Census"),
            ("lefe", true, "Leave"),
            ("leaves", false, "Leaves"),
            ("daum", true, "Datum"),
            ("statuss", false, "Statuss"),
            ("statuses", true, "Status"),
            ("censuse", true, "Census"),
        ];

        for (input, should_singularize, expected) in tests {
            let actual =
                to_class_case(input, should_singularize, &HashSet::new());
            assert_eq!(
                expected, actual,
                "Failed for input: {}, and singularize: {}",
                input, should_singularize
            );
        }
    }
}
