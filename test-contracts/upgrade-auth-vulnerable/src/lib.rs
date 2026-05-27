#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct UpgradeAuthVulnerable;

#[contractimpl]
impl UpgradeAuthVulnerable {
    /// ❌ No require_auth — any account on Stellar can replace the contract WASM.
    pub fn upgrade(env: Env, new_wasm: BytesN<32>) {
        env.deployer().update_current_contract_wasm(new_wasm);
    }
}
