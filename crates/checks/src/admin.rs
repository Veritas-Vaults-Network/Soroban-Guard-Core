//! Unprotected admin / owner entrypoints (Phase 2).

use crate::{Check, Finding};
use syn::File;

pub struct UnprotectedAdminCheck;

impl Check for UnprotectedAdminCheck {
    fn name(&self) -> &str {
        "unprotected-admin"
    }

    fn run(&self, _file: &File, _source: &str) -> Vec<Finding> {
        vec![]
    }
}
