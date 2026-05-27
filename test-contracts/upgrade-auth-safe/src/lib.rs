#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct UpgradeAuthSafe;

#[contractimpl]
impl UpgradeAuthSafe {
    /// ✅ Only the authorized caller can upgrade the contract WASM.
    pub fn upgrade(env: Env, new_wasm: BytesN<32>) {
        env.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm);
    }
}
