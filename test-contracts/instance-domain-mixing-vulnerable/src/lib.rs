#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct InstanceDomainMixingVulnerable;

#[contractimpl]
impl InstanceDomainMixingVulnerable {
    pub fn store(env: Env) {
        env.storage().instance().set(&"balance_key", &42u32);
        env.storage().instance().set(&"proposal_id", &1u32);
        env.storage().instance().set(&"config_admin", &true);
    }
}
