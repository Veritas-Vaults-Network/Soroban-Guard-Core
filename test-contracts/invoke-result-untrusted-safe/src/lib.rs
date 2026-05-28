#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val};

#[contract]
pub struct SafeInvokeResultUntrusted;

#[contractimpl]
impl SafeInvokeResultUntrusted {
    pub fn call(env: Env, contract: Address) -> Option<i128> {
        let result: Val = env.invoke_contract(&contract, &Symbol::short("get"), &());
        match result.try_into_val(&env) {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}
