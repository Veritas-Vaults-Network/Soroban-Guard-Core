#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct DecimalsMismatchSafe;

#[contractimpl]
impl DecimalsMismatchSafe {
    /// ✅ Declares 7 decimals and consistently uses 10_000_000 (10^7) as the
    /// scaling constant — off-chain clients will display balances correctly.
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn balance(env: Env, addr: Address) -> i128 {
        let raw: i128 = env.storage().instance().get(&addr).unwrap_or(0);
        raw / 10_000_000 // consistent with 7 decimals
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let scaled = amount * 10_000_000; // consistent 7-decimal scaling
        env.storage().instance().set(&to, &scaled);
        let _ = from;
    }
}
