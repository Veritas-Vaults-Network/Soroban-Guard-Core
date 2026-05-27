#![no_std]
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct DeployArgAuthSafe;

#[contractimpl]
impl DeployArgAuthSafe {
    /// ✅ require_auth_for_args binds auth to deploy arguments.
    pub fn deploy_contract(env: Env, wasm_hash: BytesN<32>, salt: u64) {
        env.require_auth_for_args((wasm_hash, salt));
        env.deployer().deploy(wasm_hash, salt);
    }

    /// ✅ Deploy with literal arguments (no parameters).
    pub fn deploy_fixed(env: Env) {
        env.require_auth();
        let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
        env.deployer().deploy(wasm_hash, 0u64);
    }
}
