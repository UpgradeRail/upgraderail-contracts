#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

use crate::errors::ContractError;
use crate::types::GovernancePolicy;

pub mod types;
pub mod errors;
pub mod storage;
pub mod policy;

#[cfg(test)]
mod test;

#[contract]
pub struct UpgradeController;

#[contractimpl]
impl UpgradeController {
    pub fn __constructor(env: Env, policy: GovernancePolicy) -> Result<(), ContractError> {
        policy::validate(&env, &policy)?;
        storage::set_policy(&env, &policy);
        storage::set_epoch(&env, 1);
        storage::set_nonce(&env, 0);
        storage::set_version(&env, 1);
        let max_ttl = env.storage().max_ttl();
        env.storage()
            .instance()
            .extend_ttl_with_limits(max_ttl, 1, max_ttl);
        Ok(())
    }

    pub fn get_policy(env: Env) -> GovernancePolicy {
        // The constructor writes this key before deployment can complete.
        storage::policy(&env).expect("initialized policy")
    }

    pub fn get_governance_epoch(env: Env) -> u64 {
        // The constructor writes this key before deployment can complete.
        storage::epoch(&env).expect("initialized governance epoch")
    }

    pub fn get_controller_version(env: Env) -> u32 {
        // The constructor writes this key before deployment can complete.
        storage::version(&env).expect("initialized controller version")
    }
}
