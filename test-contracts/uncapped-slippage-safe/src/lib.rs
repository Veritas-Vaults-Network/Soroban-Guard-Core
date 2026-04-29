#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

const MAX_SLIPPAGE_BPS: u32 = 1000; // 10%

#[contract]
pub struct DexContract;

#[contractimpl]
impl DexContract {
    /// ✅ Slippage capped at 10% before use.
    pub fn swap(env: Env, amount: i128, slippage: u32) -> i128 {
        assert!(slippage <= MAX_SLIPPAGE_BPS, "slippage exceeds maximum");
        let out = amount - (amount * slippage as i128 / 10000);
        env.storage().instance().set(&symbol_short!("last"), &out);
        out
    }
}
