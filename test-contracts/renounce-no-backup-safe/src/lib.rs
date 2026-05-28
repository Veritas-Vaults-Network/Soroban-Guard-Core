#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct RenounceNoBackupSafe;

#[contractimpl]
impl RenounceNoBackupSafe {
    /// ✅ Transfers admin to a backup address before renouncing the current
    /// admin key — the contract always retains an authorized admin.
    pub fn renounce_ownership(env: Env, backup: Address) {
        let current: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        current.require_auth();
        // Set backup first, then remove the old key
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &backup);
    }
}
