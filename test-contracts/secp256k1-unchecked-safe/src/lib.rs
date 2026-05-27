#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Bytes, BytesN, Env};

#[contract]
pub struct Secp256k1UncheckedSafe;

/// Safe: compares the recovered public key against a stored trusted key.
#[contractimpl]
impl Secp256k1UncheckedSafe {
    pub fn verify(env: Env, msg: Bytes, sig: Bytes) -> bool {
        let recovered: BytesN<65> = env.crypto().secp256k1_recover(&msg, &sig, 0);
        // Safe: compare against the stored trusted public key.
        let trusted: BytesN<65> = env
            .storage()
            .persistent()
            .get(&symbol_short!("pubkey"))
            .unwrap();
        recovered == trusted
    }
}
