#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct BalanceOverflowSafe;

const BAL: Symbol = symbol_short!("bal");

#[contractimpl]
impl BalanceOverflowSafe {
    /// ✅ Uses `checked_add` — panics on overflow instead of wrapping silently.
    pub fn deposit(env: Env, amount: i128) {
        let bal: i128 = env.storage().persistent().get(&BAL).unwrap_or(0);
        let new_bal = bal.checked_add(amount).expect("balance overflow");
        env.storage().persistent().set(&BAL, &new_bal);
    }
}
