use super::reference::Reference;

pub fn print_reference_location(reference: &Reference) -> String {
    format!(
        "\x1b[36m{}\x1b[0m:{}:{}\n",
        reference.relative_referencing_file,
        reference.source_location.line,
        reference.source_location.column,
    )
}
