#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct HashAsStorageKeyVulnerable;

/// Vulnerable: uses sha256 of user input directly as a storage key.
/// An attacker can pre-compute keys to overwrite arbitrary storage slots.
#[contractimpl]
impl HashAsStorageKeyVulnerable {
    pub fn store(env: Env, input: Bytes, value: u32) {
        // BUG: hash of user input as key — arbitrary slot write.
        let key = env.crypto().sha256(&input);
        env.storage().persistent().set(&key, &value);
    }
}
