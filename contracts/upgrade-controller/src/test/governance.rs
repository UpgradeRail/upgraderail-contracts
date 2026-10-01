use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, BytesN, Env, String, Vec};

use crate::types::{
    CreateFleetProposal, GovernancePolicy, ProposalKind, ProposalState, UpdatePolicyProposal,
};
use crate::{UpgradeController, UpgradeControllerClient};

#[test]
fn policy_update_stales_old_proposals_and_changes_approvers() {
    let env = Env::default();
    env.mock_all_auths();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let replacement = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(first.clone());
    approvers.push_back(second.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 2,
        timelock_ledgers: 3,
        proposal_lifetime_ledgers: 100,
    };
    let contract = env.register(UpgradeController, (&policy,));
    let client = UpgradeControllerClient::new(&env, &contract);

    let old = client.create_proposal(
        &first,
        &ProposalKind::CreateFleet(CreateFleetProposal {
            fleet_id: BytesN::from_array(&env, &[1; 32]),
            tag: String::from_str(&env, "old-fleet"),
            initial_wasm_hash: BytesN::from_array(&env, &[2; 32]),
            manifest_hash: BytesN::from_array(&env, &[3; 32]),
        }),
    );
    let mut next_policy = policy;
    next_policy.approvers = Vec::new(&env);
    next_policy.approvers.push_back(replacement.clone());
    next_policy.threshold = 1;
    let update = client.create_proposal(
        &first,
        &ProposalKind::UpdatePolicy(UpdatePolicyProposal {
            policy: next_policy.clone(),
        }),
    );
    client.approve(&update, &first);
    client.approve(&update, &second);
    env.ledger().set_sequence_number(env.ledger().sequence() + 3);
    client.execute_proposal(&update);

    assert_eq!(client.get_governance_epoch(), 2);
    assert_eq!(client.get_policy(), next_policy);
    assert_eq!(client.get_proposal_state(&old), ProposalState::Stale);
    assert!(client.try_approve(&old, &replacement).is_err());
    assert!(client.try_execute_proposal(&old).is_err());
    assert!(client
        .try_create_proposal(&first, &ProposalKind::UpdatePolicy(UpdatePolicyProposal { policy: client.get_policy() }))
        .is_err());
    let fresh = client.create_proposal(
        &replacement,
        &ProposalKind::UpdatePolicy(UpdatePolicyProposal {
            policy: client.get_policy(),
        }),
    );
    assert_eq!(client.get_proposal(&fresh).governance_epoch, 2);
}
