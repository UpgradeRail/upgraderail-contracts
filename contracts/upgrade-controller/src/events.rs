use soroban_sdk::{contractevent, Address, BytesN, String};

#[contractevent]
pub struct ProposalCreated {
    #[topic]
    pub proposal_id: u64,
    pub proposer: Address,
    pub governance_epoch: u64,
    pub expires_ledger: u32,
}

#[contractevent]
pub struct ProposalApproved {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub approver: Address,
    pub approval_count: u32,
}

#[contractevent]
pub struct ThresholdReached {
    #[topic]
    pub proposal_id: u64,
    pub approved_ledger: u32,
    pub execute_after_ledger: u32,
}

#[contractevent]
pub struct ApprovalRevoked {
    #[topic]
    pub proposal_id: u64,
    #[topic]
    pub approver: Address,
    pub approval_count: u32,
}

#[contractevent]
pub struct ThresholdReset {
    #[topic]
    pub proposal_id: u64,
}

#[contractevent]
pub struct ProposalCancelled {
    #[topic]
    pub proposal_id: u64,
    pub proposer: Address,
}

#[contractevent]
pub struct ProposalExecuted {
    #[topic]
    pub proposal_id: u64,
}

#[contractevent]
pub struct FleetCreated {
    #[topic]
    pub fleet_id: BytesN<32>,
    pub proposal_id: u64,
    pub tag: String,
    pub wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
    pub ledger: u32,
}

#[contractevent]
pub struct FleetUpgraded {
    #[topic]
    pub fleet_id: BytesN<32>,
    pub proposal_id: u64,
    pub old_wasm_hash: BytesN<32>,
    pub new_wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
}

#[contractevent]
pub struct PolicyUpdated {
    #[topic]
    pub proposal_id: u64,
    pub governance_epoch: u64,
}

#[contractevent]
pub struct ControllerUpgraded {
    #[topic]
    pub proposal_id: u64,
    pub new_controller_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub manifest_hash: BytesN<32>,
}
