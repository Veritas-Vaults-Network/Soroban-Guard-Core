#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Ed25519UncheckedSafe;

#[contractimpl]
impl Ed25519UncheckedSafe {
    pub fn verify(env: Env) {
        let valid = env.crypto().ed25519_verify(&env, &(), &());
        if valid {
            return;
        }
    }
}
