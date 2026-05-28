#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env, Symbol};

#[contract]
pub struct VulnerableDeployUnverified;

#[contractimpl]
impl VulnerableDeployUnverified {
    pub fn deploy(env: Env, wasm_hash: Bytes) {
        let addr = env.deployer().deploy(wasm_hash, ());
        env.storage().persistent().set(&Symbol::new(&env, "addr"), &addr);
    }
}
