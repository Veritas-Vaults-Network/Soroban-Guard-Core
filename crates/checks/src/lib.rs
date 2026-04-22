//! Vulnerability detectors for Soroban smart contracts.

pub mod admin;
pub mod auth;
pub mod contracttype;
pub mod overflow;
pub mod panic_usage;
pub mod storage;
pub mod unbounded_storage;
pub mod zero_amount;
mod util;

pub use admin::UnprotectedAdminCheck;
pub use auth::MissingRequireAuthCheck;
pub use contracttype::MissingContracttypeCheck;
pub use overflow::UncheckedArithmeticCheck;
pub use panic_usage::PanicUsageCheck;
pub use storage::UnsafeStoragePatternsCheck;
pub use unbounded_storage::UnboundedStorageCheck;
pub use zero_amount::ZeroAmountCheck;

use serde::Serialize;
use syn::File;

/// Severity of a finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// One issue reported by a check.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Finding {
    pub check_name: String,
    pub severity: Severity,
    pub file_path: String,
    pub line: usize,
    pub function_name: String,
    pub description: String,
}

/// A static analyzer check implemented against a parsed `syn::File`.
pub trait Check {
    fn name(&self) -> &str;
    fn run(&self, file: &File, source: &str) -> Vec<Finding>;
}

/// All checks executed by the analyzer (extend here as you add detectors).
///
/// Checks are **stateless and isolated**: implementations must not use shared mutable
/// static state or assume a particular invocation order. The analyzer runs each check
/// against the same parsed `syn::File` independently and concatenates `Finding`s.
pub fn default_checks() -> Vec<Box<dyn Check + Send + Sync>> {
    vec![
        Box::new(MissingRequireAuthCheck),
        Box::new(UncheckedArithmeticCheck),
        Box::new(UnprotectedAdminCheck),
        Box::new(UnsafeStoragePatternsCheck),
        Box::new(PanicUsageCheck),
        Box::new(MissingContracttypeCheck),
        Box::new(UnboundedStorageCheck),
        Box::new(ZeroAmountCheck),
    ]
}
