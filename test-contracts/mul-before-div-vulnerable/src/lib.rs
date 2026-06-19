#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MulBeforeDivVulnerable;

#[contractimpl]
impl MulBeforeDivVulnerable {
    /// ❌ `x * y / z` — intermediate `x * y` can overflow i128::MAX.
    pub fn calculate_reward(env: Env, principal: i128, rate: i128, divisor: i128) -> i128 {
        principal * rate / divisor
    }

    /// ❌ Parenthesized form — same overflow risk.
    pub fn pro_rata(env: Env, total: i128, share: i128, supply: i128) -> i128 {
        (total * share) / supply
    }
}
