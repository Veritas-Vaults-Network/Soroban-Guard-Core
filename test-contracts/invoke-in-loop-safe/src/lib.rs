#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct InvokeInLoopSafe;

#[contractimpl]
impl InvokeInLoopSafe {
    pub fn action(env: Env) {
        env.invoke_contract(&env, &env, &(), &());
    }
}
