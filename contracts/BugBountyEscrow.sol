// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./lib/forge-std/src/interfaces/IERC20.sol";

/// @title BugBountyEscrow - ERC-8183 compliant bug bounty escrow contract
/// @notice Manages bug bounty jobs on-chain with USDC payments
/// @dev Implements ERC-8183 standard for agentic commerce
contract BugBountyEscrow {

    enum Status { Open, Funded, Submitted, Completed, Rejected, Expired }

    struct Job {
        address client;      // repo owner who funds the bounty
        address provider;    // bug reporter (set when they submit)
        address evaluator;   // who judges the report
        uint256 budget;      // USDC amount in wei (6 decimals for USDC)
        uint256 expiredAt;   // timestamp when job expires
        Status status;
        string repoName;     // which repo the bounty is for
        string description;  // what to look for
        string deliverable;  // the bug report (JSONL line reference)
        address hook;        // optional hook contract
    }

    // State
    mapping(uint256 => Job) public jobs;
    uint256 public nextJobId = 1;
    IERC20 public immutable token; // USDC token

    // Events (ERC-8183 compliant)
    event JobCreated(uint256 indexed jobId, address indexed client, address indexed evaluator, string repoName);
    event ProviderSet(uint256 indexed jobId, address indexed provider);
    event BudgetSet(uint256 indexed jobId, uint256 budget);
    event JobFunded(uint256 indexed jobId, uint256 amount);
    event JobSubmitted(uint256 indexed jobId, address indexed provider, string deliverable);
    event JobCompleted(uint256 indexed jobId, bytes32 reason);
    event JobRejected(uint256 indexed jobId, bytes32 reason);
    event JobExpired(uint256 indexed jobId);
    event PaymentReleased(uint256 indexed jobId, address indexed provider, uint256 amount);
    event Refunded(uint256 indexed jobId, address indexed client, uint256 amount);

    // Errors
    error UnauthorizedCaller();
    error InvalidJobState();
    error JobIsExpired();
    error JobNotYetExpired();
    error InsufficientFunds();
    error TransferFailed();
    error InvalidParameters();

    modifier onlyClient(uint256 jobId) {
        if (jobs[jobId].client != msg.sender) revert UnauthorizedCaller();
        _;
    }

    modifier onlyProvider(uint256 jobId) {
        if (jobs[jobId].provider != msg.sender) revert UnauthorizedCaller();
        _;
    }

    modifier onlyEvaluator(uint256 jobId) {
        if (jobs[jobId].evaluator != msg.sender) revert UnauthorizedCaller();
        _;
    }

    modifier inStatus(uint256 jobId, Status expectedStatus) {
        if (jobs[jobId].status != expectedStatus) revert InvalidJobState();
        _;
    }

    modifier notExpired(uint256 jobId) {
        if (block.timestamp >= jobs[jobId].expiredAt) revert JobIsExpired();
        _;
    }

    modifier isExpired(uint256 jobId) {
        if (block.timestamp < jobs[jobId].expiredAt) revert JobNotYetExpired();
        _;
    }

    constructor(address _token) {
        if (_token == address(0)) revert InvalidParameters();
        token = IERC20(_token);
    }

    /// @notice Create a new bug bounty job (ERC-8183 compliant)
    /// @param provider Address of the provider (can be zero address to be set later)
    /// @param evaluator Address who will evaluate submissions
    /// @param expiredAt Expiration timestamp
    /// @param description Bug bounty description
    /// @param repoName Repository name for the bounty
    /// @param hook Optional hook contract address
    /// @return jobId The created job ID
    function createJob(
        address provider,
        address evaluator,
        uint256 expiredAt,
        string calldata description,
        string calldata repoName,
        address hook
    ) external returns (uint256 jobId) {
        if (evaluator == address(0)) revert InvalidParameters();
        if (expiredAt <= block.timestamp) revert InvalidParameters();
        if (bytes(description).length == 0) revert InvalidParameters();
        if (bytes(repoName).length == 0) revert InvalidParameters();

        jobId = nextJobId++;

        jobs[jobId] = Job({
            client: msg.sender,
            provider: provider,
            evaluator: evaluator,
            budget: 0,
            expiredAt: expiredAt,
            status: Status.Open,
            repoName: repoName,
            description: description,
            deliverable: "",
            hook: hook
        });

        emit JobCreated(jobId, msg.sender, evaluator, repoName);

        if (provider != address(0)) {
            emit ProviderSet(jobId, provider);
        }

        _callHook(jobId, this.createJob.selector, "");
    }

    /// @notice Set the provider for a job (ERC-8183 compliant)
    /// @param jobId Job identifier
    /// @param provider Provider address
    function setProvider(uint256 jobId, address provider)
        external
        onlyClient(jobId)
        inStatus(jobId, Status.Open)
        notExpired(jobId)
    {
        if (provider == address(0)) revert InvalidParameters();

        jobs[jobId].provider = provider;

        emit ProviderSet(jobId, provider);
        _callHook(jobId, this.setProvider.selector, abi.encode(provider));
    }

    /// @notice Set the budget for a job (ERC-8183 compliant)
    /// @param jobId Job identifier
    /// @param amount Budget amount in USDC wei
    function setBudget(uint256 jobId, uint256 amount)
        external
        onlyClient(jobId)
        inStatus(jobId, Status.Open)
        notExpired(jobId)
    {
        if (amount == 0) revert InvalidParameters();

        jobs[jobId].budget = amount;

        emit BudgetSet(jobId, amount);
        _callHook(jobId, this.setBudget.selector, abi.encode(amount));
    }

    /// @notice Fund a job with USDC (ERC-8183 compliant)
    /// @param jobId Job identifier
    /// @param expectedBudget Expected budget amount for validation
    function fund(uint256 jobId, uint256 expectedBudget)
        external
        onlyClient(jobId)
        inStatus(jobId, Status.Open)
        notExpired(jobId)
    {
        Job storage job = jobs[jobId];
        if (job.budget == 0) revert InvalidParameters();
        if (job.budget != expectedBudget) revert InvalidParameters();
        if (job.provider == address(0)) revert InvalidParameters();

        // Transfer USDC from client to this contract
        if (!token.transferFrom(msg.sender, address(this), job.budget)) {
            revert TransferFailed();
        }

        job.status = Status.Funded;

        emit JobFunded(jobId, job.budget);
        _callHook(jobId, this.fund.selector, abi.encode(expectedBudget));
    }

    /// @notice Submit bug report deliverable (ERC-8183 compliant)
    /// @param jobId Job identifier
    /// @param deliverable Reference to bug report (e.g., JSONL line hash or repo path)
    function submit(uint256 jobId, string calldata deliverable)
        external
        onlyProvider(jobId)
        inStatus(jobId, Status.Funded)
        notExpired(jobId)
    {
        if (bytes(deliverable).length == 0) revert InvalidParameters();

        jobs[jobId].deliverable = deliverable;
        jobs[jobId].status = Status.Submitted;

        emit JobSubmitted(jobId, msg.sender, deliverable);
        _callHook(jobId, this.submit.selector, abi.encode(deliverable));
    }

    /// @notice Complete a job and release payment (ERC-8183 compliant)
    /// @param jobId Job identifier
    /// @param reason Reason for completion
    function complete(uint256 jobId, bytes32 reason)
        external
        onlyEvaluator(jobId)
        inStatus(jobId, Status.Submitted)
    {
        Job storage job = jobs[jobId];
        job.status = Status.Completed;

        // Transfer payment to provider
        if (!token.transfer(job.provider, job.budget)) {
            revert TransferFailed();
        }

        emit JobCompleted(jobId, reason);
        emit PaymentReleased(jobId, job.provider, job.budget);
        _callHook(jobId, this.complete.selector, abi.encode(reason));
    }

    /// @notice Reject a job (ERC-8183 compliant)
    /// @dev Can reject from Funded (before submission) or Submitted (after submission)
    /// @param jobId Job identifier
    /// @param reason Reason for rejection
    function reject(uint256 jobId, bytes32 reason)
        external
        onlyEvaluator(jobId)
    {
        Job storage job = jobs[jobId];
        if (job.status != Status.Funded && job.status != Status.Submitted) {
            revert InvalidJobState();
        }

        job.status = Status.Rejected;

        // Refund client
        if (!token.transfer(job.client, job.budget)) {
            revert TransferFailed();
        }

        emit JobRejected(jobId, reason);
        emit Refunded(jobId, job.client, job.budget);
        _callHook(jobId, this.reject.selector, abi.encode(reason));
    }

    /// @notice Claim refund for expired job (ERC-8183 compliant)
    /// @dev Anyone can trigger refund after expiry for Funded or Submitted jobs
    /// @param jobId Job identifier
    function claimRefund(uint256 jobId)
        external
        isExpired(jobId)
    {
        Job storage job = jobs[jobId];
        if (job.status != Status.Funded && job.status != Status.Submitted) {
            revert InvalidJobState();
        }

        job.status = Status.Expired;

        // Refund client
        if (!token.transfer(job.client, job.budget)) {
            revert TransferFailed();
        }

        emit JobExpired(jobId);
        emit Refunded(jobId, job.client, job.budget);
        _callHook(jobId, this.claimRefund.selector, "");
    }

    /// @notice Cancel a job before funding (client only, Open → Rejected)
    /// @param jobId Job identifier
    function cancel(uint256 jobId)
        external
        onlyClient(jobId)
        inStatus(jobId, Status.Open)
    {
        jobs[jobId].status = Status.Rejected;

        emit JobRejected(jobId, bytes32("cancelled_by_client"));
        _callHook(jobId, this.cancel.selector, "");
    }

    /// @notice Get job details
    /// @param jobId Job identifier
    /// @return job Job struct
    function getJob(uint256 jobId) external view returns (Job memory job) {
        return jobs[jobId];
    }

    /// @notice Check if job exists
    /// @param jobId Job identifier
    /// @return exists Whether job exists
    function jobExists(uint256 jobId) external view returns (bool exists) {
        return jobs[jobId].client != address(0);
    }

    /// @dev Internal function to call hook if present
    /// @param jobId Job identifier
    /// @param selector Function selector
    /// @param data Additional data
    function _callHook(uint256 jobId, bytes4 selector, bytes memory data) internal {
        address hook = jobs[jobId].hook;
        if (hook != address(0)) {
            // Call beforeAction
            try IACPHook(hook).beforeAction(jobId, selector, data) {} catch {}
            // Note: afterAction would be called after the main logic in a full implementation
        }
    }
}

/// @title Hook interface for ERC-8183 compliance
interface IACPHook {
    function beforeAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
    function afterAction(uint256 jobId, bytes4 selector, bytes calldata data) external;
}