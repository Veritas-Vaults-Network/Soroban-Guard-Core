#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    /// Allocates Vec inside loop — should trigger alloc-in-loop
    pub fn process(env: Env) {
        for i in 0..10 {
            let v = Vec::new(&env);
            v.push_back(i);
        }
    }
}
