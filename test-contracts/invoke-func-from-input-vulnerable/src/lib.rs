#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct VulnerableInvokeFuncFromInput;

#[contractimpl]
impl VulnerableInvokeFuncFromInput {
    pub fn call(env: Env, contract: Address, user_func: String) {
        let func_name = Symbol::from_str(&env, &user_func);
        env.invoke_contract(&contract, &func_name, &());
    }
}
