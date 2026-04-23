#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SymbolShortLenVulnerable;

#[contractimpl]
impl SymbolShortLenVulnerable {
    pub fn test(env: Env) {
        // ❌ String is 10 characters, exceeds 9 character limit
        let _ = symbol_short!("toolongkey");
        
        // ❌ String is 11 characters
        let _ = symbol_short!("verylongkey");
        
        let _ = env;
    }
}
