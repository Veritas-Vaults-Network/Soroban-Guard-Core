#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct EventBeforeStorageVulnerable;

const BAL: Symbol = symbol_short!("bal");

#[contractimpl]
impl EventBeforeStorageVulnerable {
    /// ❌ Event is published before the storage write.
    /// If the set() call panics, the event is already observable on-chain.
    pub fn deposit(env: Env, amount: i128) {
        env.events().publish((symbol_short!("deposit"),), amount);
        env.storage().persistent().set(&BAL, &amount);
    }
}
