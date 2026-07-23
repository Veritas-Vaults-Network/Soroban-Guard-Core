#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn withdraw_daily(env: Env, amount: i128) {
        let daily_limit = 1000;
        assert!(amount <= daily_limit);
        Self::update_accumulator(&env, amount);
    }
}

impl Contract {
    fn update_accumulator(env: &Env, amount: i128) {
        let mut total: i128 = env.storage().instance().get(&Symbol::new(&env, "total_withdrawn")).unwrap_or(0);
        total += amount;
        env.storage().instance().set(&Symbol::new(&env, "total_withdrawn"), &total);
    }
}
