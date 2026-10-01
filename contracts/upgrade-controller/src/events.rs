use soroban_sdk::{contractevent, Address};

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
