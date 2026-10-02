use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::types::{CreateFleetProposal, GovernancePolicy, ProposalKind, ProposalState};
use crate::{UpgradeController, UpgradeControllerClient};

#[test]
fn proposer_can_cancel_only_before_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let proposer = Address::generate(&env);
    let other_approver = Address::generate(&env);
    let outsider = Address::generate(&env);
    let policy = GovernancePolicy {
        approvers: Vec::from_array(&env, [proposer.clone(), other_approver.clone()]),
        threshold: 2,
        timelock_ledgers: 5,
        proposal_lifetime_ledgers: 100,
    };
    let contract = env.register(UpgradeController, (&policy,));
    let client = UpgradeControllerClient::new(&env, &contract);
    let kind = ProposalKind::CreateFleet(CreateFleetProposal {
        fleet_id: BytesN::from_array(&env, &[1; 32]),
        tag: String::from_str(&env, "cancellable-fleet"),
        initial_wasm_hash: BytesN::from_array(&env, &[2; 32]),
        manifest_hash: BytesN::from_array(&env, &[3; 32]),
    });

    let first = client.create_proposal(&proposer, &kind);
    assert!(client.try_cancel_proposal(&first, &outsider).is_err());
    client.cancel_proposal(&first, &proposer);
    assert_eq!(client.get_proposal_state(&first), ProposalState::Cancelled);
    assert!(client.try_approve(&first, &proposer).is_err());
    assert!(client.try_execute_proposal(&first).is_err());

    let second = client.create_proposal(&proposer, &kind);
    client.approve(&second, &proposer);
    client.approve(&second, &other_approver);
    assert!(client.try_cancel_proposal(&second, &proposer).is_err());
}
