#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct WrappingBalanceOpSafe;

#[contractimpl]
impl WrappingBalanceOpSafe {
    /// Uses checked_add on balance — should pass `wrapping-balance-op`.
    pub fn deposit(env: Env, amount: i128) {
        let balance: i128 = env.storage().persistent().get(&Symbol::new(&env, "bal")).unwrap_or(0);
        let new_balance = balance.checked_add(amount).expect("overflow");
        env.storage().persistent().set(&Symbol::new(&env, "bal"), &new_balance);
    }
}
