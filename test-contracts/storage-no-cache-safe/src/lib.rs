#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageNoCacheSafe;

const K1: Symbol = symbol_short!("k1");
const K2: Symbol = symbol_short!("k2");
const K3: Symbol = symbol_short!("k3");
const K4: Symbol = symbol_short!("k4");

#[contractimpl]
impl StorageNoCacheSafe {
    pub fn process(env: Env) {
        env.require_auth();
        // ✅ Cache storage in local variable
        let storage = env.storage().instance();
        storage.set(&K1, &1u32);
        storage.set(&K2, &2u32);
        storage.set(&K3, &3u32);
        storage.set(&K4, &4u32);
    }

    pub fn mixed_tiers(env: Env) {
        env.require_auth();
        // ✅ Cache each tier separately
        let instance = env.storage().instance();
        let persistent = env.storage().persistent();
        let temporary = env.storage().temporary();
        
        instance.set(&K1, &1u32);
        persistent.set(&K2, &2u32);
        temporary.set(&K3, &3u32);
        instance.set(&K4, &4u32);
    }

    pub fn few_calls(env: Env) {
        env.require_auth();
        // ✅ Only 3 calls, no caching needed
        env.storage().instance().set(&K1, &1u32);
        env.storage().instance().set(&K2, &2u32);
        env.storage().instance().set(&K3, &3u32);
    }
}
