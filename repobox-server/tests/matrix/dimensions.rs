use std::collections::HashSet;

/// Test scenario dimensions for policy matrix testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestLayer {
    Unit,
    Shim,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyArea {
    Ownership,
    AppendOnly,
    Signatures,
    BranchTopology,
    MergeRules,
    Routing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Push,
    CreateBranch,
    DeleteBranch,
    ForcePush,
    Merge,
    RebaseResultPush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepoState {
    EmptyRepo,
    ExistingBranch,
    DivergedHistory,
    MergePresent,
    ProtectedBranch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeShape {
    Create,
    Modify,
    Delete,
    Rename,
    Append,
    Rewrite,
    ModeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActorState {
    Allowed,
    Disallowed,
    Owner,
    NonOwner,
    Signed,
    Unsigned,
    WrongSigner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientPath {
    RawGit,
    Shim,
    OutdatedShim,
    AlternateClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpectedResult {
    Allow,
    Reject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReasonCode {
    OwnershipViolation,
    AppendViolation,
    SignatureRequired,
    NonFfDenied,
    BranchCreateDenied,
    BranchDeleteDenied,
    ForcePushDenied,
    ProtectedBranchDeleteDenied,
    ProtectedBranchForcePushDenied,
    AtomicMultiRefPartialFailure,
    UnauthorizedAccess,
    PolicyViolation,
}

/// Registry of available test dimensions
pub struct DimensionRegistry {
    layers: HashSet<TestLayer>,
    policy_areas: HashSet<PolicyArea>,
    operations: HashSet<Operation>,
    repo_states: HashSet<RepoState>,
    change_shapes: HashSet<ChangeShape>,
    actor_states: HashSet<ActorState>,
    client_paths: HashSet<ClientPath>,
    expected_results: HashSet<ExpectedResult>,
    reason_codes: HashSet<ReasonCode>,
}

impl Default for DimensionRegistry {
    fn default() -> Self {
        Self {
            layers: [TestLayer::Unit, TestLayer::Shim, TestLayer::Server].into_iter().collect(),
            policy_areas: [PolicyArea::Ownership, PolicyArea::AppendOnly, PolicyArea::Signatures, PolicyArea::BranchTopology, PolicyArea::MergeRules, PolicyArea::Routing].into_iter().collect(),
            operations: [Operation::Push, Operation::CreateBranch, Operation::DeleteBranch, Operation::ForcePush, Operation::Merge, Operation::RebaseResultPush].into_iter().collect(),
            repo_states: [RepoState::EmptyRepo, RepoState::ExistingBranch, RepoState::DivergedHistory, RepoState::MergePresent, RepoState::ProtectedBranch].into_iter().collect(),
            change_shapes: [ChangeShape::Create, ChangeShape::Modify, ChangeShape::Delete, ChangeShape::Rename, ChangeShape::Append, ChangeShape::Rewrite, ChangeShape::ModeOnly].into_iter().collect(),
            actor_states: [ActorState::Allowed, ActorState::Disallowed, ActorState::Owner, ActorState::NonOwner, ActorState::Signed, ActorState::Unsigned, ActorState::WrongSigner].into_iter().collect(),
            client_paths: [ClientPath::RawGit, ClientPath::Shim, ClientPath::OutdatedShim, ClientPath::AlternateClient].into_iter().collect(),
            expected_results: [ExpectedResult::Allow, ExpectedResult::Reject].into_iter().collect(),
            reason_codes: [ReasonCode::OwnershipViolation, ReasonCode::AppendViolation, ReasonCode::SignatureRequired, ReasonCode::NonFfDenied, ReasonCode::BranchCreateDenied, ReasonCode::BranchDeleteDenied, ReasonCode::ForcePushDenied, ReasonCode::ProtectedBranchDeleteDenied, ReasonCode::ProtectedBranchForcePushDenied, ReasonCode::AtomicMultiRefPartialFailure, ReasonCode::UnauthorizedAccess, ReasonCode::PolicyViolation].into_iter().collect(),
        }
    }
}

impl DimensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_layers(&self) -> &HashSet<TestLayer> {
        &self.layers
    }

    pub fn get_policy_areas(&self) -> &HashSet<PolicyArea> {
        &self.policy_areas
    }

    pub fn get_operations(&self) -> &HashSet<Operation> {
        &self.operations
    }

    pub fn get_repo_states(&self) -> &HashSet<RepoState> {
        &self.repo_states
    }

    pub fn get_change_shapes(&self) -> &HashSet<ChangeShape> {
        &self.change_shapes
    }

    pub fn get_actor_states(&self) -> &HashSet<ActorState> {
        &self.actor_states
    }

    pub fn get_client_paths(&self) -> &HashSet<ClientPath> {
        &self.client_paths
    }

    pub fn get_expected_results(&self) -> &HashSet<ExpectedResult> {
        &self.expected_results
    }

    pub fn get_reason_codes(&self) -> &HashSet<ReasonCode> {
        &self.reason_codes
    }
}