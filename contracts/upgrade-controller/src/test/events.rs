use soroban_sdk::{
    testutils::{Address as _, Events as _},
    vec, Address, BytesN, Env, IntoVal, Map, String, Symbol, Val, Vec,
};

use crate::types::{CreateFleetProposal, GovernancePolicy, ProposalKind};
use crate::{UpgradeController, UpgradeControllerClient};

#[test]
fn creation_and_approval_events_expose_stable_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let approver = Address::generate(&env);
    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());
    let policy = GovernancePolicy {
        approvers,
        threshold: 1,
        timelock_ledgers: 3,
        proposal_lifetime_ledgers: 100,
    };
    let controller = env.register(UpgradeController, (&policy,));
    let client = UpgradeControllerClient::new(&env, &controller);
    let expires_ledger = env.ledger().sequence() + policy.proposal_lifetime_ledgers;
    let id = client.create_proposal(
        &approver,
        &ProposalKind::CreateFleet(CreateFleetProposal {
            fleet_id: BytesN::from_array(&env, &[1; 32]),
            tag: String::from_str(&env, "event-fleet"),
            initial_wasm_hash: BytesN::from_array(&env, &[2; 32]),
            manifest_hash: BytesN::from_array(&env, &[3; 32]),
        }),
    );
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                controller.clone(),
                (Symbol::new(&env, "proposal_created"), id).into_val(&env),
                Map::<Symbol, Val>::from_array(
                    &env,
                    [
                        (
                            Symbol::new(&env, "proposer"),
                            approver.clone().into_val(&env)
                        ),
                        (Symbol::new(&env, "governance_epoch"), 1_u64.into_val(&env)),
                        (
                            Symbol::new(&env, "expires_ledger"),
                            expires_ledger.into_val(&env)
                        ),
                    ]
                )
                .into_val(&env),
            )
        ]
    );
    client.approve(&id, &approver);
    let approved_ledger = env.ledger().sequence();
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                controller.clone(),
                (Symbol::new(&env, "threshold_reached"), id).into_val(&env),
                Map::<Symbol, Val>::from_array(
                    &env,
                    [
                        (
                            Symbol::new(&env, "approved_ledger"),
                            approved_ledger.into_val(&env)
                        ),
                        (
                            Symbol::new(&env, "execute_after_ledger"),
                            (approved_ledger + 3).into_val(&env)
                        ),
                    ]
                )
                .into_val(&env),
            ),
            (
                controller,
                (Symbol::new(&env, "proposal_approved"), id, approver).into_val(&env),
                Map::<Symbol, Val>::from_array(
                    &env,
                    [(Symbol::new(&env, "approval_count"), 1_u32.into_val(&env)),]
                )
                .into_val(&env),
            ),
        ]
    );
}
