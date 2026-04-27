#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VecPushInLoopVulnerable;

/// Vulnerable: push_back inside a loop with no len() guard — unbounded growth.
#[contractimpl]
impl VecPushInLoopVulnerable {
    pub fn collect(env: Env, items: Vec<u32>) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new(&env);
        for item in items {
            // BUG: no length cap — caller can grow `out` without bound,
            // exhausting the Soroban host resource budget (DoS).
            out.push_back(item);
        }
        out
    }
}
