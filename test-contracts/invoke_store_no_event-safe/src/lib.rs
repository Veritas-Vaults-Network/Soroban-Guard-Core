#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct InvokeStoreNoEventSafe;

#[contractimpl]
impl InvokeStoreNoEventSafe {
    pub fn call_and_store(env: Env, target: Address) {
        // ✅ invoke_contract result stored with event emission
        let result = env.invoke_contract::<i128>(&target, &"get_value", &());
        env.storage().persistent().set(&"cached_result", &result);
        env.events().publish(("result_cached",), (&result,));
    }

    pub fn call_without_store(env: Env, target: Address) {
        // ✅ invoke_contract result not stored
        let result = env.invoke_contract::<i128>(&target, &"get_value", &());
        let _ = result;
    }
}
