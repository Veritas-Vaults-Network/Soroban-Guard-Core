#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct PanicUsageVulnerable;

#[contractimpl]
impl PanicUsageVulnerable {
    /// Uses panic! macro — should trigger `panic-usage` (Low).
    pub fn risky_operation(env: Env) {
        panic!("Something went wrong");
    }

    /// Uses unreachable! macro — should trigger `panic-usage` (Low).
    pub fn impossible_path(env: Env) {
        if false {
            unreachable!("This should never happen");
        }
    }

    /// Panics conditionally — should still trigger `panic-usage` (Low).
    pub fn conditional_panic(env: Env, should_panic: bool) {
        if should_panic {
            panic!("User requested panic");
        }
    }
}
