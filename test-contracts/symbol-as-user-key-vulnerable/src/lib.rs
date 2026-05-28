#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct SymbolAsUserKeyVulnerable;

#[contractimpl]
impl SymbolAsUserKeyVulnerable {
    pub fn set_balance(env: Env, user: Address, amount: i128) {
        // ❌ Using symbol_short!("balance") as bare key with Address parameter
        // This causes all users to share the same storage slot
        env.storage().persistent().set(symbol_short!("balance"), &amount);
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        // ❌ Same issue here
        env.storage().persistent().get(symbol_short!("balance")).unwrap_or(0)
    }
}