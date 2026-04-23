#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    pub fn call_other(env: Env, contract: Address) {
        // ✅ Return value is captured and used
        let result = env.invoke_contract(
            &contract,
            &symbol_short!("method"),
            &(),
        );
        // Process result...
    }
}
