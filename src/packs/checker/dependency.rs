use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

#[allow(unused_variables)]
impl Checker {
    pub fn check(&self, reference: &Reference) -> Option<Violation> {
        None
    }
}
