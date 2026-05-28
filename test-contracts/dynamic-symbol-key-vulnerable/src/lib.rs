#![no_std]
use soroban_sdk::{contract, contractimpl, Env, String, Symbol};

#[contract]
pub struct DynamicSymbolKeyVulnerable;

/// Vulnerable: uses a caller-supplied string as a storage key via Symbol::new.
/// Any caller can write to an arbitrary storage slot by choosing the key.
#[contractimpl]
impl DynamicSymbolKeyVulnerable {
    pub fn store(env: Env, key: String, value: u32) {
        // BUG: Symbol::new with a runtime parameter — arbitrary slot write.
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, key), &value);
    }
}
