#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct BurnAuthSafe;

#[contractimpl]
impl BurnAuthSafe {
    /// ✅ Token owner must authorize the burn.
    pub fn burn(_env: Env, from: Address, _amount: i128) {
        from.require_auth();
    }

    /// ✅ Spender must be authorized to burn on behalf of `from`.
    pub fn burn_from(_env: Env, spender: Address, _from: Address, _amount: i128) {
        spender.require_auth();
    }
}
