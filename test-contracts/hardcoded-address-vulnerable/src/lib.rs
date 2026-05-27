#![no_std]

use soroban_sdk::{contractimpl, Env, Address};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn transfer_to_admin(env: Env, amount: i128) {
        // Hardcoded admin address - security and maintenance risk
        let admin_address = "GABC1234567890123456789012345678901234567890123456789012";
        let admin = Address::from_string(&admin_address);
        // ... transfer logic would go here
    }

    pub fn check_authorized_user(env: Env, user: Address) -> bool {
        // Another hardcoded address
        let authorized_user = "GDEF1234567890123456789012345678901234567890123456789012";
        user.to_string() == authorized_user
    }
}