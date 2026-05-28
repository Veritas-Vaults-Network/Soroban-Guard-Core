#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecPopUnwrapVulnerable;

/// Vulnerable: `.pop_back().unwrap()` and `.pop_front().unwrap()` panic on empty Vec.
#[contractimpl]
impl VecPopUnwrapVulnerable {
    pub fn dequeue(env: Env, mut queue: Vec<u32>) -> u32 {
        // BUG: panics if `queue` is empty.
        queue.pop_front().unwrap()
    }

    pub fn pop_last(env: Env, mut stack: Vec<u32>) -> u32 {
        // BUG: panics if `stack` is empty.
        stack.pop_back().unwrap()
    }
}
