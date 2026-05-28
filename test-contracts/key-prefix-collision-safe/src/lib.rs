#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct KeyPrefixCollisionSafe;

#[contractimpl]
impl KeyPrefixCollisionSafe {
    pub fn process(env: Env) {
        env.require_auth();
        // ✅ Distinct, non-overlapping keys
        env.storage().persistent().set(&"balance", &100u32);
        env.storage().persistent().set(&"locked_amount", &50u32);
    }

    pub fn user_data(env: Env) {
        env.require_auth();
        // ✅ Clearly separated namespaces
        env.storage().persistent().set(&"user:data", &1u32);
        env.storage().persistent().set(&"user:extra", &2u32);
    }

    pub fn config(env: Env) {
        env.require_auth();
        // ✅ Distinct prefixes
        env.storage().instance().set(&"admin_addr", &1u32);
        env.storage().instance().set(&"owner_addr", &2u32);
    }

    pub fn namespaced(env: Env) {
        env.require_auth();
        // ✅ Using namespace separators
        env.storage().persistent().set(&"state:balance", &100u32);
        env.storage().persistent().set(&"state:owner", &1u32);
        env.storage().persistent().set(&"config:fee", &2u32);
    }
}
