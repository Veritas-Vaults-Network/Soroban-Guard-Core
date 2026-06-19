#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, Env};

#[contract]
pub struct ContracterrorAttrVulnerable;

// ❌ Missing #[contracterror] and #[repr(u32)] attributes
#[derive(Debug)]
pub enum MyError {
    InsufficientBalance,
    InvalidAmount,
}

#[contractimpl]
impl ContracterrorAttrVulnerable {
    pub fn withdraw(env: Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(&env, MyError::InvalidAmount);
        }
        // Simulate insufficient balance
        panic_with_error!(&env, MyError::InsufficientBalance);
    }
}