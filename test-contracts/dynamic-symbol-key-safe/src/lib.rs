#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct DynamicSymbolKeySafe;

/// Safe: uses a compile-time constant symbol as the storage key.
#[contractimpl]
impl DynamicSymbolKeySafe {
    pub fn store(env: Env, value: u32) {
        // Safe: key is a compile-time constant.
        env.storage()
            .persistent()
            .set(&symbol_short!("data"), &value);
    }
}
