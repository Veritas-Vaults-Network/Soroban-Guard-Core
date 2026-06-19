#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct ExtendTtlInLoopVulnerable;

/// Vulnerable: extend_ttl called inside a loop — unbounded compute cost.
#[contractimpl]
impl ExtendTtlInLoopVulnerable {
    pub fn refresh_all(env: Env, keys: Vec<Address>) {
        for key in keys {
            // BUG: one host call per iteration — scales with list size.
            env.storage().persistent().extend_ttl(&key, 100, 200);
        }
    }
}
