#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct CommitmentNotClearedSafe;

#[contractimpl]
impl CommitmentNotClearedSafe {
    /// Store a commitment hash during the commit phase.
    pub fn commit(env: Env, hash: u64) {
        env.storage()
            .persistent()
            .set(&symbol_short!("commit"), &hash);
    }

    /// Safe: removes the commitment after a successful reveal to prevent replay.
    pub fn reveal(env: Env, secret: u64) {
        let stored: u64 = env
            .storage()
            .persistent()
            .get(&symbol_short!("commit"))
            .unwrap();
        assert_eq!(stored, secret);
        env.storage().persistent().remove(&symbol_short!("commit"));
    }
}
