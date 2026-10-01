#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct FleetV2Compatible;

#[contractimpl]
impl FleetV2Compatible {
    pub fn version(_env: Env) -> u32 {
        2
    }
}
