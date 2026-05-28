#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map};

#[contract]
pub struct WhitelistContract;

#[contractimpl]
impl WhitelistContract {
    /// ✅ O(1) Map lookup — no linear scan.
    pub fn is_allowed(env: Env, caller: Address) -> bool {
        let map: Map<Address, bool> = env
            .storage()
            .persistent()
            .get(&symbol_short!("wl"))
            .unwrap_or(Map::new(&env));
        map.contains_key(caller)
    }
}
