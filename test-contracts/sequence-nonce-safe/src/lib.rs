#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct SequenceNonceSafe;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    LastNonce,
}

#[contractimpl]
impl SequenceNonceSafe {
    /// Safely uses sequence as nonce and stores it in persistent storage.
    pub fn safe_nonce_check(env: Env, nonce: u32) {
        if env.ledger().sequence() == nonce {
            env.storage().persistent().set(&DataKey::LastNonce, &nonce);
            // Do something
        }
    }

    /// Another safe pattern with proper storage.
    pub fn safe_greater_check(env: Env, nonce: u32) {
        if env.ledger().sequence() > nonce {
            env.storage().persistent().set(&DataKey::LastNonce, &nonce);
            // Process safely
        }
    }

    /// Uses sequence without comparison — safe (no nonce usage).
    pub fn just_sequence_usage(env: Env) {
        let current_seq = env.ledger().sequence();
        // Just logging or other safe usage
    }
}