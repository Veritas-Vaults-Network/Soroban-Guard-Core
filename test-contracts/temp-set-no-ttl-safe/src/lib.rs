#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct TempSetNoTtlSafe;

/// Safe: calls extend_ttl immediately after writing to temporary storage.
#[contractimpl]
impl TempSetNoTtlSafe {
    pub fn acquire_lock(env: Env) {
        env.storage()
            .temporary()
            .set(&symbol_short!("lock"), &true);
        // Safe: TTL is explicitly extended to match the intended validity window.
        env.storage()
            .temporary()
            .extend_ttl(&symbol_short!("lock"), 100, 200);
    }
}
