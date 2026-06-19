#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct ContracttypeSafe;

/// Has #[contracttype] — safe.
#[derive(Clone)]
#[soroban_sdk::contracttype]
pub struct UserInfo {
    pub id: u32,
    pub name: String,
}

/// Has #[contracttype] in derive — safe.
#[derive(Clone, soroban_sdk::contracttype)]
pub enum Status {
    Active,
    Inactive,
    Suspended,
}

#[contractimpl]
impl ContracttypeSafe {
    /// Stores UserInfo with proper contracttype attribute.
    pub fn store_user(env: Env, id: u32) {
        let user = UserInfo {
            id,
            name: "Alice".into(),
        };
        let storage_key = soroban_sdk::Symbol::new(&env, "user");
        env.storage().instance().set(&storage_key, &user);
    }

    /// Stores Status enum with proper contracttype attribute.
    pub fn store_status(env: Env) {
        let status = Status::Active;
        let storage_key = soroban_sdk::Symbol::new(&env, "status");
        env.storage().instance().set(&storage_key, &status);
    }
}
