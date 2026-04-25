#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct HardcodedBytesSafe;

#[contractimpl]
impl HardcodedBytesSafe {
    pub fn process(env: Env) {
        let constant = [0u8; 32];
        let _ = Bytes::from_slice(&env, &constant);
    }
}
