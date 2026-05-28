#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageHasGetMismatchVulnerable;

const BALANCE: Symbol = symbol_short!("balance");
const LOCKED: Symbol = symbol_short!("locked");
const OWNER: Symbol = symbol_short!("owner");

#[contractimpl]
impl StorageHasGetMismatchVulnerable {
    pub fn transfer(env: Env, amount: u32) {
        env.require_auth();
        // ❌ Check has(BALANCE) but get(LOCKED) - TOCTOU vulnerability
        if env.storage().persistent().has(&BALANCE) {
            let locked: u32 = env.storage().persistent().get(&LOCKED).unwrap_or(0);
            let _ = (amount, locked);
        }
    }

    pub fn update_owner(env: Env, new_owner: soroban_sdk::Address) {
        env.require_auth();
        // ❌ Check has(OWNER) but get(BALANCE) on instance storage
        if env.storage().instance().has(&OWNER) {
            let _balance: u32 = env.storage().instance().get(&BALANCE).unwrap_or(0);
            env.storage().instance().set(&OWNER, &new_owner);
        }
    }
}
