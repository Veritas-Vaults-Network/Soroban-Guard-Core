#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct InvokeStoreNoEventVulnerable;

#[contractimpl]
impl InvokeStoreNoEventVulnerable {
    pub fn call_and_store(env: Env, target: Address) {
        // ❌ invoke_contract result stored without emitting event
        let result = env.invoke_contract::<i128>(&target, &"get_value", &());
        env.storage().persistent().set(&"cached_result", &result);
    }
}
