#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct BrokenPauseVulnerable;

/// Vulnerable: pause() does nothing — no paused flag is written to storage.
#[contractimpl]
impl BrokenPauseVulnerable {
    pub fn pause(_env: Env) {
        // BUG: no storage write — circuit-breaker has no effect.
    }

    pub fn unpause(_env: Env) {
        // BUG: same — no flag cleared.
    }
}
