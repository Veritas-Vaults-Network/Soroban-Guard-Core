#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

#[contract]
pub struct WhitelistContract;

#[contractimpl]
impl WhitelistContract {
    /// ❌ O(n) linear scan over storage-backed Vec — DoS risk.
    pub fn is_allowed(env: Env, caller: Address) -> bool {
        let list: Vec<Address> = env
            .storage()
            .persistent()
            .get(&symbol_short!("wl"))
            .unwrap_or(Vec::new(&env));
        list.iter().any(|a| a == caller)
    }
}
