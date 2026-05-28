#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct KeyPrefixCollisionVulnerable;

#[contractimpl]
impl KeyPrefixCollisionVulnerable {
    pub fn process(env: Env) {
        env.require_auth();
        // ❌ "balance" and "balance_locked" have prefix collision
        env.storage().persistent().set(&"balance", &100u32);
        env.storage().persistent().set(&"balance_locked", &50u32);
    }

    pub fn user_data(env: Env) {
        env.require_auth();
        // ❌ "user_data" is prefix of "user_data_extra"
        env.storage().persistent().set(&"user_data", &1u32);
        env.storage().persistent().set(&"user_data_extra", &2u32);
    }

    pub fn config(env: Env) {
        env.require_auth();
        // ❌ "admin" is prefix of "admin_key"
        env.storage().instance().set(&"admin", &1u32);
        env.storage().instance().set(&"admin_key", &2u32);
    }
}
