#![no_std]

use soroban_sdk::{contractimpl, Env, Address, Symbol, symbol_short};

pub struct Contract;

const ADMIN_KEY: Symbol = symbol_short!("admin");
const AUTH_USER_KEY: Symbol = symbol_short!("auth_user");

#[contractimpl]
impl Contract {
    pub fn initialize(env: Env, admin: Address, authorized_user: Address) {
        env.storage().persistent().set(&ADMIN_KEY, &admin);
        env.storage().persistent().set(&AUTH_USER_KEY, &authorized_user);
    }

    pub fn transfer_to_admin(env: Env, amount: i128) {
        // Get admin address from storage
        let admin: Address = env.storage().persistent().get(&ADMIN_KEY).unwrap();
        // ... transfer logic would go here
    }

    pub fn check_authorized_user(env: Env, user: Address) -> bool {
        // Get authorized user from storage
        let authorized_user: Address = env.storage().persistent().get(&AUTH_USER_KEY).unwrap();
        user == authorized_user
    }

    pub fn transfer_to_address(env: Env, to: Address, amount: i128) {
        // Address passed as parameter - safe
        // ... transfer logic would go here
    }
}