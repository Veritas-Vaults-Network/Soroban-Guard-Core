#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct SafeDeploySaltPredictable;

#[contractimpl]
impl SafeDeploySaltPredictable {
    pub fn deploy(env: Env, wasm_hash: Bytes) {
        env.deployer().deploy(wasm_hash, ());
    }
}
