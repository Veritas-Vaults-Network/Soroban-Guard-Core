#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FeeContract;

#[contractimpl]
impl FeeContract {
    // ❌ Uses f64 arithmetic — non-deterministic across WASM host environments.
    pub fn calculate_fee(_env: Env, amount: i128) -> i128 {
        let fee = amount as f64 * 0.025_f64;
        fee as i128
    }
}
