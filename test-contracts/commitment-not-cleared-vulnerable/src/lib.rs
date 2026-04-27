#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct CommitmentNotClearedVulnerable;

#[contractimpl]
impl CommitmentNotClearedVulnerable {
    /// Store a commitment hash during the commit phase.
    pub fn commit(env: Env, hash: u64) {
        env.storage()
            .persistent()
            .set(&symbol_short!("commit"), &hash);
    }

    /// BUG: reads the commitment but never removes it — replay attack possible.
    pub fn reveal(env: Env, secret: u64) {
        let stored: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("commit"))
            .unwrap();
        assert_eq!(stored, secret);
        // Missing: env.storage().persistent().remove(&symbol_short!("commit"));
    }
}
