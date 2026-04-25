#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

const KEY: u32 = 0;

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn call(env: Env, callee: Address) {
        let stored_admin = env.storage().persistent().get(&KEY).unwrap();
        if stored_admin == callee {
            env.invoke_contract(&callee, &Symbol::new(&env, "method"), &());
        }
    }
}
