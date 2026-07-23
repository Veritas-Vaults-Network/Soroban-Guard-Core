#![no_std]
//! Safe fixture for `revoked-admin-reuse`.
//!
//! `set_admin` explicitly loads the `revoked_admins` Vec and asserts the new
//! address is not present before accepting it, preventing re-appointment of a
//! previously revoked admin.
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct RevokedAdminReuseSafe;

#[contractimpl]
impl RevokedAdminReuseSafe {
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

    /// ✅ SAFE: checks that new_admin is not in the revoked list before
    /// storing it as the current admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        env.require_auth();
        let revoked: Vec<Address> = env
            .storage()
            .persistent()
            .get(&"revoked_admins")
            .unwrap_or(Vec::new(&env));
        // Reject a previously revoked admin.
        if revoked.contains(&new_admin) {
            panic!("address was previously revoked and cannot be re-appointed as admin");
        }
        env.storage().persistent().set(&"admin", &new_admin);
    }

    /// Read the current admin.
    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&"admin").unwrap()
    }
}
