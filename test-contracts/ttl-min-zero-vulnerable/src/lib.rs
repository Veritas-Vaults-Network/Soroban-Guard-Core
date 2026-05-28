#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

#[contract]
pub struct TtlMinZeroVulnerable;

const BALANCE: Symbol = symbol_short!("balance");
const CONFIG: Symbol = symbol_short!("config");

#[contractimpl]
impl TtlMinZeroVulnerable {
    pub fn set_balance(env: Env, amount: i128) {
        env.storage().persistent().set(&BALANCE, &amount);
        // ❌ min = 0: the extension only fires when remaining TTL < 0,
        //    which never happens — this call is always a no-op.
        env.storage().persistent().extend_ttl(&BALANCE, 0, 10000);
    }

    pub fn refresh_instance(env: Env) {
        // ❌ Same problem on instance storage.
        env.storage().instance().extend_ttl(0, 5000);
    }

    pub fn init(env: Env, val: u32) {
        env.storage().persistent().set(&CONFIG, &val);
        // ❌ Looks intentional but provides no real TTL guarantee.
        env.storage().persistent().extend_ttl(&CONFIG, 0, 17280);
    }
}
