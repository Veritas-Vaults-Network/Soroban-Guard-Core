#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct NoParamNoAuthVulnerable;

/// Vulnerable: public zero-parameter function writes to storage without auth.
/// Any caller can invoke this and reset the counter.
#[contractimpl]
impl NoParamNoAuthVulnerable {
    pub fn reset(env: Env) {
        // BUG: no require_auth, no parameters — anyone can call this.
        env.storage()
            .instance()
            .set(&symbol_short!("count"), &0u32);
    }
}
