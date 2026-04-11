//! Unsafe persistent storage patterns (Phase 2).

use crate::{Check, Finding};
use syn::File;

pub struct UnsafeStoragePatternsCheck;

impl Check for UnsafeStoragePatternsCheck {
    fn name(&self) -> &str {
        "unsafe-storage-patterns"
    }

    fn run(&self, _file: &File, _source: &str) -> Vec<Finding> {
        vec![]
    }
}
