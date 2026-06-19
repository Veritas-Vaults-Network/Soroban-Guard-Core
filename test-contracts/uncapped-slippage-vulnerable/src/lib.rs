#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct DexContract;

#[contractimpl]
impl DexContract {
    /// ❌ No cap on slippage — caller can pass 10000 (100%), disabling price protection.
    pub fn swap(env: Env, amount: i128, slippage: u32) -> i128 {
        let out = amount - (amount * slippage as i128 / 10000);
        env.storage().instance().set(&symbol_short!("last"), &out);
        out
    }
}
