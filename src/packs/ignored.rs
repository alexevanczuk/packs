use std::collections::HashSet;

pub fn is_ignored(rules: &HashSet<String>, path: &str) -> anyhow::Result<bool> {
    let (allow_list, deny_list): (HashSet<&String>, HashSet<&String>) =
        rules.iter().partition(|rule| rule.starts_with('!'));

    // allow-list (starts with !) takes precedence over deny-list (does not start with !
    if allow_list.iter().any(|rule| is_match(&rule[1..], path)) {
        return Ok(false);
    }
    if deny_list.iter().any(|rule| is_match(rule, path)) {
        return Ok(true);
    }

    Ok(false)
}

fn is_match(rule: &str, path: &str) -> bool {
    match fnmatch_regex2::glob_to_regex(rule) {
        Ok(regex) => regex.is_match(path),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! test_ignore {
        ($name:ident, $rules:expr, $path:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(
                    is_ignored($rules, $path).unwrap(),
                    $expected,
                    "Testing path: {}",
                    $path
                );
            }
        };
    }

    #[macro_export]
    macro_rules! ignored {
        ($name:ident, $rules:expr, $path:expr) => {
            test_ignore!($name, $rules, $path, true);
        };
    }

    #[macro_export]
    macro_rules! not_ignored {
        ($name:ident, $rules:expr, $path:expr) => {
            test_ignore!($name, $rules, $path, false);
        };
    }

    ignored!(
        foo1,
        &HashSet::from(["packs/foo/**/*".to_string()]),
        "packs/foo/app/services/my.rb"
    );
    ignored!(
        foo2,
        &HashSet::from(["**/*".to_string()]),
        "logs/monday/foo.bar"
    );

    not_ignored!(
        nofoo1,
        &HashSet::from(["*/**".to_string(), "!packs/foo/**".to_string()]),
        "packs/foo/app/services/my.rb"
    );

    #[test]
    fn test_is_match() {
        assert!(is_match("foo", "foo"));
        assert!(!is_match("foo", "bar"));
        assert!(is_match("foo*", "foobar"));
        assert!(is_match("packs/foo/**", "packs/foo/app/services/my.rb"));
        assert!(is_match("packs/foo/**/*", "packs/foo/app/services/my.rb"));
    }
}
