#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct NegativeBalanceSafe;

const BALANCE_KEY: Symbol = symbol_short!("balance");

#[contractimpl]
impl NegativeBalanceSafe {
    /// Checks for underflow before subtracting — should pass.
    pub fn withdraw(env: Env, amount: i128) {
        let balance: i128 = 100;
        if balance >= amount {
            env.storage().instance().set(&BALANCE_KEY, &(balance - amount));
        }
    }
}
