#![no_std]
//! Vulnerable fixture for `batch-partial-write`.
//!
//! `batch_update` writes each item to storage **before** validating it.
//! If item[2] fails validation, items[0] and items[1] are already persisted —
//! the batch is partially applied with no rollback.
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct BatchPartialWriteVulnerable;

#[contractimpl]
impl BatchPartialWriteVulnerable {
    /// ❌ BUG: writes to storage first, then validates.
    /// A failing item leaves earlier items permanently written.
    pub fn batch_update(env: Env, values: Vec<u32>) -> Result<(), u32> {
        for val in values.iter() {
            // write happens before validation — partial state on early exit
            env.storage().persistent().set(&val, &val);
            if val == 0 {
                return Err(val); // some earlier items already written!
            }
        }
        Ok(())
    }

    /// ❌ BUG: uses `?` operator after write inside a loop.
    pub fn batch_apply(env: Env, keys: Vec<u32>) -> Result<(), u32> {
        for k in keys.iter() {
            env.storage().instance().set(&k, &k);
            let _check = validate(k)?;
        }
        Ok(())
    }
}

fn validate(v: u32) -> Result<u32, u32> {
    if v > 1000 { Err(v) } else { Ok(v) }
}
