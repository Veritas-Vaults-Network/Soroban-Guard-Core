#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct InterproceduralSupplyCapSafe;

#[contractimpl]
impl InterproceduralSupplyCapSafe {
    /// Both public entrypoints funnel through a single checked helper, so the cap
    /// enforced in `mint_checked` protects every path that can increase supply.
    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::mint_checked(env, to, amount);
    }

    pub fn emergency_mint(env: Env, to: Address, amount: i128) {
        Self::mint_checked(env, to, amount);
    }

    fn mint_checked(env: Env, _to: Address, amount: i128) {
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
}
