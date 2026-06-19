#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecMutateInLoopSafe;

#[contractimpl]
impl VecMutateInLoopSafe {
    pub fn process_items(env: Env, items: Vec<i32>) {
        // ✅ Collect values first, then mutate
        let mut new_items = Vec::new(&env);
        for item in items.iter() {
            new_items.push_back(item + 1);
        }
    }

    pub fn process_with_index(env: Env, items: Vec<i32>) {
        // ✅ Iterate by index and mutate different vec
        let mut results = Vec::new(&env);
        for i in 0..items.len() {
            let item = items.get(i).unwrap_or(0);
            results.push_back(item + 1);
        }
    }

    pub fn read_only_iteration(env: Env, items: Vec<i32>) {
        // ✅ Just reading, no mutations
        for item in items.iter() {
            let _ = item;
        }
    }
}
