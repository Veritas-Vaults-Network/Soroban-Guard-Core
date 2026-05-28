#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct KeyTypeMismatchVulnerable;

// Written as a Symbol via symbol_short!
const BALANCE: Symbol = symbol_short!("balance");
const CONFIG: Symbol = symbol_short!("config");

#[contractimpl]
impl KeyTypeMismatchVulnerable {
    pub fn set_balance(env: Env, amount: i128) {
        // Written with a Symbol key
        env.storage().persistent().set(&BALANCE, &amount);
    }

    pub fn get_balance(env: Env) -> i128 {
        // ❌ Looked up with a string literal — different runtime type.
        //    This will never find the entry written by set_balance.
        env.storage().persistent().get("balance").unwrap_or(0)
    }

    pub fn init_config(env: Env, val: u32) {
        // ❌ Written with a string literal this time...
        env.storage().persistent().set("config", &val);
    }

    pub fn get_config(env: Env) -> u32 {
        // ❌ ...but read back with the Symbol const — also a mismatch.
        env.storage().persistent().get(&CONFIG).unwrap_or(0)
    }
}
