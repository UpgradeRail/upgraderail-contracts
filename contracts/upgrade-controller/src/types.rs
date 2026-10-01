use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernancePolicy {
    pub approvers: Vec<Address>,
    pub threshold: u32,
    pub timelock_ledgers: u32,
    pub proposal_lifetime_ledgers: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fleet {
    pub id: BytesN<32>,
    pub tag: String,
    pub created_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateFleetProposal {
    pub fleet_id: BytesN<32>,
    pub tag: String,
    pub initial_wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeFleetProposal {
    pub fleet_id: BytesN<32>,
    pub expected_wasm_hash: BytesN<32>,
    pub new_wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePolicyProposal {
    pub policy: GovernancePolicy,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeControllerProposal {
    pub expected_controller_version: u32,
    pub new_controller_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalKind {
    CreateFleet(CreateFleetProposal),
    UpgradeFleet(UpgradeFleetProposal),
    UpdatePolicy(UpdatePolicyProposal),
    UpgradeController(UpgradeControllerProposal),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StoredProposalStatus {
    Active,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub kind: ProposalKind,
    pub governance_epoch: u64,
    pub created_ledger: u32,
    pub expires_ledger: u32,
    pub approval_count: u32,
    pub approved_ledger: Option<u32>,
    pub execute_after_ledger: Option<u32>,
    pub status: StoredProposalStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalState {
    AwaitingApprovals,
    Timelocked,
    Ready,
    Expired,
    Stale,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Policy,
    GovernanceEpoch,
    ProposalNonce,
    ControllerVersion,
    Fleet(BytesN<32>),
    FleetByTag(String),
    Proposal(u64),
    Approval(u64, Address),
}
