#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, contracterror, Env};

#[contract]
pub struct ContracterrorAttrSafe;

// ✅ Has #[contracterror] attribute
#[contracterror]
#[derive(Debug)]
pub enum MyError {
    InsufficientBalance,
    InvalidAmount,
}

#[contractimpl]
impl ContracterrorAttrSafe {
    pub fn withdraw(env: Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(&env, MyError::InvalidAmount);
        }
        // Simulate insufficient balance
        panic_with_error!(&env, MyError::InsufficientBalance);
    }
}