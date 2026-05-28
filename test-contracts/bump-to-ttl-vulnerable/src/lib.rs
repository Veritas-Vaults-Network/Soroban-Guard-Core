#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct BumpToTtlVulnerable;

const BALANCE: Symbol = symbol_short!("balance");
const OWNER: Symbol = symbol_short!("owner");

#[contractimpl]
impl BumpToTtlVulnerable {
    pub fn update_balance(env: Env, amount: u32) {
        env.require_auth();
        // ❌ bump_to_ttl only extends if TTL is below threshold
        env.storage().persistent().bump_to_ttl(&BALANCE, 100, 1000);
        env.storage().persistent().set(&BALANCE, &amount);
    }

    pub fn update_owner(env: Env, new_owner: soroban_sdk::Address) {
        env.require_auth();
        // ❌ bump_to_ttl on instance storage
        env.storage().instance().bump_to_ttl(&OWNER, 100, 1000);
        env.storage().instance().set(&OWNER, &new_owner);
    }
}
