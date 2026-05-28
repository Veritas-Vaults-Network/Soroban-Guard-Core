#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

#[contract]
pub struct OwnershipImmediateVulnerable;

#[contractimpl]
impl OwnershipImmediateVulnerable {
    /// ❌ Transfers ownership in a single step — if `new_admin` is wrong,
    /// ownership is permanently lost with no recovery path.
    pub fn set_admin(env: Env, new_admin: Address) {
        let current: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .unwrap();
        current.require_auth();
        // Direct single-step write — no two-step propose/accept pattern
        env.storage()
            .instance()
            .set(&symbol_short!("admin"), &new_admin);
    }
}
