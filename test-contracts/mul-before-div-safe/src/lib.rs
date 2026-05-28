#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MulBeforeDivSafe;

#[contractimpl]
impl MulBeforeDivSafe {
    /// ✅ Uses checked_mul to guard against overflow before dividing.
    pub fn calculate_reward(env: Env, principal: i128, rate: i128, divisor: i128) -> i128 {
        principal
            .checked_mul(rate)
            .expect("overflow")
            / divisor
    }

    /// ✅ Restructured to divide first when safe, avoiding large intermediate.
    pub fn pro_rata(env: Env, total: i128, share: i128, supply: i128) -> i128 {
        (total / supply) * share
    }
}
