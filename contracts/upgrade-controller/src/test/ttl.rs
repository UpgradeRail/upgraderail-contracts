use soroban_sdk::{
    testutils::{storage::{Instance as _, Persistent as _}, Address as _, Ledger as _},
    Address, BytesN, Env, String, Vec,
};

use crate::policy::PROPOSAL_TTL_SAFETY_BUFFER;
use crate::ttl::{INSTANCE_TTL_TARGET, PERSISTENT_RECORD_TTL_TARGET};
use crate::types::{CreateFleetProposal, DataKey, GovernancePolicy, ProposalKind};
use crate::{UpgradeController, UpgradeControllerClient};

const V1_WASM: &[u8] = include_bytes!("../../../../fixtures/wasm/fleet_v1.wasm");

#[test]
fn maintenance_retains_instance_proposal_approvals_and_fleet_reference() {
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
    let initial = env.deployer().upload_contract_wasm(V1_WASM);
    let fleet_id = BytesN::from_array(&env, &[1; 32]);
    let tag = String::from_str(&env, "ttl-fleet");
    let id = client.create_proposal(
        &approver,
        &ProposalKind::CreateFleet(CreateFleetProposal {
            fleet_id: fleet_id.clone(),
            tag: tag.clone(),
            initial_wasm_hash: initial,
            manifest_hash: BytesN::from_array(&env, &[3; 32]),
        }),
    );
    client.approve(&id, &approver);
    env.as_contract(&controller, || {
        assert!(env.storage().instance().get_ttl() >= INSTANCE_TTL_TARGET);
        assert!(
            env.storage().persistent().get_ttl(&DataKey::Proposal(id))
                >= policy.proposal_lifetime_ledgers + PROPOSAL_TTL_SAFETY_BUFFER
        );
        assert!(
            env.storage().persistent().get_ttl(&DataKey::Approval(id, approver.clone()))
                >= policy.proposal_lifetime_ledgers + PROPOSAL_TTL_SAFETY_BUFFER
        );
    });
    client.maintain_controller();
    client.maintain_proposal(&id);
    env.as_contract(&controller, || {
        assert!(env.storage().instance().get_ttl() >= INSTANCE_TTL_TARGET);
        assert!(
            env.storage().persistent().get_ttl(&DataKey::Proposal(id))
                >= PERSISTENT_RECORD_TTL_TARGET
        );
        assert!(
            env.storage().persistent().get_ttl(&DataKey::Approval(id, approver.clone()))
                >= PERSISTENT_RECORD_TTL_TARGET
        );
    });
    assert!(client.try_maintain_proposal(&999).is_err());
    assert!(client.try_maintain_fleet(&fleet_id).is_err());

    env.ledger().set_sequence_number(env.ledger().sequence() + 3);
    client.execute_proposal(&id);
    client.maintain_fleet(&fleet_id);
    env.as_contract(&controller, || {
        assert!(
            env.storage().persistent().get_ttl(&DataKey::Fleet(fleet_id.clone()))
                >= PERSISTENT_RECORD_TTL_TARGET
        );
        assert!(
            env.storage().persistent().get_ttl(&DataKey::FleetByTag(tag.clone()))
                >= PERSISTENT_RECORD_TTL_TARGET
        );
        assert!(env.executable_refs().get_ttl(&tag) >= PERSISTENT_RECORD_TTL_TARGET);
    });
}
