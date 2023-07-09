use super::parsing::ruby::zeitwerk::constant_resolver::ConstantDefinition;

pub trait ConstantResolver {
    fn resolve(
        &self,
        fully_or_partially_qualified_constant: &str,
        namespace_path: &[&str],
    ) -> Option<ConstantDefinition>;
}
