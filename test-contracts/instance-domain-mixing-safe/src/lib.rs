#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct InstanceDomainMixingSafe;

#[contractimpl]
impl InstanceDomainMixingSafe {
    pub fn store(env: Env) {
        env.storage().instance().set(&"balance_key", &42u32);
        env.storage().instance().set(&"allowance_key", &7u32);
    }
}
