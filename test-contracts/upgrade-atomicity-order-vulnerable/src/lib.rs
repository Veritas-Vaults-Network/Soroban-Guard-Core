#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct UpgradeAtomicityOrderVulnerable;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl UpgradeAtomicityOrderVulnerable {
    pub fn init(env: Env) {
        env.storage().instance().set(&SCHEMA_VERSION, &1u32);
    }

    /// The schema-version bump only runs when `bump_schema` is true, but the
    /// WASM swap runs unconditionally right after. A caller that passes
    /// `bump_schema = false` upgrades the contract with no version write at
    /// all on that path, even though a `set(&SCHEMA_VERSION, ...)` call
    /// "exists in the file" — which is all the older substring-based
    /// `upgrade-no-schema-version` check looks for.
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, bump_schema: bool) {
        if bump_schema {
            env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        }
        env.deployer().update_current_contract_wasm(wasm_hash);
    }
}
