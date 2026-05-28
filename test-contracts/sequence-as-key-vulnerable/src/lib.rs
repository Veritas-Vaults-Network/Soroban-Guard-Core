#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct SequenceAsKeyVulnerable;

#[contractimpl]
impl SequenceAsKeyVulnerable {
    pub fn store_with_sequence(env: Env, value: u32) {
        env.storage().persistent().set(&env.ledger().sequence(), &value);
    }

    pub fn store_with_timestamp(env: Env, value: u32) {
        env.storage().instance().set(&env.ledger().timestamp(), &value);
    }
}
