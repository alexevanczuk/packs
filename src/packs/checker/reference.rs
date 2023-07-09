use crate::packs::{pack::Pack, SourceLocation};

#[derive(Debug)]
pub struct Reference<'a> {
    pub constant_name: String,
    pub defining_pack: Option<&'a Pack>,
    pub relative_defining_file: Option<String>,
    pub referencing_pack: &'a Pack,
    pub relative_referencing_file: String,
    pub source_location: SourceLocation,
}
