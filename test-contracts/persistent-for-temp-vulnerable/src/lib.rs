#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct PersistentForTempVulnerable;

const NONCE: Symbol = symbol_short!("nonce");
const LOCK: Symbol = symbol_short!("lock");

#[contractimpl]
impl PersistentForTempVulnerable {
    /// ❌ One-time nonce stored in persistent storage — wastes ledger space
    ///    and requires manual TTL management for data that is inherently transient.
    pub fn consume_nonce(env: Env, expected: u64) {
        let current: u64 = env.storage().persistent().get(&NONCE).unwrap_or(0);
        assert_eq!(current, expected, "invalid nonce");
        env.storage().persistent().set(&NONCE, &(current + 1));
    }

    /// ❌ Mutex-style lock stored in persistent storage — should be temporary.
    pub fn acquire_lock(env: Env) {
        let locked: bool = env.storage().persistent().get(&LOCK).unwrap_or(false);
        assert!(!locked, "already locked");
        env.storage().persistent().set(&LOCK, &true);
    }

    pub fn release_lock(env: Env) {
        env.storage().persistent().set(&LOCK, &false);
    }
}
