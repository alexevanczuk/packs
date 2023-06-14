use serde::Deserialize;
use std::collections::HashMap;

#[derive(PartialEq, Debug, Deserialize)]
struct ViolationGroup {
    violations: Vec<String>,
    files: Vec<String>,
}

#[derive(PartialEq, Debug, Deserialize)]
struct PackageTodo {
    #[serde(flatten)]
    violations_by_pack: HashMap<String, HashMap<String, ViolationGroup>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_trivial_case() {
        let contents: String = String::from(
            "
        # This file contains a list of dependencies that are not part of the long term plan for the
        # 'packs/foo' package.
        # We should generally work to reduce this list over time.
        #
        # You can regenerate this file using the following command:
        #
        # bin/packwerk update-todo
        packs/bar:
            \"::Bar\":
                violations:
                - dependency
                files:
                - packs/foo/app/services/foo.rb
            \"::Baz\":
                violations:
                - dependency
                - privacy
                files:
                - packs/foo/app/services/foo.rb
        ",
        );

        let mut violations_by_pack: HashMap<
            String,
            HashMap<String, ViolationGroup>,
        > = HashMap::new();
        let mut bar_violations = HashMap::new();
        bar_violations.insert(
            String::from("::Bar"),
            ViolationGroup {
                violations: vec![String::from("dependency")],
                files: vec![String::from("packs/foo/app/services/foo.rb")],
            },
        );
        bar_violations.insert(
            String::from("::Baz"),
            ViolationGroup {
                violations: vec![
                    String::from("dependency"),
                    String::from("privacy"),
                ],
                files: vec![String::from("packs/foo/app/services/foo.rb")],
            },
        );
        violations_by_pack.insert(String::from("packs/bar"), bar_violations);
        let expected = PackageTodo { violations_by_pack };

        let actual: PackageTodo = serde_yaml::from_str(&contents).unwrap();
        assert_eq!(expected, actual);
    }
}
