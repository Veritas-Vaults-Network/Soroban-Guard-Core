#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct EventsNoCacheVulnerable;

#[contractimpl]
impl EventsNoCacheVulnerable {
    pub fn process(env: Env) {
        // ❌ env.events() called 4 times without caching
        env.events().publish(("event1",), ());
        env.events().publish(("event2",), ());
        env.events().publish(("event3",), ());
        env.events().publish(("event4",), ());
    }
}
