# repo.box Spec: Bug Bounties

## Overview

Bug bounties integrate repo.box's append-only git permissions with on-chain ERC-8183 escrow contracts to create trustless security research programs. Researchers submit immutable bug reports via git, while payments are managed automatically through smart contracts.

## Core Principles

- **Immutable Reports**: Bug reports are append-only JSONL entries that cannot be modified or deleted
- **Trustless Payments**: USDC payments held in ERC-8183 escrow contracts, automatically released upon approval
- **Transparent Process**: All bounty jobs, submissions, and evaluations are recorded on-chain
- **Permissionless Participation**: Anyone can submit reports to public bounty repos
- **Objective Evaluation**: Independent evaluators assess reports without bias

## System Architecture

### On-Chain Component: BugBountyEscrow Contract

The `BugBountyEscrow` contract implements ERC-8183 "Agentic Commerce" standard for trustless job escrow:

**Contract Address**: TBD (deploy to mainnet)
**Token**: USDC (0xA0b86a33E6417a1C2E59D1A4fEce5b15e91caE9b on mainnet)

**Job Lifecycle**:
1. `Open` → Client creates job with description and evaluator
2. `Funded` → Client deposits USDC budget into escrow
3. `Submitted` → Provider submits bug report deliverable
4. `Completed` → Evaluator approves and payment releases
5. `Rejected`/`Expired` → Client can claim refund

### Off-Chain Component: Append-Only Git Repos

Bug bounty repositories use repo.box permissions for immutable report collection:

```yaml
# .repobox/config.yml
permissions:
  default: deny
  rules:
    - '* append ./**/*.jsonl'  # Anyone can append bug reports
    - '* read >*'              # Anyone can read everything
```

## Full Bug Bounty Flow

### 1. Bounty Creation

Repository owner creates a bounty job on-chain:

```typescript
// Create job
const jobId = await bugBountyContract.createJob(
  provider,     // 0x0 for open bounty
  evaluator,    // trusted security expert
  expiredAt,    // timestamp
  description,  // "Find SQL injection vulnerabilities"
  repoName,     // "mycompany/backend-api"
  hook          // optional callback contract
);

// Set budget and fund
await bugBountyContract.setBudget(jobId, amount);
await usdcContract.approve(bugBountyContract.address, amount);
await bugBountyContract.fund(jobId, amount);
```

### 2. Bug Discovery & Submission

Security researcher finds a vulnerability and submits a report:

```bash
# Clone bounty repo
git clone https://repo.box/mycompany/bug-bounty-backend-api
cd bug-bounty-backend-api

# Append bug report to JSONL file
echo '{
  "reporter": "0x742d35Cc123C6F89dE3b8EfDf4e5e7c3C1234567",
  "severity": "high",
  "title": "SQL injection in /api/auth/login",
  "file": "api/auth.js",
  "line": 23,
  "description": "Username parameter vulnerable to SQLi...",
  "timestamp": "2026-03-21T16:09:23.456Z"
}' >> reports/2026-03-21.jsonl

# Commit and push
git add reports/2026-03-21.jsonl
git commit -m "Bug report: SQL injection in auth endpoint"
git push
```

### 3. On-Chain Submission

Researcher links their git submission to the on-chain job:

```typescript
// Reference to the specific JSONL line
const deliverable = "reports/2026-03-21.jsonl:line:42";

// Submit to contract
await bugBountyContract.submit(jobId, deliverable);
```

### 4. Evaluation & Payment

Evaluator reviews the report and makes a decision:

```typescript
// Approve valid report → payment to researcher
await bugBountyContract.complete(jobId, "Valid SQLi vulnerability");

// Or reject invalid/duplicate report
await bugBountyContract.reject(jobId, "Already reported");
```

## API Integration

### Server Endpoints

The repo.box server provides API endpoints for bounty discovery and management:

#### GET /bounties/{repoName}

Returns active bug bounties for a repository:

```json
{
  "repo": "mycompany/backend-api",
  "bounties": [
    {
      "jobId": 42,
      "client": "0x1234...",
      "evaluator": "0x5678...",
      "budget": "1000000000", // 1000 USDC (6 decimals)
      "expiredAt": 1735689600,
      "status": "Funded",
      "description": "Find SQL injection vulnerabilities",
      "repoName": "mycompany/backend-api"
    }
  ]
}
```

#### GET /bounties/{repoName}/reports

Returns submitted bug reports from the repository:

```json
{
  "reports": [
    {
      "hash": "sha256:abc123...",
      "reporter": "0x742d35...",
      "severity": "high",
      "title": "SQL injection in /api/auth/login",
      "timestamp": "2026-03-21T16:09:23.456Z",
      "file": "reports/2026-03-21.jsonl",
      "line": 42
    }
  ]
}
```

### Git Server Integration

The git server validates permissions and logs bounty-related activities:

1. **Report Validation**: Ensures appended JSONL is valid JSON with required fields
2. **Immutability**: Prevents modification or deletion of existing report lines
3. **Event Logging**: Records report submissions for on-chain correlation

## Repository Template

Bug bounty repos are created from the template at `templates/bug-bounty/`:

```
templates/bug-bounty/
├── .repobox/
│   └── config.yml        # Append-only permissions
├── reports/
│   └── .gitkeep         # Directory for JSONL reports
└── README.md            # Instructions and format spec
```

### Repository Setup

```bash
# Create new bug bounty repo from template
repobox create mycompany/bug-bounty-api --template=bug-bounty

# Deploy and configure on-chain escrow
./deploy-bounty.sh mycompany/bug-bounty-api
```

## JSONL Report Format

Bug reports must follow this exact JSON schema:

```typescript
interface BugReport {
  reporter: string;     // Ethereum address (checksummed)
  severity: "critical" | "high" | "medium" | "low";
  title: string;        // Brief description
  file: string;         // Path to vulnerable file
  line: number;         // Line number (optional, use 0 if unknown)
  description: string;  // Detailed explanation with reproduction steps
  timestamp: string;    // ISO 8601 timestamp
}
```

**Example**:
```json
{"reporter":"0x742d35Cc123C6F89dE3b8EfDf4e5e7c3C1234567","severity":"critical","title":"Remote code execution via deserialization","file":"api/upload.js","line":156,"description":"The file upload endpoint deserializes user-provided data without validation, allowing arbitrary code execution. Proof of concept: upload a pickle file containing os.system('rm -rf /') payload.","timestamp":"2026-03-21T16:09:23.456Z"}
```

## Security Considerations

### Report Validation

- **Address Validation**: Reporter addresses must be valid, checksummed Ethereum addresses
- **Schema Compliance**: Reports must exactly match the JSON schema
- **Content Sanitization**: Descriptions are treated as plain text, no HTML/markdown rendering
- **Size Limits**: Reports limited to 10KB to prevent abuse

### Contract Security

- **Reentrancy Protection**: Uses checks-effects-interactions pattern
- **Access Control**: Strict role-based permissions (client/provider/evaluator)
- **Expiration Handling**: Automatic refunds for expired bounties
- **Token Safety**: Proper ERC20 handling with transfer validation

### Git Security

- **Append-Only Enforcement**: Prevents tampering with existing reports
- **Signature Verification**: All commits must be properly signed
- **Immutable History**: No force-push or history rewriting allowed

## Economics

### Payment Model

- **Client Deposits**: USDC locked in escrow when bounty is funded
- **Automatic Release**: Payment sent directly to reporter on approval
- **Gas Optimization**: Minimal on-chain operations to reduce costs
- **Refund Protection**: Clients can reclaim funds from expired/rejected bounties

### Fee Structure

- **Platform Fee**: 2% of bounty amount goes to repo.box treasury
- **Evaluator Fee**: Optional fee paid to independent evaluators
- **Gas Costs**: Borne by respective parties (client for funding, provider for submission)

## Deployment Guide

### Contract Deployment

```bash
# Deploy to mainnet
forge create BugBountyEscrow --constructor-args 0xA0b86a33E6417a1C2E59D1A4fEce5b15e91caE9b

# Verify on Etherscan
forge verify-contract --chain mainnet 0x... BugBountyEscrow
```

### Server Configuration

```yaml
# config.yml
bug_bounties:
  enabled: true
  contract_address: "0x..."
  usdc_address: "0xA0b86a33E6417a1C2E59D1A4fEce5b15e91caE9b"
  default_evaluator: "0x..." # Optional default evaluator
```

## Future Enhancements

### Planned Features

- **Multi-Token Support**: Accept payments in ETH, DAI, and other tokens
- **Reputation System**: Track researcher success rates and evaluator fairness
- **Automated Screening**: AI-powered duplicate detection and initial triage
- **Batch Operations**: Submit multiple reports in a single transaction
- **Integration APIs**: Webhook notifications and external service integrations

### Research Areas

- **Zero-Knowledge Proofs**: Private bounty submissions for sensitive vulnerabilities
- **Cross-Chain Support**: Deploy on L2s for lower gas costs
- **Oracle Integration**: External verification services for complex reports
- **Legal Framework**: Standardized terms and responsible disclosure policies

---

**Note**: This specification assumes ERC-8183 standard ratification and mainnet deployment of the BugBountyEscrow contract. Implementation may require adjustments based on final standard and security audits.