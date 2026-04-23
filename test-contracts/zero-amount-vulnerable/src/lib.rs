#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ZeroAmountVulnerable;

#[contractimpl]
impl ZeroAmountVulnerable {
    /// Transfer without zero-amount check — should trigger `zero-amount-transfer` (Low).
    pub fn transfer(env: Env, _from: Address, _to: Address, amount: i128) {
        // No validation that amount > 0
        emit_transfer_event(&env, amount);
    }

    /// Mint without amount check — should trigger `zero-amount-transfer` (Low).
    pub fn mint(env: Env, _to: Address, amount: i128) {
        emit_mint_event(&env, amount);
    }

    /// Burn without amount check — should trigger `zero-amount-transfer` (Low).
    pub fn burn(env: Env, _from: Address, amount: i128) {
        emit_burn_event(&env, amount);
    }

    /// Generic function without amount param — should not trigger.
    pub fn transfer_ownership(_env: Env, _new_owner: Address) {
        // No amount parameter, so not flagged
    }
}

fn emit_transfer_event(_env: &Env, _amount: i128) {
    // Event emission
}

fn emit_mint_event(_env: &Env, _amount: i128) {
    // Event emission
}

fn emit_burn_event(_env: &Env, _amount: i128) {
    // Event emission
}
