#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct MigrationSafe;

/// Safe: reads the version sentinel before performing any storage writes.
/// Asserts the pre-migration version and updates it after migration completes.
#[contractimpl]
impl MigrationSafe {
    pub fn migrate(env: Env) {
        // Read and assert the current schema version before any writes.
        let version: u32 = env
            .storage()
            .instance()
            .get(&symbol_short!("version"))
            .unwrap_or(0);
        assert_eq!(version, 1u32, "unexpected schema version");

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

        // Update the version sentinel after migration.
        env.storage()
            .instance()
            .set(&symbol_short!("version"), &2u32);
    }
}
