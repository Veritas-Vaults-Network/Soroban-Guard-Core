#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FeeContract;

const FEE_BPS: i128 = 250; // 2.5% in basis points

#[contractimpl]
impl FeeContract {
    // ✅ Integer-only arithmetic — deterministic on all WASM hosts.
    pub fn calculate_fee(_env: Env, amount: i128) -> i128 {
        amount * FEE_BPS / 10_000
    }
}
