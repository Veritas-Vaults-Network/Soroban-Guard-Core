#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecMutateInLoopVulnerable;

#[contractimpl]
impl VecMutateInLoopVulnerable {
    pub fn process_items(env: Env, items: Vec<i32>) {
        // ❌ Mutating items while iterating over it
        for item in items.iter() {
            items.push_back(item + 1);
        }
    }

    pub fn remove_items(env: Env, items: Vec<i32>) {
        // ❌ Removing items while iterating
        for item in items.iter() {
            items.remove(0);
        }
    }

    pub fn set_items(env: Env, items: Vec<i32>) {
        // ❌ Setting items while iterating
        for item in items.iter() {
            items.set(0, item + 1);
        }
    }
}
