//! Vulnerability detectors for Soroban smart contracts.

pub mod admin;
pub mod admin_in_temp;
pub mod dynamic_symbol_key;
pub mod instance_vec_growth;
pub mod invoke_unchecked_cast;
pub mod migration_guard;
pub mod negative_deposit;
pub mod no_param_no_auth;
pub mod storage_type_confusion;
pub mod withdraw_auth;
pub mod admin_key_removal;
pub mod admin_overwrite;
pub mod assert_for_auth;
pub mod auth;
pub mod authorize_as_contract;
pub mod burn_auth;
pub mod map_key_explosion;
pub mod mint_auth;
pub mod contracttype;
pub mod current_contract_unwrap;
pub mod float_arithmetic;
pub mod instance_ttl;
pub mod missing_ttl;
pub mod no_std;
pub mod env_in_struct;
pub mod sequence_as_key;
pub mod temp_get_no_has;
pub mod overflow;
pub mod panic_usage;
pub mod partial_write_on_error;
pub mod reentrancy;
pub mod self_transfer;
pub mod sequence_nonce;
pub mod storage;
pub mod instance_domain_mixing;
pub mod ttl_arg_order;
pub mod unbounded_storage;
pub mod unbounded_input_storage;
pub mod weak_randomness;
pub mod token_transfer_unchecked;
pub mod token_burn_auth;
pub mod zero_amount;
pub mod unvalidated_invoke_target;
mod util;

pub use admin::UnprotectedAdminCheck;
pub use admin_in_temp::AdminInTempCheck;
pub use dynamic_symbol_key::DynamicSymbolKeyCheck;
pub use instance_vec_growth::InstanceVecGrowthCheck;
pub use invoke_unchecked_cast::InvokeUncheckedCastCheck;
pub use migration_guard::MigrationGuardCheck;
pub use negative_deposit::NegativeDepositCheck;
pub use no_param_no_auth::NoParamNoAuthCheck;
pub use storage_type_confusion::StorageTypeConfusionCheck;
pub use withdraw_auth::WithdrawAuthCheck;
pub use admin_key_removal::AdminKeyRemovalCheck;
pub use admin_overwrite::AdminOverwriteCheck;
pub use assert_for_auth::AssertForAuthCheck;
pub use auth::MissingRequireAuthCheck;
pub use authorize_as_contract::AuthorizeAsContractCheck;
pub use burn_auth::BurnAuthCheck;
pub use map_key_explosion::MapKeyExplosionCheck;
pub use mint_auth::MintAuthCheck;
pub use current_contract_unwrap::CurrentContractUnwrapCheck;
pub use contracttype::MissingContracttypeCheck;
pub use float_arithmetic::FloatArithmeticCheck;
pub use instance_ttl::InstanceTtlCheck;
pub use missing_ttl::MissingTtlExtensionCheck;
pub use no_std::NoStdCheck;
pub use sequence_as_key::SequenceAsKeyCheck;
pub use env_in_struct::EnvInStructCheck;
pub use overflow::UncheckedArithmeticCheck;
pub use panic_usage::PanicUsageCheck;
pub use partial_write_on_error::PartialWriteOnErrorCheck;
pub use reentrancy::ReentrancyCheck;
pub use self_transfer::SelfTransferCheck;
pub use sequence_nonce::SequenceNonceCheck;
pub use storage::UnsafeStoragePatternsCheck;
pub use instance_domain_mixing::InstanceDomainMixingCheck;
pub use token_transfer_unchecked::TokenTransferUncheckedCheck;
pub use token_burn_auth::TokenBurnAuthCheck;
pub use ttl_arg_order::TtlArgOrderCheck;
pub use unbounded_storage::UnboundedStorageCheck;
pub use unbounded_input_storage::UnboundedInputStorageCheck;
pub use weak_randomness::WeakRandomnessCheck;
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
        Box::new(AdminOverwriteCheck),
        Box::new(AdminKeyRemovalCheck),
        Box::new(UnsafeStoragePatternsCheck),
        Box::new(InstanceDomainMixingCheck),
        Box::new(PanicUsageCheck),
        Box::new(PartialWriteOnErrorCheck),
        Box::new(MissingContracttypeCheck),
        Box::new(UnboundedStorageCheck),
        Box::new(UnboundedInputStorageCheck),
        Box::new(ZeroAmountCheck),
        Box::new(SelfTransferCheck),
        Box::new(SequenceAsKeyCheck),
        Box::new(NoStdCheck),
        Box::new(EnvInStructCheck),
        Box::new(TempGetNoHasCheck),
        Box::new(FloatArithmeticCheck),
        Box::new(WeakRandomnessCheck),
        Box::new(ReentrancyCheck),
        Box::new(TokenTransferUncheckedCheck),
        Box::new(ContracterrorAttrCheck),
        Box::new(TokenBurnAuthCheck),
        Box::new(MintAuthCheck),
        Box::new(SequenceNonceCheck),
        Box::new(AssertForAuthCheck),
        Box::new(AuthorizeAsContractCheck),
        Box::new(MapKeyExplosionCheck),
        Box::new(DynamicSymbolKeyCheck),
        Box::new(InstanceVecGrowthCheck),
        Box::new(MigrationGuardCheck),
        Box::new(WithdrawAuthCheck),
        Box::new(InvokeUncheckedCastCheck),
        Box::new(NegativeDepositCheck),
        Box::new(NoParamNoAuthCheck),
        Box::new(StorageTypeConfusionCheck),
    ]
}
