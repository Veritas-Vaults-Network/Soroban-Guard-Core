#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct UnauthFeeSetterSafe;

const FEE_KEY: Symbol = symbol_short!("fee");

/// Safe: set_fee requires admin authorization before writing to storage.
#[contractimpl]
impl UnauthFeeSetterSafe {
    pub fn set_fee(env: Env, admin: Address, fee: u32) {
        // ✅ Only the admin can update the fee.
        admin.require_auth();
        env.storage().instance().set(&FEE_KEY, &fee);
    }

    pub fn get_fee(env: Env) -> u32 {
        env.storage().instance().get(&FEE_KEY).unwrap_or(0)
    }
}
