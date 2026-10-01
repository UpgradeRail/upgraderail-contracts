use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::storage;
use crate::types::{
    CreateFleetProposal, GovernancePolicy, ProposalKind, StoredProposalStatus,
    UpdatePolicyProposal, UpgradeControllerProposal, UpgradeFleetProposal,
};
use crate::{UpgradeController, UpgradeControllerClient};

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    let outsider = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(first.clone());
    approvers.push_back(second.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 2,
        timelock_ledgers: 5,
        proposal_lifetime_ledgers: 100,
    };
    let id = env.register(UpgradeController, (&policy,));
    (env, id, first, outsider)
}

fn create_fleet(env: &Env) -> ProposalKind {
    ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: BytesN::from_array(env, &[1; 32]),
        tag: String::from_str(env, "fleet-a"),
        initial_wasm_hash: BytesN::from_array(env, &[2; 32]),
        manifest_hash: BytesN::from_array(env, &[3; 32]),
    })
}

#[test]
fn creates_distinct_proposals_without_implicit_approval() {
    let (env, id, approver, _) = setup();
    let client = UpgradeControllerClient::new(&env, &id);
    let first = client.create_proposal(&approver, &create_fleet(&env));
    let second = client.create_proposal(&approver, &create_fleet(&env));
    assert_eq!((first, second), (1, 2));

    let stored = env.as_contract(&id, || storage::proposal(&env, first).unwrap());
    assert_eq!(stored.proposer, approver);
    assert_eq!(stored.governance_epoch, 1);
    assert_eq!(stored.expires_ledger, stored.created_ledger + 100);
    assert_eq!(stored.approval_count, 0);
    assert_eq!(stored.approved_ledger, None);
    assert_eq!(stored.execute_after_ledger, None);
    assert_eq!(stored.status, StoredProposalStatus::Active);
}

#[test]
fn rejects_nonapprover_and_invalid_payloads() {
    let (env, id, approver, outsider) = setup();
    let client = UpgradeControllerClient::new(&env, &id);
    assert!(client.try_create_proposal(&outsider, &create_fleet(&env)).is_err());

    let ProposalKind::CreateFleet(mut payload) = create_fleet(&env) else {
        unreachable!();
    };
    payload.tag = String::from_str(&env, "");
    assert!(client
        .try_create_proposal(&approver, &ProposalKind::CreateFleet(payload.clone()))
        .is_err());
    payload.tag = String::from_str(&env, "fleet-a");
    payload.initial_wasm_hash = BytesN::from_array(&env, &[0; 32]);
    assert!(client
        .try_create_proposal(&approver, &ProposalKind::CreateFleet(payload))
        .is_err());

    let missing_fleet = ProposalKind::UpgradeFleet(UpgradeFleetProposal {
        fleet_id: BytesN::from_array(&env, &[9; 32]),
        expected_wasm_hash: BytesN::from_array(&env, &[2; 32]),
        new_wasm_hash: BytesN::from_array(&env, &[4; 32]),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    assert!(client.try_create_proposal(&approver, &missing_fleet).is_err());

    let mut invalid_policy = client.get_policy();
    invalid_policy.threshold = 0;
    assert!(client
        .try_create_proposal(
            &approver,
            &ProposalKind::UpdatePolicy(UpdatePolicyProposal {
                policy: invalid_policy,
            }),
        )
        .is_err());

    let invalid_version = ProposalKind::UpgradeController(UpgradeControllerProposal {
        expected_controller_version: 1,
        new_controller_version: 3,
        new_wasm_hash: BytesN::from_array(&env, &[4; 32]),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });
    assert!(client.try_create_proposal(&approver, &invalid_version).is_err());
}

#[test]
fn proposer_can_cancel_only_before_threshold() {
    let (env, id, proposer, outsider) = setup();
    let client = UpgradeControllerClient::new(&env, &id);
    let first = client.create_proposal(&proposer, &create_fleet(&env));
    assert!(client.try_cancel_proposal(&first, &outsider).is_err());
    client.cancel_proposal(&first, &proposer);
    assert_eq!(
        client.get_proposal_state(&first),
        crate::types::ProposalState::Cancelled
    );
    assert!(client.try_approve(&first, &proposer).is_err());

    let second = client.create_proposal(&proposer, &create_fleet(&env));
    let other_approver = client.get_policy().approvers.get(1).unwrap();
    client.approve(&second, &proposer);
    client.approve(&second, &other_approver);
    assert!(client.try_cancel_proposal(&second, &proposer).is_err());
}
