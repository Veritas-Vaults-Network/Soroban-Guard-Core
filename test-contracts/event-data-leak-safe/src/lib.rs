#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn emit_event_safe(env: Env) {
        // ✅ Short plaintext string (within 32 char limit)
        env.events().publish(
            (symbol_short!("event"),),
            "safe"
        );
    }

    pub fn emit_event_at_limit(env: Env) {
        // ✅ Exactly 32 characters - at the limit
        env.events().publish(
            (symbol_short!("event"),),
            "12345678901234567890123456789012"
        );
    }
}
