#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct UploadWasmAuthSafe;

#[contractimpl]
impl UploadWasmAuthSafe {
    /// Calls upload_contract_wasm with require_auth — should pass.
    pub fn upload(env: Env, wasm: soroban_sdk::Bytes) {
        env.require_auth();
        env.deployer().upload_contract_wasm(wasm);
    }
}
