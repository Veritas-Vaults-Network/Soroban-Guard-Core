#![no_std]

use soroban_sdk::{contractimpl, Env};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn calculate_fee(env: Env, total_amount: i128, fee_percent: i128, multiplier: i128) -> i128 {
        // This will truncate the division result before multiplying
        (total_amount / 100) * fee_percent * multiplier
    }

    pub fn split_reward(env: Env, total_reward: i128, num_participants: i128, bonus_multiplier: i128) -> i128 {
        // Division before multiplication - precision loss
        (total_reward / num_participants) * bonus_multiplier
    }
}