# Bug Bounty Repository

This repository is set up for collecting bug reports through repo.box's append-only system integrated with the ERC-8183 bug bounty escrow contract.

## How It Works

1. **Bug Bounty Creation**: Repository owners create bug bounty jobs on-chain using the BugBountyEscrow contract
2. **Funding**: Clients fund bounties with USDC, which is held in escrow
3. **Report Submission**: Security researchers find bugs and submit reports by appending to JSONL files in this repo
4. **Evaluation**: Designated evaluators review reports and approve/reject them on-chain
5. **Payment**: Approved reports trigger automatic USDC payments to researchers

## Submitting Bug Reports

### Report Format

Submit bug reports by appending a single line to a JSONL file in the `reports/` directory. Each line should be valid JSON with the following structure:

```json
{
  "reporter": "0x1234567890123456789012345678901234567890",
  "severity": "critical|high|medium|low",
  "title": "Brief description of the vulnerability",
  "file": "path/to/vulnerable/file.js",
  "line": 42,
  "description": "Detailed explanation of the vulnerability, how to reproduce it, and potential impact",
  "timestamp": "2026-03-21T16:09:23.456Z"
}
```

### Example Report

```json
{"reporter":"0x742d35Cc123C6F89dE3b8EfDf4e5e7c3C1234567","severity":"high","title":"SQL injection in user login endpoint","file":"api/auth.js","line":23,"description":"The user login endpoint at /api/auth/login is vulnerable to SQL injection through the username parameter. An attacker can bypass authentication by injecting SQL code. To reproduce: send POST request with username=\"admin' OR '1'='1' --\" and any password. This allows complete database access.","timestamp":"2026-03-21T16:09:23.456Z"}
```

### Submission Process

1. **Find a Bug**: Identify a security vulnerability in the target repository
2. **Create Report**: Format your findings according to the JSON schema above
3. **Submit via Git**:
   ```bash
   # Clone this repo
   git clone [repository-url]
   cd [repository-name]

   # Append your report (replace YYYY-MM-DD with current date)
   echo '{"reporter":"0x...","severity":"high",...}' >> reports/YYYY-MM-DD.jsonl

   # Commit and push
   git add reports/YYYY-MM-DD.jsonl
   git commit -m "Bug report: [brief description]"
   git push
   ```

### Report Requirements

- **Reproducible**: Include clear steps to reproduce the vulnerability
- **Detailed**: Provide sufficient technical detail for evaluation
- **Original**: Submit only original findings, not previously reported issues
- **In-scope**: Ensure the vulnerability is within the defined scope of the bounty
- **Accurate**: Use correct wallet address as reporter to receive payment

## Bug Bounty Escrow Contract

The escrow system is powered by the ERC-8183 compliant `BugBountyEscrow` smart contract. Key features:

- **Trustless Payments**: USDC payments are held in escrow and released automatically upon approval
- **Transparent Process**: All bounty jobs and their status are visible on-chain
- **Fair Evaluation**: Independent evaluators assess reports objectively
- **Expiration Protection**: Clients can claim refunds if bounties expire without valid submissions

### Contract Functions

- `createJob()`: Create a new bug bounty with description and budget
- `fund()`: Lock USDC in escrow for the bounty
- `submit()`: Submit a bug report reference (JSONL line hash)
- `complete()`: Approve a report and release payment
- `reject()`: Reject a report with reason
- `claimRefund()`: Reclaim funds from expired bounties

## Severity Levels

- **Critical**: Immediate threat to user funds or system integrity
- **High**: Significant security impact or potential for exploitation
- **Medium**: Moderate security concern with limited impact
- **Low**: Minor security issue or informational finding

## Repository Permissions

This repository uses repo.box permissions that allow:
- ✅ Anyone can **append** to JSONL files (add bug reports)
- ✅ Anyone can **read** all content (view reports and documentation)
- ❌ **Modifications** or **deletions** are restricted to repository owners

This ensures immutable reporting while preventing tampering with existing reports.

## Getting Help

- **Contract Documentation**: See the BugBountyEscrow.sol contract for technical details
- **repo.box Docs**: Learn more about the permission system at [repo.box documentation]
- **Support**: Contact repository owners for bounty-specific questions

---

**⚠️ Legal Notice**: Bug hunting should only be performed on systems you own or have explicit permission to test. This bounty program operates under responsible disclosure principles.