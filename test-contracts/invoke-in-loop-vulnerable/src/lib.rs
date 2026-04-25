#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct InvokeInLoopVulnerable;

#[contractimpl]
impl InvokeInLoopVulnerable {
    pub fn attack(env: Env) {
        for _ in 0..3 {
            env.invoke_contract(&env, &env, &(), &());
        }
    }
}
