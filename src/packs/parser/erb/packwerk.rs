pub(crate) mod extractor;

#[cfg(test)]
mod tests {
    use crate::packs::parser::erb::packwerk::extractor::extract_from_contents;
    use crate::packs::Range;
    use crate::packs::UnresolvedReference;

    #[test]
    fn trivial_case() {
        let contents: String = String::from("<%= Foo %>");
        assert_eq!(
            vec![UnresolvedReference {
                name: String::from("Foo"),
                namespace_path: vec![],
                location: Range::default()
            }],
            extract_from_contents(contents)
        );
    }
}
