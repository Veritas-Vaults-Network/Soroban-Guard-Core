#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct SymbolShortLenSafe;

#[contractimpl]
impl SymbolShortLenSafe {
    pub fn test(env: Env) {
        // ✅ String is 3 characters, within limit
        let _ = symbol_short!("key");
        
        // ✅ String is exactly 9 characters, at the limit
        let _ = symbol_short!("123456789");
        
        // ✅ String is 5 characters
        let _ = symbol_short!("token");
        
        let _ = env;
    }
}
