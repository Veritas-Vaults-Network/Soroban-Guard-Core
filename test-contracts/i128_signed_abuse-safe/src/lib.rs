#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct I128SignedAbuseSafe;

#[contractimpl]
impl I128SignedAbuseSafe {
    // ✅ Non-negative values use u128 / u64, enforcing the invariant at the type level.
    pub fn mint(env: Env, balance: u128, supply: u128) {
        let _ = (env, balance, supply);
    }

    pub fn set_limit(env: Env, cap: u64, limit: u64) {
        let _ = (env, cap, limit);
    }

    pub fn record(env: Env, count: u64, total: u128, amount: u128) {
        let _ = (env, count, total, amount);
    }

    // ✅ i128 is fine for values that are legitimately signed (e.g. price delta).
    pub fn adjust_price(env: Env, delta: i128) {
        let _ = (env, delta);
    }
}
