#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    /// ✅ Topic is a fixed symbol_short! constant — not user-controlled.
    pub fn emit_fixed_topic(env: Env) {
        env.events().publish((symbol_short!("transfer"),), true);
    }

    /// ✅ Topic is a named constant path — not user-controlled.
    pub fn emit_const_topic(env: Env) {
        const TOPIC: Symbol = symbol_short!("mint");
        env.events().publish((TOPIC,), 1u32);
    }

    /// ✅ Topic built with Symbol::new — a call expression, treated as safe.
    pub fn emit_symbol_new_topic(env: Env) {
        env.events()
            .publish((Symbol::new(&env, "approve"),), true);
    }
}
