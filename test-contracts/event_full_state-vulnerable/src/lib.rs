#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Symbol};

#[contract]
pub struct EventFullStateVulnerable;

#[contractimpl]
impl EventFullStateVulnerable {
    pub fn update_balance(env: Env, amount: i128) {
        let key = Symbol::new(&env, "balance");
        let old_balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        let new_balance = old_balance + amount;
        env.storage().persistent().set(&key, &new_balance);
        
        // ❌ Publishing full storage value instead of delta
        let full_value = env.storage().persistent().get(&key).unwrap_or(0);
        env.events().publish(("balance_updated",), (full_value,));
    }
}
