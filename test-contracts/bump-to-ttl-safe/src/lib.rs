#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct BumpToTtlSafe;

const BALANCE: Symbol = symbol_short!("balance");
const OWNER: Symbol = symbol_short!("owner");

#[contractimpl]
impl BumpToTtlSafe {
    pub fn update_balance(env: Env, amount: u32) {
        env.require_auth();
        // ✅ extend_ttl guarantees extension
        env.storage().persistent().extend_ttl(&BALANCE, 100, 1000);
        env.storage().persistent().set(&BALANCE, &amount);
    }

    pub fn update_owner(env: Env, new_owner: soroban_sdk::Address) {
        env.require_auth();
        // ✅ extend_ttl on instance storage
        env.storage().instance().extend_ttl(&OWNER, 100, 1000);
        env.storage().instance().set(&OWNER, &new_owner);
    }

    pub fn temp_cache(env: Env, value: u32) {
        env.require_auth();
        // ✅ bump_to_ttl is acceptable for temporary storage
        let temp_key = symbol_short!("temp");
        env.storage().temporary().bump_to_ttl(&temp_key, 100, 1000);
        env.storage().temporary().set(&temp_key, &value);
    }
}
