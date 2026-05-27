#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct NoParamNoAuthSafe;

/// Safe: requires auth before writing to storage.
#[contractimpl]
impl NoParamNoAuthSafe {
    pub fn reset(env: Env) {
        // Safe: only the contract itself (or an authorized caller) can reset.
        env.require_auth();
        env.storage()
            .instance()
            .set(&symbol_short!("count"), &0u32);
    }
}
