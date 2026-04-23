#![no_std]

use soroban_sdk::{contractimpl, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn get_balance_safe(env: Env, key: Symbol) -> i128 {
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn get_balance_safe_default(env: Env, key: Symbol) -> i128 {
        env.storage().persistent().get(&key).unwrap_or_default()
    }

    pub fn transfer_safe(env: Env, from: Symbol, to: Symbol, amount: i128) {
        let from_balance = env.storage().persistent().get(&from).unwrap_or(0);
        let to_balance = env.storage().persistent().get(&to).unwrap_or(0);

        env.storage().persistent().set(&from, &(from_balance - amount));
        env.storage().persistent().set(&to, &(to_balance + amount));
    }
}