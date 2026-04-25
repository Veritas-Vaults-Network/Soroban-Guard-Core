#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct TempSetNoTtlVulnerable;

/// Vulnerable: writes to temporary storage without calling extend_ttl.
/// The entry uses the default TTL, which may expire before the lock is released.
#[contractimpl]
impl TempSetNoTtlVulnerable {
    pub fn acquire_lock(env: Env) {
        // BUG: no extend_ttl after set — entry may expire unexpectedly.
        env.storage()
            .temporary()
            .set(&symbol_short!("lock"), &true);
    }
}
