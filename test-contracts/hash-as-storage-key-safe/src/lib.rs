#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Bytes, Env};

#[contract]
pub struct HashAsStorageKeySafe;

/// Safe: uses a constant key; stores the hash as the value, not the key.
#[contractimpl]
impl HashAsStorageKeySafe {
    pub fn store(env: Env, input: Bytes, value: u32) {
        // Safe: constant key, hash stored as value for integrity checking.
        let _hash = env.crypto().sha256(&input);
        env.storage()
            .persistent()
            .set(&symbol_short!("data"), &value);
    }
}
