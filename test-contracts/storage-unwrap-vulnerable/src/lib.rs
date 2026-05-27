#![no_std]

use soroban_sdk::{contractimpl, Env, Symbol};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn get_balance_unwrap(env: Env, key: Symbol) -> i128 {
        env.storage().persistent().get(&key).unwrap()
    }

    pub fn get_balance_expect(env: Env, key: Symbol) -> i128 {
        env.storage().persistent().get(&key).expect("Balance not found")
    }

    pub fn transfer_unwrap(env: Env, from: Symbol, to: Symbol, amount: i128) {
        let from_balance = env.storage().persistent().get(&from).unwrap();
        let to_balance = env.storage().persistent().get(&to).unwrap_or(0);

        env.storage().persistent().set(&from, &(from_balance - amount));
        env.storage().persistent().set(&to, &(to_balance + amount));
    }
}