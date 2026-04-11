//! Unchecked arithmetic (Phase 2).

use crate::{Check, Finding};
use syn::File;

pub struct UncheckedArithmeticCheck;

impl Check for UncheckedArithmeticCheck {
    fn name(&self) -> &str {
        "unchecked-arithmetic"
    }

    fn run(&self, _file: &File, _source: &str) -> Vec<Finding> {
        vec![]
    }
}
