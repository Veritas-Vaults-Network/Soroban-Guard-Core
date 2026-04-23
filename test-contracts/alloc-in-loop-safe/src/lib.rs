#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    /// Allocates Vec before loop — should pass alloc-in-loop
    pub fn process(env: Env) {
        let mut v = Vec::new(&env);
        for i in 0..10 {
            v.push_back(i);
        }
    }
}
