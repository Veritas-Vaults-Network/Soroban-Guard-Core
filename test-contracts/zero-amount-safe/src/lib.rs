#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ZeroAmountSafe;

#[contractimpl]
impl ZeroAmountSafe {
    /// Transfer with zero-amount check — safe.
    pub fn transfer(env: Env, _from: Address, _to: Address, amount: i128) -> Result<(), String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }
        emit_transfer_event(&env, amount);
        Ok(())
    }

    /// Mint with amount validation — safe.
    pub fn mint(_env: Env, _to: Address, amount: i128) -> Result<(), String> {
        if amount == 0 {
            return Err("Cannot mint zero amount".to_string());
        }
        // Proceed with minting
        Ok(())
    }

    /// Burn with amount check — safe.
    pub fn burn(env: Env, _from: Address, amount: i128) -> Result<(), String> {
        if amount <= 0 {
            return Err("Burn amount must be positive".to_string());
        }
        emit_burn_event(&env, amount);
        Ok(())
    }

    /// Generic function without amount param — not checked.
    pub fn transfer_ownership(_env: Env, _new_owner: Address) {
        // No amount parameter
    }
}

fn emit_transfer_event(_env: &Env, _amount: i128) {
    // Event emission
}

fn emit_burn_event(_env: &Env, _amount: i128) {
    // Event emission
}
