#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct BytesNotBytesNSafe;

/// Safe: uses BytesN<N> to enforce exact sizes for cryptographic parameters.
#[contractimpl]
impl BytesNotBytesNSafe {
    pub fn verify(env: Env, hash: BytesN<32>, signature: BytesN<64>) -> bool {
        let _ = (env, hash, signature);
        true
    }
}
