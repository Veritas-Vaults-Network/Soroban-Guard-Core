#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct HardcodedBytesVulnerable;

#[contractimpl]
impl HardcodedBytesVulnerable {
    pub fn process(env: Env) {
        let _ = Bytes::from_slice(&env, &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]);
    }
}
