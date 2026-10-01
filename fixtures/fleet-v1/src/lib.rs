#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Env};

#[contract]
pub struct FleetV1;

#[contractimpl]
impl FleetV1 {
    pub fn __constructor(env: Env) {
        env.storage().instance().set(&symbol_short!("value"), &0_i64);
    }

    pub fn set_value(env: Env, value: i64) {
        env.storage().instance().set(&symbol_short!("value"), &value);
    }

    pub fn get_value(env: Env) -> i64 {
        env.storage()
            .instance()
            .get(&symbol_short!("value"))
            .expect("value initialized by constructor")
    }

    pub fn version(_env: Env) -> u32 {
        1
    }
}
