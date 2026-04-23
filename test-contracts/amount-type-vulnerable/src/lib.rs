#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AmountTypeVulnerable;

#[contractimpl]
impl AmountTypeVulnerable {
    pub fn transfer(env: Env, to: Address, amount: u64) {
        // ❌ Using u64 for amount instead of i128
        // This silently truncates values and is incompatible with Soroban token interface
        let _ = (env, to, amount);
    }

    pub fn set_balance(env: Env, balance: u32) {
        // ❌ Using u32 for balance instead of i128
        let _ = (env, balance);
    }

    pub fn deposit(env: Env, value: u64) {
        // ❌ Using u64 for value instead of i128
        let _ = (env, value);
    }
}
