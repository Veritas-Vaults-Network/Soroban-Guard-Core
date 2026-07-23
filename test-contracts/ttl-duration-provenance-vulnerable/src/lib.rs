#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct VulnerableContract;

const KEY: Symbol = symbol_short!("data");

#[contractimpl]
impl VulnerableContract {
    /// Duration traced directly back to a function parameter — no cap applied.
    /// A caller can request an arbitrarily large TTL.
    pub fn refresh_tier(env: Env, requested_ttl: u32) {
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, requested_ttl);
    }

    /// Duration traced through an intermediate binding back to a parameter.
    pub fn refresh_user(env: Env, ttl: u32) {
        let duration = ttl;
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, duration);
    }

    /// Duration traced through a helper that returns its parameter directly.
    pub fn refresh_via_helper(env: Env, requested_ttl: u32) {
        let dur = pass_through(requested_ttl);
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, dur);
    }

    /// Instance storage: same pattern, duration from parameter.
    pub fn bump_instance(env: Env, ttl: u32) {
        env.storage().instance().extend_ttl(100, ttl);
    }
}

fn pass_through(val: u32) -> u32 {
    val
}
