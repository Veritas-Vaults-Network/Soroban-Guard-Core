#![no_std]

use soroban_sdk::{contractimpl, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn calculate_fee(env: Env, total_amount: i128, fee_percent: i128, multiplier: i128) -> i128 {
        // Multiply before dividing to maintain precision
        (total_amount * fee_percent * multiplier) / 100
    }

    pub fn split_reward(env: Env, total_reward: i128, num_participants: i128, bonus_multiplier: i128) -> i128 {
        // Multiply before dividing to avoid truncation
        (total_reward * bonus_multiplier) / num_participants
    }
}