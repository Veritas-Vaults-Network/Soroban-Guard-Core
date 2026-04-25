#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AssertForAuthVulnerable;

#[contractimpl]
impl AssertForAuthVulnerable {
    /// Uses assert! for access control with Address equality — vulnerable.
    pub fn vulnerable_auth_equal(env: Env, caller: Address) {
        assert!(caller == admin);
        // Do privileged operation
    }

    /// Uses assert! for access control with Address inequality — vulnerable.
    pub fn vulnerable_auth_not_equal(env: Env, caller: Address) {
        assert!(caller != admin);
        // Do operation
    }

    /// Another vulnerable pattern.
    pub fn vulnerable_admin_check(env: Env, user: Address) {
        assert!(user == admin);
        // Admin operation
    }
}