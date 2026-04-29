#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    /// ❌ topic comes directly from a caller-supplied parameter — any string
    /// can be injected, leaking data and enabling event-log spam.
    pub fn emit_with_param_topic(env: Env, topic: Symbol) {
        env.events().publish((topic,), true);
    }

    /// ❌ second element of the tuple is user-supplied.
    pub fn emit_mixed_topics(env: Env, user_topic: Symbol) {
        env.events().publish((symbol_short!("evt"), user_topic), 1u32);
    }
}
