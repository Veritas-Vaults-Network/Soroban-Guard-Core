#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct BalanceNegativeCheckVulnerable;

const BALANCE_KEY: Symbol = symbol_short!("balance");

#[contractimpl]
impl BalanceNegativeCheckVulnerable {
    /// ❌ Uses `balance == 0` instead of `balance <= 0`.
    /// A negative balance (from a bug) would bypass this guard.
    pub fn burn(env: Env, amount: i128) {
        let balance: i128 = env.storage().instance().get(&BALANCE_KEY).unwrap_or(0);
        if balance == 0 {
            panic!("insufficient balance");
        }
        env.storage().instance().set(&BALANCE_KEY, &(balance - amount));
    }
}
