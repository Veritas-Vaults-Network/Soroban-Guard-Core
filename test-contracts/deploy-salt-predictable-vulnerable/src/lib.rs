#![no_std]
use soroban_sdk::{contract, contractimpl, Bytes, Env};

#[contract]
pub struct VulnerableDeploySaltPredictable;

#[contractimpl]
impl VulnerableDeploySaltPredictable {
    pub fn deploy(env: Env, wasm_hash: Bytes) {
        let salt = env.ledger().sequence();
        env.deployer().deploy(wasm_hash, salt);
    }
}
