#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

pub mod types;
pub mod errors;
pub mod storage;
pub mod policy;

#[contract]
pub struct UpgradeController;

#[contractimpl]
impl UpgradeController {
    pub fn version(_env: Env) -> u32 {
        1
    }
}
