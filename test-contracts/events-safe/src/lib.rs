#![no_std]

use soroban_sdk::{contractimpl, Env, Symbol, symbol_short};

pub struct Contract;

const BALANCE: Symbol = symbol_short!("balance");

#[contractimpl]
impl Contract {
    pub fn set_balance(env: Env, amount: i128) {
        env.storage().persistent().set(&BALANCE, &amount);
        env.events().publish(("balance_updated",), (amount,));
    }

    pub fn transfer(env: Env, from: Symbol, to: Symbol, amount: i128) {
        // Read current balances
        let from_balance = env.storage().persistent().get(&from).unwrap_or(0);
        let to_balance = env.storage().persistent().get(&to).unwrap_or(0);

        // Update balances
        env.storage().persistent().set(&from, &(from_balance - amount));
        env.storage().persistent().set(&to, &(to_balance + amount));

        // Emit transfer event
        env.events().publish(("transfer",), (from, to, amount));
    }
}