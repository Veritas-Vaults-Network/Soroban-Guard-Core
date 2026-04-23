#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct UploadWasmAuthVulnerable;

#[contractimpl]
impl UploadWasmAuthVulnerable {
    /// Calls upload_contract_wasm without require_auth — should trigger check.
    pub fn upload(env: Env, wasm: soroban_sdk::Bytes) {
        env.deployer().upload_contract_wasm(wasm);
    }
}
