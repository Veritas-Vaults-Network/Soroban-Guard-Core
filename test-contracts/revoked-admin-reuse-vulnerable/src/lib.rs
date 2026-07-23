#![no_std]
//! Vulnerable fixture for `revoked-admin-reuse`.
//!
//! This contract maintains a `revoked_admins` Vec for audit purposes, but
//! `set_admin` never checks whether the incoming address was previously revoked.
//! A previously-revoked admin can be silently re-appointed.
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct RevokedAdminReuseVulnerable;

#[contractimpl]
impl RevokedAdminReuseVulnerable {
    /// Record a revoked admin in persistent storage for compliance tracking.
    pub fn revoke_admin(env: Env, admin: Address) {
        env.require_auth();
        let mut revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or(Vec::new(&env));
        revoked.push_back(admin.clone());
        env.storage().persistent().set(&"revoked_admins", &revoked);
    }

    /// ❌ BUG: accepts any Address as the new admin without checking the
    /// revoked_admins list — a previously revoked admin can be re-appointed.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        // Missing: check that new_admin is not in revoked_admins
        env.storage().persistent().set(&"admin", &new_admin);
    }

    /// Read the current admin.
    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&"admin").unwrap()
    }
}
