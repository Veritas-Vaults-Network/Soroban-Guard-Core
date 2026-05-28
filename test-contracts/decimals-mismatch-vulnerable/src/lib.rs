#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct DecimalsMismatchVulnerable;

#[contractimpl]
impl DecimalsMismatchVulnerable {
    /// ❌ Declares 7 decimals but uses 1_000_000 (6 decimals) as the scaling
    /// constant — off-chain clients will display balances 10× too large.
    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn balance(env: Env, addr: Address) -> i128 {
        let raw: i128 = env.storage().instance().get(&addr).unwrap_or(0);
        raw / 1_000_000 // implies 6 decimals, but decimals() says 7
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let scaled = amount * 1_000_000; // 6-decimal scaling
        env.storage().instance().set(&to, &scaled);
        let _ = from;
    }
}
