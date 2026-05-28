#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecPopUnwrapSafe;

/// Safe: handle the `Option` returned by `pop_front` / `pop_back` explicitly.
#[contractimpl]
impl VecPopUnwrapSafe {
    pub fn dequeue(env: Env, mut queue: Vec<u32>) -> u32 {
        // ✅ Returns a default value instead of panicking on empty Vec.
        queue.pop_front().unwrap_or(0)
    }

    pub fn pop_last(env: Env, mut stack: Vec<u32>) -> u32 {
        // ✅ Match on the Option to handle the empty case gracefully.
        match stack.pop_back() {
            Some(v) => v,
            None => 0,
        }
    }
}
