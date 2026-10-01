#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FleetV1;

#[contractimpl]
impl FleetV1 {
    pub fn version(_env: Env) -> u32 {
        1
    }
}
