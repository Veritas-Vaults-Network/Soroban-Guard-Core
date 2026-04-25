#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Address, Env, Vec};

#[contract]
pub struct InstanceVecGrowthSafe;

const MAX_PARTICIPANTS: u32 = 100;

/// Safe: enforces a maximum length before appending to the Vec.
#[contractimpl]
impl InstanceVecGrowthSafe {
    pub fn register(env: Env, participant: Address) {
        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&symbol_short!("parts"))
            .unwrap_or(vec![&env]);
        // Safe: length is checked before appending.
        assert!(list.len() < MAX_PARTICIPANTS, "participant list is full");
        list.push_back(participant);
        env.storage()
            .instance()
            .set(&symbol_short!("parts"), &list);
    }
}
