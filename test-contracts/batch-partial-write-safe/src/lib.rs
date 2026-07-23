#![no_std]
//! Safe fixture for `batch-partial-write`.
//!
//! Uses the two-pass idiom: validate **all** elements in a first loop
//! (no writes), then write **all** elements in a second loop (no exits).
//! If any element is invalid, the function returns before any write occurs.
use soroban_sdk::{contract, contractimpl, Env, Vec};

#[contract]
pub struct BatchPartialWriteSafe;

#[contractimpl]
impl BatchPartialWriteSafe {
    /// ✅ SAFE: two-pass — validate all first, then write all.
    pub fn batch_update(env: Env, values: Vec<u32>) -> Result<(), u32> {
        // Pass 1: validate every element before touching storage
        for val in values.iter() {
            if val == 0 {
                return Err(val); // no storage written yet
            }
        }
        // Pass 2: all elements are valid, now write atomically
        for val in values.iter() {
            env.storage().persistent().set(&val, &val);
        }
        Ok(())
    }

    /// ✅ SAFE: exit guard comes before the write in the same iteration.
    pub fn guarded_store(env: Env, keys: Vec<u32>) -> Result<(), u32> {
        for k in keys.iter() {
            if k > 1000 {
                return Err(k); // exit before any write
            }
            env.storage().persistent().set(&k, &k);
        }
        Ok(())
    }
}
