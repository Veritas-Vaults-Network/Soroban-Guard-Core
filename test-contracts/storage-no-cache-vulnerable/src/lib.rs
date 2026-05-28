#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct StorageNoCacheVulnerable;

const K1: Symbol = symbol_short!("k1");
const K2: Symbol = symbol_short!("k2");
const K3: Symbol = symbol_short!("k3");
const K4: Symbol = symbol_short!("k4");

#[contractimpl]
impl StorageNoCacheVulnerable {
    pub fn process(env: Env) {
        env.require_auth();
        // ❌ Multiple env.storage().instance() calls without caching
        env.storage().instance().set(&K1, &1u32);
        env.storage().instance().set(&K2, &2u32);
        env.storage().instance().set(&K3, &3u32);
        env.storage().instance().set(&K4, &4u32);
    }

    pub fn mixed_tiers(env: Env) {
        env.require_auth();
        // ❌ Multiple storage calls across different tiers
        env.storage().instance().set(&K1, &1u32);
        env.storage().persistent().set(&K2, &2u32);
        env.storage().temporary().set(&K3, &3u32);
        env.storage().instance().set(&K4, &4u32);
    }
}
