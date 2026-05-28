#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct UnauthFeeSetterVulnerable;

const FEE_KEY: Symbol = symbol_short!("fee");

/// Vulnerable: set_fee writes to storage without require_auth.
/// Any caller can manipulate the protocol fee.
#[contractimpl]
impl UnauthFeeSetterVulnerable {
    pub fn set_fee(env: Env, fee: u32) {
        // BUG: no require_auth — anyone can change the fee.
        env.storage().instance().set(&FEE_KEY, &fee);
    }

    pub fn get_fee(env: Env) -> u32 {
        env.storage().instance().get(&FEE_KEY).unwrap_or(0)
    }
}
