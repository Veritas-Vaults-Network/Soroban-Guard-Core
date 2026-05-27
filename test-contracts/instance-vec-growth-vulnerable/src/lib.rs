#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, Vec};

#[contract]
pub struct InstanceVecGrowthVulnerable;

/// Vulnerable: appends to a Vec in instance storage without enforcing a max length.
/// Eventually the entry will exceed the maximum instance storage size, bricking the contract.
#[contractimpl]
impl InstanceVecGrowthVulnerable {
    pub fn register(env: Env, participant: Address) {
        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&symbol_short!("parts"))
            .unwrap_or(vec![&env]);
        // BUG: no length cap before push_back.
        list.push_back(participant);
        env.storage()
            .instance()
            .set(&symbol_short!("parts"), &list);
    }
}
