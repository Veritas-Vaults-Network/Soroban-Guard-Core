#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    /// Uses sha256 without nonce — should trigger weak-commitment
    pub fn commit(env: Env, data: Bytes) {
        let hash = env.crypto().sha256(&data);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "hash"), &hash);
    }
}
