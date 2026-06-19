#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct RenounceNoBackupVulnerable;

#[contractimpl]
impl RenounceNoBackupVulnerable {
    /// ❌ Removes the admin key with no backup — the contract is permanently
    /// locked after this call with no recovery path.
    pub fn renounce_ownership(env: Env) {
        env.storage()
            .instance()
            .remove(&symbol_short!("admin"));
    }
}
