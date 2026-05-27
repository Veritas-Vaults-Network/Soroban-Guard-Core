#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct VulnerableContract;

#[contractimpl]
impl VulnerableContract {
    pub fn call_other(env: Env, contract: Address) {
        // ❌ Return value of invoke_contract is ignored
        env.invoke_contract(
            &contract,
            &symbol_short!("method"),
            &(),
        );
    }

    pub fn call_other_wildcard(env: Env, contract: Address) {
        // ❌ Return value bound to _ is still ignored
        let _ = env.invoke_contract(
            &contract,
            &symbol_short!("method"),
            &(),
        );
    }
}
