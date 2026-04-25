#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct MigrationVulnerable;

/// Vulnerable: rewrites storage keys without checking the current schema version.
/// Calling `migrate` twice will corrupt data because there is no version sentinel guard.
#[contractimpl]
impl MigrationVulnerable {
    pub fn migrate(env: Env) {
        // BUG: no version/schema sentinel read before writing.
        let old_val: u32 = env
            .storage()
            .persistent()
            .get(&symbol_short!("old"))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&symbol_short!("new"), &old_val);
        env.storage()
            .persistent()
            .remove(&symbol_short!("old"));
    }
}
