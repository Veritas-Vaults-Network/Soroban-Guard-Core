#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    /// Uses sha256 with nonce — should pass weak-commitment
    pub fn commit(env: Env, data: Bytes, nonce: Bytes) {
        let combined = (data, nonce);
        let hash = env.crypto().sha256(&combined);
        env.storage()
            .instance()
            .set(&soroban_sdk::Symbol::new(&env, "hash"), &hash);
    }
}
