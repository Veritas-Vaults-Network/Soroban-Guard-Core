#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, BytesN, Env};

#[contract]
pub struct UpgradeAtomicityOrderSafe;

const SCHEMA_VERSION: soroban_sdk::Symbol = symbol_short!("ver");

#[contractimpl]
impl UpgradeAtomicityOrderSafe {
    pub fn init(env: Env) {
        env.storage().instance().set(&SCHEMA_VERSION, &1u32);
    }

    /// The schema-version bump runs unconditionally before the WASM swap on
    /// every path through this function, regardless of `bump_schema` --
    /// there is no branch that can reach `update_current_contract_wasm`
    /// without writing the version key first.
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>, bump_schema: bool) {
        env.storage().instance().set(&SCHEMA_VERSION, &2u32);
        if bump_schema {
            env.events()
                .publish((symbol_short!("upgrade"),), env.ledger().timestamp());
        }
        env.deployer().update_current_contract_wasm(wasm_hash);
    }
}
