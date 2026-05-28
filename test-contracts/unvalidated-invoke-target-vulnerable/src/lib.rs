#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn call(env: Env, callee: Address) {
        // ❌ Uses caller-supplied contract address directly without validation.
        env.invoke_contract(&callee, &Symbol::new(&env, "method"), &());
    }
}
