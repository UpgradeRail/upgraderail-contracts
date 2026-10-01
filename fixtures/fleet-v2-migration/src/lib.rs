#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FleetV2Migration;

#[contractimpl]
impl FleetV2Migration {
    pub fn version(_env: Env) -> u32 {
        2
    }
}
