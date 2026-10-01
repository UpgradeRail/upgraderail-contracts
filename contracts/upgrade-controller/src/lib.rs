#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::errors::ContractError;
use crate::types::{Fleet, GovernancePolicy, Proposal, ProposalKind, ProposalState};

pub mod approvals;
pub mod errors;
pub mod events;
pub mod fleets;
pub mod policy;
pub mod proposals;
pub mod storage;
pub mod ttl;
pub mod types;

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
        ttl::maintain_controller(&env)?;
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

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        kind: ProposalKind,
    ) -> Result<u64, ContractError> {
        let id = proposals::create(&env, proposer, kind)?;
        ttl::maintain_controller(&env)?;
        Ok(id)
    }

    pub fn approve(env: Env, proposal_id: u64, approver: Address) -> Result<(), ContractError> {
        approvals::approve(&env, proposal_id, approver)?;
        ttl::maintain_controller(&env)
    }

    pub fn revoke_approval(
        env: Env,
        proposal_id: u64,
        approver: Address,
    ) -> Result<(), ContractError> {
        approvals::revoke(&env, proposal_id, approver)?;
        ttl::maintain_controller(&env)
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        storage::proposal(&env, proposal_id).ok_or(ContractError::ProposalNotFound)
    }

    pub fn get_proposal_state(env: Env, proposal_id: u64) -> Result<ProposalState, ContractError> {
        proposals::state(&env, proposal_id)
    }

    pub fn has_approved(env: Env, proposal_id: u64, approver: Address) -> bool {
        storage::has_approval(&env, proposal_id, &approver)
    }

    pub fn cancel_proposal(
        env: Env,
        proposal_id: u64,
        proposer: Address,
    ) -> Result<(), ContractError> {
        proposals::cancel(&env, proposal_id, proposer)?;
        ttl::maintain_controller(&env)
    }

    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        proposals::execute(&env, proposal_id)?;
        ttl::maintain_controller(&env)
    }

    pub fn get_fleet(env: Env, fleet_id: BytesN<32>) -> Result<Fleet, ContractError> {
        storage::fleet(&env, &fleet_id).ok_or(ContractError::FleetNotFound)
    }

    pub fn get_current_wasm(env: Env, fleet_id: BytesN<32>) -> Result<BytesN<32>, ContractError> {
        fleets::current_wasm(&env, &fleet_id)
    }

    pub fn maintain_controller(env: Env) -> Result<(), ContractError> {
        ttl::maintain_controller(&env)
    }

    pub fn maintain_fleet(env: Env, fleet_id: BytesN<32>) -> Result<(), ContractError> {
        ttl::maintain_fleet(&env, &fleet_id)?;
        ttl::maintain_controller(&env)
    }

    pub fn maintain_proposal(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        ttl::maintain_proposal(&env, proposal_id)?;
        ttl::maintain_controller(&env)
    }
}
