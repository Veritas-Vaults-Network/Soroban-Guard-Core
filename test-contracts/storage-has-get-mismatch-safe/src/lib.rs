#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageHasGetMismatchSafe;

const BALANCE: Symbol = symbol_short!("balance");
const LOCKED: Symbol = symbol_short!("locked");
const OWNER: Symbol = symbol_short!("owner");

#[contractimpl]
impl StorageHasGetMismatchSafe {
    pub fn transfer(env: Env, amount: u32) {
        env.require_auth();
        // ✅ Check and get use the same key
        if env.storage().persistent().has(&BALANCE) {
            let balance: u32 = env.storage().persistent().get(&BALANCE).unwrap_or(0);
            let _ = (amount, balance);
        }
    }

    pub fn check_locked(env: Env) {
        env.require_auth();
        // ✅ Separate checks for different keys
        if env.storage().persistent().has(&BALANCE) {
            let balance: u32 = env.storage().persistent().get(&BALANCE).unwrap_or(0);
            if env.storage().persistent().has(&LOCKED) {
                let locked: u32 = env.storage().persistent().get(&LOCKED).unwrap_or(0);
                let _ = (balance, locked);
            }
        }
    }

    pub fn update_owner(env: Env, new_owner: soroban_sdk::Address) {
        env.require_auth();
        // ✅ Check and get use the same key on instance storage
        if env.storage().instance().has(&OWNER) {
            let _owner: soroban_sdk::Address = env.storage().instance().get(&OWNER).unwrap();
            env.storage().instance().set(&OWNER, &new_owner);
        }
    }
}
