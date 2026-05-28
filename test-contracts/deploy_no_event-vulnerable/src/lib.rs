#![no_std]

use soroban_sdk::{contractimpl, Env, BytesN};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deploy_sub_contract(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>) {
        // Deploys a sub-contract without emitting an event
        env.deployer().deploy(wasm_hash, salt);
        // Missing env.events().publish() call to notify indexers of the deployment
    }
}