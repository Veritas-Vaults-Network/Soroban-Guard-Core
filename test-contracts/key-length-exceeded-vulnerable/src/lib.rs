#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct KeyLengthExceededVulnerable;

#[contractimpl]
impl KeyLengthExceededVulnerable {
    /// ❌ Key exceeds 32 bytes — will cause runtime failure.
    pub fn set_value(env: Env, value: u32) {
        env.storage()
            .persistent()
            .set(&"this_is_a_very_long_key_that_exceeds_the_limit", &value);
    }

    /// ❌ Another oversized key in get operation.
    pub fn get_value(env: Env) -> Option<u32> {
        env.storage()
            .persistent()
            .get(&"another_extremely_long_key_name_that_is_too_big")
    }
}
