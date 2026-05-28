#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct ExtendTtlInLoopSafe;

/// Safe: extend_ttl called once outside any loop.
#[contractimpl]
impl ExtendTtlInLoopSafe {
    pub fn refresh(env: Env) {
        env.storage()
            .persistent()
            .extend_ttl(&symbol_short!("data"), 100, 200);
    }
}
