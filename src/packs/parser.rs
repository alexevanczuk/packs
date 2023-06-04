pub(crate) mod ruby;
pub(crate) use ruby::packwerk::extractor::extract_from_path as extract_from_ruby_path;
pub(crate) mod erb;
#[allow(unused_imports)]
pub(crate) use erb::packwerk::extractor::extract_from_path as extract_from_erb_path;

// TODO: Move this somewhere else
pub(crate) use ruby::packwerk::extractor::UnresolvedReference;
