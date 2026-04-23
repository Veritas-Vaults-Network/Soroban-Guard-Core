#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AmountTypeSafe;

#[contractimpl]
impl AmountTypeSafe {
    pub fn transfer(env: Env, to: Address, amount: i128) {
        // ✅ Using i128 for amount as per Soroban token interface
        let _ = (env, to, amount);
    }

    pub fn set_balance(env: Env, balance: i128) {
        // ✅ Using i128 for balance
        let _ = (env, balance);
    }

    pub fn deposit(env: Env, value: i128) {
        // ✅ Using i128 for value
        let _ = (env, value);
    }

    pub fn process(env: Env, count: u64) {
        // ✅ Using u64 for non-amount parameter is fine
        let _ = (env, count);
    }
}
