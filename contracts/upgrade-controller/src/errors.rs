use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    InvalidPolicy = 1,
    NoApprovers = 2,
    TooManyApprovers = 3,
    DuplicateApprover = 4,
    InvalidThreshold = 5,
    InvalidTimelock = 6,
    InvalidProposalLifetime = 7,
    NotApprover = 20,
    FleetExists = 30,
    FleetNotFound = 31,
    TagAlreadyUsed = 32,
    ExecutableRefMissing = 33,
    CurrentWasmMismatch = 34,
    CandidateMatchesCurrent = 35,
    InvalidFleetTag = 36,
    ProposalNotFound = 50,
    ProposalNotActive = 51,
    ProposalExpired = 52,
    ProposalStale = 53,
    AlreadyApproved = 54,
    ApprovalNotFound = 55,
    ThresholdNotMet = 56,
    TimelockNotStarted = 57,
    TimelockNotElapsed = 58,
    CannotCancelAfterThreshold = 59,
    ControllerVersionMismatch = 70,
    InvalidControllerVersion = 71,
    ArithmeticOverflow = 80,
    TtlConfigurationInvalid = 81,
}
