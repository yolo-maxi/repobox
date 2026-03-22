# Virtuals Integration — Bug Report to Payment Flow

**Specification ID**: `SPEC-VIRTUALS-001`  
**Priority**: P1  
**Tags**: feature, x402, virtuals, docs, hackathon  
**Author**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)  
**Date**: 2026-03-22  

## Summary

Implement and document the complete end-to-end "Virtuals integration" flow where AI agents autonomously: file bug reports, create branches, push fixes, have PRs reviewed, get merged, and receive x402 micropayments. This showcases repo.box as the crypto-native infrastructure for autonomous agent work.

## Context and Motivation

**Problem:**
- AI agents need a complete workflow for autonomous software maintenance
- Traditional git hosting lacks crypto-native payment rails for agent work
- No clear demonstration of agent → code → payment pipeline exists
- Hackathon judges need a compelling "wow factor" use case

**Solution:**
- Full Virtuals platform integration with repo.box's EVM-signed git
- Autonomous agent workflow: bug report → branch → fix → PR → merge → payment
- Real x402 USDC micropayments triggered by successful merges
- Comprehensive documentation and live demo for hackathon presentation

**Value Proposition:**
- Agents can earn crypto for fixing bugs
- Maintainers get automated fixes with quality gates
- Transparent, on-chain record of all contributions
- Demonstrates repo.box's utility beyond just git hosting

## User Story

**As a Virtuals AI agent,**  
I want to be able to discover bug bounties in repositories, autonomously implement fixes, and receive cryptocurrency payments upon successful merge,  
**So that** I can participate in a sustainable economy where AI agents contribute to open source software and get compensated.

**As a repository maintainer,**  
I want to set up automated bug bounties that AI agents can claim and fulfill,  
**So that** I can get high-quality code contributions without manual developer recruitment.

## Technical Requirements

### 1. Bug Report Detection and Response

**Scope**: Agents discover and respond to bug reports/issues

**Requirements:**
- **BR-1.1**: Repository `.repobox/config.yml` must support `virtuals` configuration block
- **BR-1.2**: Virtuals config specifies bug bounty amounts in USDC per issue severity
- **BR-1.3**: Agent discovery mechanism for available bug bounties
- **BR-1.4**: Issue template validation (ensure bug reports contain reproduction steps)
- **BR-1.5**: Agent claim mechanism to avoid duplicate work

**Implementation:**
```yaml
# .repobox/config.yml
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"  # USDC
    high: "25.00"
    medium: "10.00" 
    low: "5.00"
  agent_requirements:
    min_reputation: 0.8  # Future: agent reputation system
    required_tests: true
```

### 2. Autonomous Branch Creation and Development

**Scope**: Agents create feature branches and implement fixes

**Requirements:**
- **BD-2.1**: Agents must use EVM-signed commits via repo.box credential helper
- **BD-2.2**: Branch naming convention: `agent/{agent-id}/fix-{issue-number}`
- **BD-2.3**: Commit messages must reference the original issue
- **BD-2.4**: All commits must include agent identity in signature metadata
- **BD-2.5**: Branch protection: force-push disabled for agent branches

**Implementation:**
- Extend existing git credential helper to support agent identities
- Server-side validation of commit signatures
- Branch naming enforcement in server hooks

### 3. Pull Request Automation

**Scope**: Agent-generated PRs with quality gates

**Requirements:**
- **PR-3.1**: Auto-generated PR description linking to original issue
- **PR-3.2**: Required status checks: tests, linting, security scans
- **PR-3.3**: Agent self-testing: PR must include test coverage for the fix
- **PR-3.4**: Human review requirement for all agent PRs (no auto-merge)
- **PR-3.5**: PR template for agent submissions with checklist

**Implementation:**
- Server-side PR creation hooks
- Integration with existing test runners
- Web interface improvements for agent PR review

### 4. x402 Payment Integration

**Scope**: Crypto payments triggered by successful merges

**Requirements:**
- **PAY-4.1**: x402 payment triggered automatically on merge to main
- **PAY-4.2**: Payment amount based on issue severity from bug bounty config
- **PAY-4.3**: Payment goes to the EVM address from agent's commit signature
- **PAY-4.4**: Transaction hash recorded in merge commit message
- **PAY-4.5**: Support for payment splitting (multiple agents on one fix)
- **PAY-4.6**: Fallback mechanisms for payment failures

**Implementation:**
```yaml
# Repository x402 config for agent payments
x402:
  agent_payments:
    network: "base"
    token: "USDC"  # 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
    treasury: "0x..."  # Repository treasury address
    gas_sponsor: "0x..."  # Who pays transaction fees
```

### 5. Documentation and Discovery

**Scope**: Agent-readable information and human documentation

**Requirements:**
- **DOC-5.1**: Update `/llms.txt` with Virtuals integration details
- **DOC-5.2**: Agent discovery endpoint: `/.well-known/virtuals.json`
- **DOC-5.3**: Landing page case study with real example
- **DOC-5.4**: Playground demonstration of the full flow
- **DOC-5.5**: OpenAPI specification for agent interactions

**Implementation:**
- Structured data formats for agent consumption
- Human-readable case studies with screenshots
- Interactive playground demos

## Current State Analysis

### Existing Infrastructure

**✅ Available:**
- EVM-signed commits via repobox credential helper
- x402 payment configuration in `.repobox/config.yml`
- Git server with custom verbs and permission system
- Web explorer interface for repository browsing
- Test infrastructure and CI/CD hooks

**❌ Missing:**
- Virtuals-specific configuration schema
- Agent discovery mechanisms
- Automated payment triggering on merge
- Agent-specific branch protection rules
- PR automation and templates

### Code Locations

**Frontend (Next.js):**
- Repository detail page: `/web/src/app/explore/[address]/[name]/page.tsx`
- Configuration UI: TBD (needs new component)

**Backend (Rust):**
- Config parsing: `/repobox-core/src/config.rs` 
- Git operations: `/repobox-core/src/git.rs`
- Server hooks: `/repobox-server/src/hooks/`

**Documentation:**
- llms.txt: `/public/llms.txt`
- Landing page: `/repobox-landing/`

## Technical Implementation Plan

### Phase 1: Configuration Schema (Week 1)

**Tasks:**
1. **VIRT-1.1**: Extend `config.rs` with `VirtualsConfig` struct
2. **VIRT-1.2**: Add validation for bug bounty amounts and agent requirements
3. **VIRT-1.3**: Update config parser to handle virtuals block
4. **VIRT-1.4**: Add configuration UI in web interface
5. **VIRT-1.5**: Write comprehensive config documentation

**Acceptance Criteria:**
- ✅ Repository can specify bug bounty amounts per severity
- ✅ Configuration validation prevents invalid setups
- ✅ Web UI allows easy config editing
- ✅ Documentation includes example configurations

### Phase 2: Agent Discovery and Branch Management (Week 2)

**Tasks:**
1. **VIRT-2.1**: Implement `/.well-known/virtuals.json` endpoint
2. **VIRT-2.2**: Add agent branch naming validation
3. **VIRT-2.3**: Extend commit signature validation for agent identities
4. **VIRT-2.4**: Implement branch protection for agent workflows
5. **VIRT-2.5**: Create agent onboarding documentation

**Acceptance Criteria:**
- ✅ Agents can discover available bounties via standardized endpoint
- ✅ Agent commits are properly signed and validated
- ✅ Branch naming conventions are enforced
- ✅ Force-push protection works for agent branches

### Phase 3: PR Automation and Review (Week 3)

**Tasks:**
1. **VIRT-3.1**: Create PR templates for agent submissions
2. **VIRT-3.2**: Implement automatic PR creation for agent branches
3. **VIRT-3.3**: Add status checks integration
4. **VIRT-3.4**: Create review guidelines for agent PRs
5. **VIRT-3.5**: Implement PR quality scoring

**Acceptance Criteria:**
- ✅ Agent PRs include all required information
- ✅ Status checks run automatically
- ✅ Human reviewers have clear guidelines
- ✅ Quality metrics help prioritize reviews

### Phase 4: Payment Integration (Week 4)

**Tasks:**
1. **VIRT-4.1**: Implement merge hooks for payment triggering
2. **VIRT-4.2**: Create x402 payment integration
3. **VIRT-4.3**: Add payment status tracking
4. **VIRT-4.4**: Implement payment splitting for collaborative fixes
5. **VIRT-4.5**: Create payment failure recovery mechanisms

**Acceptance Criteria:**
- ✅ Payments automatically trigger on successful merge
- ✅ Payment amounts match configured bounty levels
- ✅ Transaction hashes are recorded
- ✅ Failed payments can be retried

### Phase 5: Documentation and Demo (Week 5)

**Tasks:**
1. **VIRT-5.1**: Update `/llms.txt` with complete Virtuals integration
2. **VIRT-5.2**: Create landing page case study
3. **VIRT-5.3**: Build playground demonstration
4. **VIRT-5.4**: Write agent integration guides
5. **VIRT-5.5**: Record demo videos for hackathon

**Acceptance Criteria:**
- ✅ Complete agent-readable documentation
- ✅ Compelling case study with real examples
- ✅ Working playground demonstration
- ✅ Professional demo materials ready

## Testing Strategy

### Unit Tests

**Configuration:**
- Valid/invalid virtuals config parsing
- Bug bounty amount validation
- Agent requirement validation

**Payment Logic:**
- x402 payment calculation
- Payment splitting algorithms
- Failure recovery mechanisms

### Integration Tests

**End-to-End Workflows:**
1. **Test Case 1: Complete Bug Fix Flow**
   - Setup: Repository with virtuals config and open bug
   - Action: Agent claims bug, creates branch, implements fix, creates PR
   - Verification: PR created, tests pass, human review, merge triggers payment
   - Expected: USDC payment sent to agent's address

2. **Test Case 2: Multiple Agents on One Issue**
   - Setup: Two agents claim same bug
   - Action: First agent creates branch and PR
   - Verification: Second agent cannot duplicate work
   - Expected: Only first agent eligible for payment

3. **Test Case 3: Payment Failure Recovery**
   - Setup: Repository with insufficient treasury balance
   - Action: Agent completes fix and gets merged
   - Verification: Payment fails, retry mechanism activates
   - Expected: Payment completed when treasury refilled

### Security Tests

**Agent Authentication:**
- Invalid EVM signatures rejected
- Agent identity spoofing prevented
- Branch protection bypassing attempts blocked

**Payment Security:**
- Treasury balance validation
- Double-payment prevention
- Gas estimation and limits

## Acceptance Criteria

### Functional Requirements

- ✅ **F-1**: Agents can discover bug bounties via machine-readable endpoint
- ✅ **F-2**: Agents can create branches with EVM-signed commits
- ✅ **F-3**: Agent PRs include all required metadata and tests
- ✅ **F-4**: Human reviewers can efficiently review agent contributions
- ✅ **F-5**: Successful merges trigger automatic USDC payments
- ✅ **F-6**: All transactions have on-chain records

### Performance Requirements

- ✅ **P-1**: Agent discovery responds within 200ms
- ✅ **P-2**: Payment processing completes within 60 seconds
- ✅ **P-3**: PR creation doesn't impact git performance
- ✅ **P-4**: Configuration updates apply immediately

### Security Requirements

- ✅ **S-1**: All agent commits require valid EVM signatures
- ✅ **S-2**: Payment amounts cannot exceed configured limits
- ✅ **S-3**: Treasury addresses cannot be modified by agents
- ✅ **S-4**: Branch protection prevents unauthorized force-pushes

### Documentation Requirements

- ✅ **D-1**: Complete agent integration guide available
- ✅ **D-2**: Landing page includes compelling case study
- ✅ **D-3**: Playground demonstrates full workflow
- ✅ **D-4**: OpenAPI spec covers all agent endpoints

## Implementation Details

### Agent Discovery Format

**Endpoint:** `GET /.well-known/virtuals.json`

```json
{
  "version": "1.0",
  "repository": {
    "name": "example-repo",
    "address": "0x1234...",
    "virtuals_enabled": true
  },
  "bug_bounties": {
    "active_issues": [
      {
        "id": "42",
        "title": "Memory leak in async handler",
        "severity": "high", 
        "bounty_usdc": "25.00",
        "claimed": false,
        "created_at": "2026-03-20T10:00:00Z",
        "labels": ["bug", "async", "memory"],
        "description": "...",
        "reproduction_steps": "..."
      }
    ]
  },
  "requirements": {
    "min_reputation": 0.8,
    "required_tests": true,
    "review_required": true
  },
  "payment": {
    "network": "base",
    "token": "USDC",
    "treasury": "0x5678..."
  }
}
```

### Commit Message Format

```
Fix memory leak in async handler (#42)

- Added proper cleanup in AsyncHandler::drop()
- Updated tests to verify memory usage
- Resolves issue #42

Agent: 0xAgent123...
Bounty: 25.00 USDC
Virtuals-ID: virt_fix_42_20260322
```

### Payment Transaction Metadata

```
{
  "repository": "0x1234.../example-repo",
  "issue_id": "42",
  "pr_id": "123", 
  "agent_address": "0xAgent123...",
  "bounty_amount": "25.00",
  "currency": "USDC",
  "network": "base",
  "merge_commit": "abc123...",
  "timestamp": "2026-03-22T15:30:00Z"
}
```

## Documentation Changes

### Landing Page Updates

**New Section: "AI Agent Economy"**
- Hero case study of Virtuals integration
- Step-by-step flow visualization
- Real payment examples with etherscan links
- Agent testimonials (if available)

**Updated Value Props:**
- "Crypto-native payments for AI contributions"
- "Autonomous agent workforce for your repos"
- "Quality-gated bug bounties with instant payments"

### llms.txt Updates

```
## Virtuals Integration

repo.box supports AI agents through the Virtuals platform:

1. **Discovery**: GET /.well-known/virtuals.json for available bounties
2. **Claiming**: Create branch: agent/{agent-id}/fix-{issue-number}
3. **Development**: Use EVM-signed commits via credential helper
4. **Submission**: Auto-generated PR with required metadata
5. **Payment**: USDC sent to agent address on successful merge

Bug bounty configuration in .repobox/config.yml:
```

### Playground Examples

**New Demo: "Agent Bug Fix Flow"**
1. Mock repository with configured bounties
2. Simulated agent discovery and claiming
3. Interactive commit signing demonstration
4. Payment visualization with blockchain explorer

## Risks and Mitigations

### Technical Risks

**Risk**: Payment failures due to network congestion  
**Mitigation**: Retry mechanism with exponential backoff, treasury monitoring

**Risk**: Agent commit signature spoofing  
**Mitigation**: Server-side signature validation, agent reputation tracking

**Risk**: Double-payment on concurrent merges  
**Mitigation**: Database transactions, payment status locking

### Business Risks

**Risk**: Agents gaming the system for easy bounties  
**Mitigation**: Human review requirement, quality scoring, reputation system

**Risk**: High gas costs making small bounties uneconomical  
**Mitigation**: Payment batching, gas sponsorship options

**Risk**: Virtuals platform changes breaking integration  
**Mitigation**: Versioned API, fallback to direct agent interaction

## Success Metrics

### Engagement Metrics
- Number of repositories enabling Virtuals integration
- Number of active agents participating
- Bug fix completion rate and time-to-resolution
- Payment volume and frequency

### Quality Metrics  
- Human reviewer satisfaction scores
- Bug fix acceptance rate
- Code quality metrics (test coverage, linting)
- Re-work rate (fixes requiring additional changes)

### Technical Metrics
- API response times for agent discovery
- Payment success rate and processing time
- Git operation performance impact
- Error rates and failure recovery

## Future Enhancements

### Phase 2 Features (Post-Hackathon)
- Agent reputation and ranking system
- Bounty marketplace with bidding
- Multi-repository project coordination
- Advanced payment splitting (reviewers, maintainers)
- Integration with other agent platforms beyond Virtuals

### Ecosystem Growth
- Agent development tools and SDKs
- Repository analytics for maintainer insights
- Cross-platform agent identity standards
- Governance tokens for platform decisions

## Conclusion

The Virtuals integration positions repo.box as the premier platform for AI agent collaboration in open source software. By combining EVM-signed git operations with crypto-native payments, we create a sustainable economy where AI agents can contribute meaningfully to software projects while receiving fair compensation.

This specification provides a complete roadmap from initial configuration to production deployment, ensuring the hackathon demonstration showcases not just a prototype but a production-ready system that can scale with the growing AI agent ecosystem.

**Next Steps:**
1. Begin Phase 1 implementation (configuration schema)
2. Set up test repository for development and demo
3. Coordinate with Virtuals team on agent integration requirements
4. Prepare demo materials and case studies for hackathon presentation

---

*This specification will be updated as implementation progresses and requirements are refined.*# Updated Sun Mar 22 12:19:42 AM UTC 2026
