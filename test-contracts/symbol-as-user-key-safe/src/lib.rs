#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct SymbolAsUserKeySafe;

#[contractimpl]
impl SymbolAsUserKeySafe {
    pub fn set_balance(env: Env, user: Address, amount: i128) {
        // ✅ Using (&user, symbol_short!("balance")) to create per-user keys
        env.storage().persistent().set((&user, symbol_short!("balance")), &amount);
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        // ✅ Same safe pattern
        env.storage().persistent().get((&user, symbol_short!("balance"))).unwrap_or(0)
    }

    pub fn set_global_total(env: Env, total: i128) {
        // ✅ Using symbol_short! as global key is fine when no Address parameter
        env.storage().persistent().set(symbol_short!("total"), &total);
    }
}