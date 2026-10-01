use soroban_sdk::{Address, BytesN, Env, String};

use crate::types::{DataKey, Fleet, GovernancePolicy, Proposal};

pub(crate) fn set_policy(env: &Env, policy: &GovernancePolicy) {
    env.storage().instance().set(&DataKey::Policy, policy);
}

pub(crate) fn policy(env: &Env) -> Option<GovernancePolicy> {
    env.storage().instance().get(&DataKey::Policy)
}

pub(crate) fn set_epoch(env: &Env, epoch: u64) {
    env.storage()
        .instance()
        .set(&DataKey::GovernanceEpoch, &epoch);
}

pub(crate) fn epoch(env: &Env) -> Option<u64> {
    env.storage().instance().get(&DataKey::GovernanceEpoch)
}

pub(crate) fn set_nonce(env: &Env, nonce: u64) {
    env.storage()
        .instance()
        .set(&DataKey::ProposalNonce, &nonce);
}

pub(crate) fn nonce(env: &Env) -> Option<u64> {
    env.storage().instance().get(&DataKey::ProposalNonce)
}

pub(crate) fn set_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ControllerVersion, &version);
}

pub(crate) fn version(env: &Env) -> Option<u32> {
    env.storage().instance().get(&DataKey::ControllerVersion)
}

pub(crate) fn set_fleet(env: &Env, fleet: &Fleet) {
    env.storage()
        .persistent()
        .set(&DataKey::Fleet(fleet.id.clone()), fleet);
    env.storage()
        .persistent()
        .set(&DataKey::FleetByTag(fleet.tag.clone()), &fleet.id);
}

pub(crate) fn fleet(env: &Env, id: &BytesN<32>) -> Option<Fleet> {
    env.storage().persistent().get(&DataKey::Fleet(id.clone()))
}

pub(crate) fn tag_used(env: &Env, tag: &String) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::FleetByTag(tag.clone()))
}

pub(crate) fn set_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.id), proposal);
}

pub(crate) fn proposal(env: &Env, id: u64) -> Option<Proposal> {
    env.storage().persistent().get(&DataKey::Proposal(id))
}

pub(crate) fn set_approval(env: &Env, id: u64, approver: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::Approval(id, approver.clone()), &true);
}

pub(crate) fn has_approval(env: &Env, id: u64, approver: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Approval(id, approver.clone()))
}

pub(crate) fn remove_approval(env: &Env, id: u64, approver: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::Approval(id, approver.clone()));
}
