#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FleetV2Breaking;

#[contractimpl]
impl FleetV2Breaking {
    pub fn version(_env: Env) -> u32 {
        2
    }
}
