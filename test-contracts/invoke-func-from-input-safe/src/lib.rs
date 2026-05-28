#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

#[contract]
pub struct SafeInvokeFuncFromInput;

#[contractimpl]
impl SafeInvokeFuncFromInput {
    pub fn call(env: Env, contract: Address) {
        let func_name = Symbol::short("get");
        env.invoke_contract(&contract, &func_name, &());
    }
}
