#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct BalanceOverflowVulnerable;

const BAL: Symbol = symbol_short!("bal");

#[contractimpl]
impl BalanceOverflowVulnerable {
    /// ❌ Reads balance from persistent storage and adds `amount` with plain `+`.
    /// A sufficiently large `amount` overflows `i128`, wrapping the balance to
    /// a negative or zero value — effectively stealing funds.
    pub fn deposit(env: Env, amount: i128) {
        let bal: i128 = env.storage().persistent().get(&BAL).unwrap_or(0);
        let new_bal = bal + amount;
        env.storage().persistent().set(&BAL, &new_bal);
    }
}
