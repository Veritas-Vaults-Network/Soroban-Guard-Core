#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val};

#[contract]
pub struct VulnerableInvokeResultUntrusted;

#[contractimpl]
impl VulnerableInvokeResultUntrusted {
    pub fn call(env: Env, contract: Address) -> i128 {
        env.invoke_contract::<Val>(&contract, &Symbol::short("get"), &())
            .try_into_val(&env)
    }
}
