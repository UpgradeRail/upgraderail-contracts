#![no_std]

use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct FleetV2Breaking;

#[contractimpl]
impl FleetV2Breaking {
    pub fn read_text(env: Env) -> String {
        String::from_str(&env, "value API removed")
    }

    pub fn version(_env: Env) -> u32 {
        2
    }
}
