#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct UncappedFeeSafe;

const FEE_KEY: Symbol = symbol_short!("fee_bps");

#[contractimpl]
impl UncappedFeeSafe {
    /// ✅ fee_bps is capped at 10000 (100%) before being applied to amount.
    pub fn charge(env: Env, amount: i128) -> i128 {
        let fee_bps: i128 = env.storage().persistent().get(&FEE_KEY).unwrap_or(0);
        assert!(fee_bps <= 10000, "fee exceeds 100%");
        amount * fee_bps / 10000
    }
}
