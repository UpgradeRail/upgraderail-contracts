#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigratedValue {
    pub value: i64,
    pub migrated_ledger: u32,
}

#[contract]
pub struct FleetV2Migration;

#[contractimpl]
impl FleetV2Migration {
    pub fn migrate(env: Env) -> bool {
        if env.storage().instance().has(&symbol_short!("state")) {
            return false;
        }
        let Some(value) = env
            .storage()
            .instance()
            .get::<_, i64>(&symbol_short!("value"))
        else {
            return false;
        };
        env.storage().instance().set(
            &symbol_short!("state"),
            &MigratedValue {
                value,
                migrated_ledger: env.ledger().sequence(),
            },
        );
        env.storage().instance().remove(&symbol_short!("value"));
        true
    }

    pub fn get_value(env: Env) -> Option<i64> {
        env.storage()
            .instance()
            .get::<_, MigratedValue>(&symbol_short!("state"))
            .map(|state| state.value)
    }

    pub fn migration_ledger(env: Env) -> Option<u32> {
        env.storage()
            .instance()
            .get::<_, MigratedValue>(&symbol_short!("state"))
            .map(|state| state.migrated_ledger)
    }

    pub fn version(_env: Env) -> u32 {
        2
    }
}
