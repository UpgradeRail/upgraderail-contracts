use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::types::GovernancePolicy;

pub const MAX_APPROVERS: u32 = 20;
pub const PROPOSAL_TTL_SAFETY_BUFFER: u32 = 17_280;

pub(crate) fn validate(env: &Env, policy: &GovernancePolicy) -> Result<(), ContractError> {
    let count = policy.approvers.len();
    if count == 0 {
        return Err(ContractError::NoApprovers);
    }
    if count > MAX_APPROVERS {
        return Err(ContractError::TooManyApprovers);
    }
    for i in 0..count {
        let first = policy
            .approvers
            .get(i)
            .ok_or(ContractError::InvalidPolicy)?;
        for j in (i + 1)..count {
            let second = policy
                .approvers
                .get(j)
                .ok_or(ContractError::InvalidPolicy)?;
            if first == second {
                return Err(ContractError::DuplicateApprover);
            }
        }
    }
    if policy.threshold == 0 || policy.threshold > count {
        return Err(ContractError::InvalidThreshold);
    }
    if policy.timelock_ledgers == 0 {
        return Err(ContractError::InvalidTimelock);
    }
    if policy.proposal_lifetime_ledgers <= policy.timelock_ledgers {
        return Err(ContractError::InvalidProposalLifetime);
    }
    let required_ttl = policy
        .proposal_lifetime_ledgers
        .checked_add(PROPOSAL_TTL_SAFETY_BUFFER)
        .ok_or(ContractError::InvalidProposalLifetime)?;
    if required_ttl > core::cmp::min(env.storage().max_ttl(), crate::ttl::MAX_TTL_EXTENSION) {
        return Err(ContractError::InvalidProposalLifetime);
    }
    Ok(())
}

pub(crate) fn is_approver(policy: &GovernancePolicy, candidate: &Address) -> bool {
    policy.approvers.iter().any(|address| address == *candidate)
}
