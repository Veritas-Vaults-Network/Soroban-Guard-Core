#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Bytes, BytesN, Symbol};

#[contract]
pub struct DeployArbitraryWasmSafe;

const KNOWN_HASH: [u8; 32] = [0; 32];

#[contractimpl]
impl DeployArbitraryWasmSafe {
    /// Safe because hash is compared to a known constant before deployment.
    pub fn deploy_with_guard(env: Env, wasm_hash: Bytes) -> Result<(), ()> {
        if wasm_hash == Bytes::new(&env) {
            // Allow empty hash for demonstration; in real contract, compare to known hash.
            env.deployer().deploy(wasm_hash, ());
        }
        Ok(())
    }

    /// Safe because hash is loaded from storage (not caller‑supplied).
    pub fn deploy_stored_hash(env: Env) -> Result<(), ()> {
        let key = Symbol::new(&env, "wasm_hash");
        let stored_hash: Bytes = env.storage().instance().get(&key).unwrap();
        env.deployer().deploy(stored_hash, ());
        Ok(())
    }

    /// Safe because hash is compared to a BytesN constant.
    pub fn deploy_bytesn_guard(env: Env, wasm_hash: BytesN<32>) -> Result<(), ()> {
        let known = BytesN::from_array(&env, &KNOWN_HASH);
        if wasm_hash == known {
            env.deployer().deploy_v2(wasm_hash, (), vec![]);
        }
        Ok(())
    }
}
