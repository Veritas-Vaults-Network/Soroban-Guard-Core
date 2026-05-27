#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct DeployArgAuthVulnerable;

#[contractimpl]
impl DeployArgAuthVulnerable {
    /// ❌ require_auth() doesn't bind to deploy arguments.
    /// Attacker can reuse auth to deploy with different wasm_hash or salt.
    pub fn deploy_contract(env: Env, wasm_hash: BytesN<32>, salt: u64) {
        env.require_auth();
        env.deployer().deploy(wasm_hash, salt);
    }
}
