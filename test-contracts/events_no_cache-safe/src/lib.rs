#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct EventsNoCacheSafe;

#[contractimpl]
impl EventsNoCacheSafe {
    pub fn process(env: Env) {
        // ✅ env.events() cached in local variable
        let events = env.events();
        events.publish(("event1",), ());
        events.publish(("event2",), ());
        events.publish(("event3",), ());
        events.publish(("event4",), ());
    }

    pub fn process_few(env: Env) {
        // ✅ env.events() called only 3 times (threshold is >3)
        env.events().publish(("event1",), ());
        env.events().publish(("event2",), ());
        env.events().publish(("event3",), ());
    }
}
