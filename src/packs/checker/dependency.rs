use crate::packs::checker::Reference;
use crate::packs::Violation;

pub struct Checker {}

impl Checker {
    pub fn check(&self, reference: &Reference) -> Option<Violation> {
        None
    }
}
