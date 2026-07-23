#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct InterproceduralSupplyCapVulnerable;

#[contractimpl]
impl InterproceduralSupplyCapVulnerable {
    /// Safe path: enforces max_supply before minting.
    pub fn mint(env: Env, _to: Address, amount: i128) {
        let supply: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("supply"))
            .unwrap_or(0);
        let max_supply: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("max_supply"))
            .unwrap();
        assert!(supply + amount <= max_supply, "exceeds max supply");
        env.storage()
            .persistent()
            .set(&symbol_short!("supply"), &(supply + amount));
    }

    /// BUG: a second, independent path to the same total-supply key with no cap check.
    /// `mint` enforces `max_supply`, but this "emergency" entrypoint bypasses it entirely.
    pub fn emergency_mint(env: Env, _to: Address, amount: i128) {
        let supply: i128 = env
            .storage()
            .persistent()
            .get(&symbol_short!("supply"))
            .unwrap_or(0);
        // Missing: assert!(supply + amount <= max_supply);
        env.storage()
            .persistent()
            .set(&symbol_short!("supply"), &(supply + amount));
    }
}
