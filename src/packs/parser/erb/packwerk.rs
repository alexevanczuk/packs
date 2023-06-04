pub(crate) mod extractor;

#[cfg(test)]
mod tests {
    use crate::packs::parser::erb::packwerk::extractor::extract_from_contents;
    use crate::packs::Range;
    use crate::packs::UnresolvedReference;

    #[test]
    #[ignore]
    fn trivial_case() {
        let contents: String = String::from("<%= Foo %>");
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range {
                    start_row: 1,
                    start_col: 0,
                    end_row: 1,
                    end_col: 4
                }
            }],
            extract_from_contents(contents)
        );
    }
}
