#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct BytesNotBytesNVulnerable;

/// Vulnerable: uses Bytes (variable-length) for fixed-size crypto parameters.
#[contractimpl]
impl BytesNotBytesNVulnerable {
    pub fn verify(env: Env, hash: Bytes, signature: Bytes) -> bool {
        // BUG: Bytes allows wrong-sized inputs — should be BytesN<32> / BytesN<64>.
        let _ = (env, hash, signature);
        true
    }
}
