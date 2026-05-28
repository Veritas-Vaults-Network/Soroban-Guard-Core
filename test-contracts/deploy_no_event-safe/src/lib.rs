#![no_std]

use soroban_sdk::{contractimpl, Env, BytesN, symbol_short};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn deploy_sub_contract(env: Env, wasm_hash: BytesN<32>, salt: BytesN<32>) {
        // Deploys a sub-contract and emits an event to notify indexers
        env.deployer().deploy(wasm_hash, salt);
        // Emit deployment event for off-chain indexing
        env.events().publish(("sub_contract_deployed",), (wasm_hash, salt));
    }
}