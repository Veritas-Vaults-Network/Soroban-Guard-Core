#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct SupplyCapVulnerable;

#[contractimpl]
impl SupplyCapVulnerable {
    /// BUG: mints tokens without checking against max_supply cap.
    /// An attacker (or admin) can mint unlimited tokens, breaking tokenomics.
    pub fn mint(env: Env, _to: Address, amount: i128) {
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
