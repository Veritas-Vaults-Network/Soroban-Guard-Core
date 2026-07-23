#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct SafeContract;

const KEY: Symbol = symbol_short!("data");
const MAX_TTL: u32 = 10_000;

#[contractimpl]
impl SafeContract {
    /// Duration is a literal constant — fully safe.
    pub fn refresh_literal(env: Env) {
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, 5000);
    }

    /// Parameter is capped with `.min()` before being passed to extend_ttl.
    pub fn refresh_capped(env: Env, requested_ttl: u32) {
        let capped = requested_ttl.min(MAX_TTL);
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, capped);
    }

    /// Parameter is capped with `.clamp()` inline.
    pub fn refresh_clamped(env: Env, requested_ttl: u32) {
        let capped = requested_ttl.clamp(0u32, MAX_TTL);
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, capped);
    }

    /// `.min()` applied inline on the method call argument.
    pub fn refresh_inline_min(env: Env, requested_ttl: u32) {
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, requested_ttl.min(MAX_TTL));
    }

    /// Helper returns a literal — safe.
    pub fn refresh_via_helper(env: Env) {
        let dur = safe_ttl();
        env.storage()
            .persistent()
            .extend_ttl(&KEY, 100, dur);
    }

    /// Instance storage with a literal — safe.
    pub fn bump_instance(env: Env) {
        env.storage().instance().extend_ttl(100, 200);
    }
}

fn safe_ttl() -> u32 {
    500
}
