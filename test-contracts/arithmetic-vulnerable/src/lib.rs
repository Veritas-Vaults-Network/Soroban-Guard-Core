#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct ArithmeticVulnerable;

#[contractimpl]
impl ArithmeticVulnerable {
    /// Wrapping `+` on `i128` — should trigger `unchecked-arithmetic` (Medium).
    pub fn total(_env: Env, a: i128, b: i128) -> i128 {
        a + b
    }
}
