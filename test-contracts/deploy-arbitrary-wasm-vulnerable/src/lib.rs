#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Bytes, BytesN};

#[contract]
pub struct DeployArbitraryWasmVulnerable;

#[contractimpl]
impl DeployArbitraryWasmVulnerable {
    /// Deploys arbitrary WASM using caller‑supplied hash – should trigger
    /// `deploy-arbitrary-wasm` (High).
    pub fn deploy_bytes(env: Env, wasm_hash: Bytes) {
        env.deployer().deploy(wasm_hash, ());
    }

    /// Deploys arbitrary WASM using caller‑supplied hash (BytesN) – should trigger.
    pub fn deploy_bytesn(env: Env, wasm_hash: BytesN<32>) {
        env.deployer().deploy_v2(wasm_hash, (), vec![]);
    }
}
