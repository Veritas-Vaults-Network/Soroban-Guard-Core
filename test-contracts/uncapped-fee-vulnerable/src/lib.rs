#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct UncappedFeeVulnerable;

const FEE_KEY: Symbol = symbol_short!("fee_bps");

#[contractimpl]
impl UncappedFeeVulnerable {
    /// ❌ fee_bps is read from storage and multiplied against amount with no
    /// `<= 10000` guard. An admin who sets fee_bps > 10000 can drain the
    /// caller's entire balance or more.
    pub fn charge(env: Env, amount: i128) -> i128 {
        let fee_bps: i128 = env.storage().persistent().get(&FEE_KEY).unwrap_or(0);
        amount * fee_bps / 10000
    }
}
