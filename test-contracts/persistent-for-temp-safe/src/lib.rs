#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct PersistentForTempSafe;

const NONCE: Symbol = symbol_short!("nonce");
const LOCK: Symbol = symbol_short!("lock");
// Long-lived data correctly uses persistent storage with a non-transient key name.
const BALANCE: Symbol = symbol_short!("balance");

#[contractimpl]
impl PersistentForTempSafe {
    /// ✅ One-time nonce stored in temporary storage — expires naturally,
    ///    no wasted ledger space, no manual TTL cleanup needed.
    pub fn consume_nonce(env: Env, expected: u64) {
        let current: u64 = env.storage().temporary().get(&NONCE).unwrap_or(0);
        assert_eq!(current, expected, "invalid nonce");
        env.storage().temporary().set(&NONCE, &(current + 1));
        env.storage()
            .temporary()
            .extend_ttl(&NONCE, 100, 200);
    }

    /// ✅ Short-lived lock stored in temporary storage — auto-expires,
    ///    no persistent ledger bloat.
    pub fn acquire_lock(env: Env) {
        let locked: bool = env.storage().temporary().get(&LOCK).unwrap_or(false);
        assert!(!locked, "already locked");
        env.storage().temporary().set(&LOCK, &true);
        env.storage().temporary().extend_ttl(&LOCK, 50, 100);
    }

    pub fn release_lock(env: Env) {
        env.storage().temporary().set(&LOCK, &false);
    }

    /// ✅ Long-lived balance correctly uses persistent storage.
    pub fn set_balance(env: Env, amount: i128) {
        env.storage().persistent().set(&BALANCE, &amount);
    }
}
