#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct Ed25519UncheckedVulnerable;

#[contractimpl]
impl Ed25519UncheckedVulnerable {
    pub fn verify(env: Env) {
        env.crypto().ed25519_verify(&env, &(), &());
    }
}
