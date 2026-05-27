#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn emit_event_leak(env: Env) {
        // ❌ Emitting oversized plaintext string (> 32 chars) - privacy risk
        env.events().publish(
            (symbol_short!("leak"),),
            "this is a very long string that exceeds the limit"
        );
    }

    pub fn emit_private_key(env: Env) {
        // ❌ Emitting sensitive data in plaintext
        env.events().publish(
            (symbol_short!("key"),),
            "SAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
    }
}
