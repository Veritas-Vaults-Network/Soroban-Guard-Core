#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct NegativeBalanceVulnerable;

const BALANCE_KEY: Symbol = symbol_short!("balance");

#[contractimpl]
impl NegativeBalanceVulnerable {
    /// Subtracts without checking for underflow — should trigger check.
    pub fn withdraw(env: Env, amount: i128) {
        let balance: i128 = 100;
        env.storage().instance().set(&BALANCE_KEY, &(balance - amount));
    }
}
