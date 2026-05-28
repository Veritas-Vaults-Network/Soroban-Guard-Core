#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct KeyLengthExceededSafe;

#[contractimpl]
impl KeyLengthExceededSafe {
    /// ✅ Key is within 32-byte limit.
    pub fn set_value(env: Env, value: u32) {
        env.storage().persistent().set(&"value", &value);
    }

    /// ✅ Key is within 32-byte limit.
    pub fn get_value(env: Env) -> Option<u32> {
        env.storage().persistent().get(&"value")
    }
}
