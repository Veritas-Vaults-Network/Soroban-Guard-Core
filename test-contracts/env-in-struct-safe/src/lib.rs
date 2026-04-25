#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

pub struct SafeStruct {
    count: u32,
}

#[contract]
pub struct EnvInStructSafe;

#[contractimpl]
impl EnvInStructSafe {
    pub fn test(_env: Env) {
        let _ = SafeStruct { count: 1 };
    }
}
