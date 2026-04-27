#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecPushInLoopSafe;

const MAX_ITEMS: u32 = 100;

/// Safe: push_back inside a loop guarded by a len() check.
#[contractimpl]
impl VecPushInLoopSafe {
    pub fn collect(env: Env, items: Vec<u32>) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new(&env);
        for item in items {
            // ✅ Length cap prevents unbounded growth.
            if out.len() >= MAX_ITEMS {
                break;
            }
            out.push_back(item);
        }
        out
    }
}
