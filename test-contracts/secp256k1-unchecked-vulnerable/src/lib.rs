#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct Secp256k1UncheckedVulnerable;

/// Vulnerable: recovers a public key but never compares it against a trusted key.
/// The recovery result provides no authentication guarantee.
#[contractimpl]
impl Secp256k1UncheckedVulnerable {
    pub fn verify(env: Env, msg: Bytes, sig: Bytes) -> bool {
        // BUG: recovered key is never compared to a trusted key.
        let _recovered = env.crypto().secp256k1_recover(&msg, &sig, 0);
        true
    }
}
