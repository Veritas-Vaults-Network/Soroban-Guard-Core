#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct WrappingBalanceOpVulnerable;

#[contractimpl]
impl WrappingBalanceOpVulnerable {
    /// Uses wrapping_add on balance — should trigger `wrapping-balance-op` (High).
    pub fn deposit(env: Env, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&Symbol::new(&env, "bal")).unwrap_or(0);
        let new_balance = balance.wrapping_add(amount);
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &new_balance);
    }
}
