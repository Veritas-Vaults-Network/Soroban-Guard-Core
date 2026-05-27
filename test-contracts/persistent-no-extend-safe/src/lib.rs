#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

const ADMIN_KEY: &str = "admin";
const TTL_THRESHOLD: u32 = 100_000;
const TTL_EXTEND_TO: u32 = 200_000;

#[contract]
pub struct SafeContract;

#[contractimpl]
impl SafeContract {
    // ✅ Persistent entry written and TTL extended in the same call
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&symbol_short!("admin"), &admin);
        env.storage()
            .persistent()
            .extend_ttl(&symbol_short!("admin"), TTL_THRESHOLD, TTL_EXTEND_TO);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&symbol_short!("admin"))
            .unwrap()
    }
}
