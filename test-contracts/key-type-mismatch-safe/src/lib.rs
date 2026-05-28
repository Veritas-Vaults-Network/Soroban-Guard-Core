#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct KeyTypeMismatchSafe;

const BALANCE: Symbol = symbol_short!("balance");
const CONFIG: Symbol = symbol_short!("config");

#[contractimpl]
impl KeyTypeMismatchSafe {
    pub fn set_balance(env: Env, amount: i128) {
        // ✅ Written with a Symbol key
        env.storage().persistent().set(&BALANCE, &amount);
    }

    pub fn get_balance(env: Env) -> i128 {
        // ✅ Read back with the same Symbol key — types match, lookup succeeds
        env.storage().persistent().get(&BALANCE).unwrap_or(0)
    }

    pub fn init_config(env: Env, val: u32) {
        // ✅ Written with a Symbol key
        env.storage().persistent().set(&CONFIG, &val);
    }

    pub fn get_config(env: Env) -> u32 {
        // ✅ Read back with the same Symbol key
        env.storage().persistent().get(&CONFIG).unwrap_or(0)
    }
}
