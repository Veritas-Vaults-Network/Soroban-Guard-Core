#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env, Symbol};

#[contract]
pub struct SafeDeployUnverified;

#[contractimpl]
impl SafeDeployUnverified {
    pub fn deploy(env: Env, wasm_hash: Bytes) {
        let addr = env.deployer().deploy(wasm_hash, ());
        if env.is_contract(&addr) {
            env.storage().persistent().set(&Symbol::new(&env, "addr"), &addr);
        }
    }
}
