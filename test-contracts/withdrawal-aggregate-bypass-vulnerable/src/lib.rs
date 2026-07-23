#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn withdraw_daily(env: Env, amount: i128) {
        let daily_limit = 1000;
        assert!(amount <= daily_limit);
        // Vulnerable: no accumulator touches anywhere
    }
}
