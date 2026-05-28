#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct EventFullStateSafe;

#[contractimpl]
impl EventFullStateSafe {
    pub fn update_balance(env: Env, amount: i128) {
        let key = Symbol::new(&env, "balance");
        let old_balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_balance = old_balance + amount;
        env.storage().persistent().set(&key, &new_balance);
        
        // ✅ Publishing only the delta (amount changed)
        env.events().publish(("balance_updated",), (amount,));
    }

    pub fn update_with_literal(env: Env, amount: i128) {
        let key = Symbol::new(&env, "balance");
        let old_balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_balance = old_balance + amount;
        env.storage().persistent().set(&key, &new_balance);
        
        // ✅ Publishing literal data, not storage value
        env.events().publish(("balance_updated",), (42,));
    }
}
