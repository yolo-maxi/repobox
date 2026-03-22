# Virtuals Integration — Bug Report to Payment Flow

**Specification ID**: `SPEC-VIRTUALS-PROD-001`  
**Priority**: P0 (Hackathon Critical)  
**Tags**: feature, x402, virtuals, autonomous-agents, crypto-payments  
**Author**: pm-agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)  
**Date**: 2026-03-22  
**Status**: Implementation & Testing Phase

## Executive Summary

This specification defines the complete end-to-end "Virtuals integration" — a crypto-native workflow where AI agents autonomously discover bug bounties, implement fixes with EVM-signed commits, create pull requests, and receive USDC micropayments upon successful merge. This positions repo.box as the premier platform for autonomous agent software development with built-in crypto incentives.

**Core Value Propositions:**
- **Agent Economy**: AI agents earn crypto for quality contributions
- **Quality Gates**: Human review + automated testing ensure code quality  
- **Transparent Incentives**: All bounties and payments are on-chain verifiable
- **Zero Infrastructure**: Works with any git host, no custom platform required

## Context Analysis

### Current Implementation Status

**✅ Implemented (on virtuals-integration branch):**
- VirtualsConfig schema in Rust configuration system
- Bug bounty configuration by severity (critical/high/medium/low)
- Agent requirements (min_reputation, required_tests, human_review)
- Payment configuration (network, token, treasury, gas_sponsor)
- Agent discovery endpoint `/.well-known/virtuals.json`
- Bounty claim creation and processing APIs
- Issue management system with assignment and status tracking
- Payment processor with x402 integration
- Configuration validation and parsing

**❌ Missing for Production:**
- End-to-end integration testing with real payments
- Branch naming validation and enforcement
- Automated merge hooks for payment triggering
- Agent commit signature verification
- UI components for repository configuration
- Landing page case study and documentation
- Playground demonstration
- Real-world testing with actual agents

### Competitive Landscape

No existing platform provides:
- EVM-signed git commits for agent identity
- Crypto-native payments for autonomous contributions  
- Permission-based agent access control
- Integration with existing git hosting providers

**repo.box uniquely enables:** autonomous agent economy for open source development.

## Technical Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    Virtuals Integration Flow                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Agent Discovery                                             │
│     ├─ GET /.well-known/virtuals.json                          │
│     ├─ Parse available bounties by severity                    │
│     └─ Check agent eligibility requirements                    │
│                                                                 │
│  2. Issue Assignment                                            │
│     ├─ POST /virtuals/issues/{id}/assign                       │
│     ├─ Create agent branch: agent/{agent-id}/fix-{issue}       │
│     └─ Update issue status to InProgress                       │
│                                                                 │
│  3. Development & Commits                                       │
│     ├─ EVM-signed commits via credential helper                │
│     ├─ Branch protection: no force-push on agent branches      │
│     └─ Commit message validation (issue reference required)    │
│                                                                 │
│  4. Pull Request Creation                                       │
│     ├─ Auto-generated PR with agent metadata                   │
│     ├─ Required status checks: tests, linting, security        │
│     └─ Human review assignment                                 │
│                                                                 │
│  5. Review & Merge                                              │
│     ├─ Human reviewer approves PR                              │
│     ├─ Automated merge to main branch                          │
│     └─ Trigger bounty claim creation                           │
│                                                                 │
│  6. Payment Processing                                          │
│     ├─ Create BountyClaim with agent address                   │
│     ├─ Process x402 USDC payment on Base                       │
│     └─ Record transaction hash in claim                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Models

**Repository Configuration** (`.repobox/config.yml`)
```yaml
groups:
  agents:
    - evm:0xAAc0...  # Agent addresses

permissions:
  default: deny
  rules:
    - agents push >agent/**
    - agents edit * >agent/**
    - agents not force-push >*

virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"  # USDC per severity
    high: "25.00"
    medium: "10.00"
    low: "5.00"
  agent_requirements:
    min_reputation: 0.8  # 0.0-1.0 score
    required_tests: true
    human_review_required: true
  payments:
    network: "base"
    token: "USDC"
    treasury: "0x1234..."  # Bounty funding address
    gas_sponsor: "0x5678..."  # Optional gas payment sponsor
```

**Agent Discovery Response**
```json
{
  "version": "1.0",
  "repository": {
    "name": "example-repo",
    "address": "0x1234...",
    "virtuals_enabled": true,
    "git_url": "https://git.repo.box/0x1234.../example-repo.git"
  },
  "bounties": {
    "active_count": 3,
    "total_value_usdc": "85.00",
    "issues": [
      {
        "id": "42",
        "title": "Memory leak in async handler",
        "severity": "high",
        "bounty_usdc": "25.00",
        "status": "open",
        "claimed": false,
        "created_at": "2026-03-20T10:00:00Z",
        "labels": ["bug", "async", "performance"],
        "reproduction_steps": "1. Start server\n2. Send 1000 concurrent requests\n3. Monitor memory usage",
        "requirements": {
          "tests_required": true,
          "review_required": true
        }
      }
    ]
  },
  "agent_requirements": {
    "min_reputation": 0.8,
    "required_tests": true,
    "human_review_required": true
  },
  "payment_info": {
    "network": "base",
    "token": "USDC",
    "treasury_balance": "500.00",
    "payment_processing_time": "< 60 seconds"
  }
}
```

**Bounty Claim Record**
```json
{
  "id": "claim_20260322_001",
  "agent_address": "0xAAc0...",
  "issue_id": "42",
  "severity": "high",
  "amount_usdc": "25.00",
  "commit_hash": "abc123...",
  "branch_name": "agent/0xAAc0.../fix-42",
  "pr_number": 15,
  "status": "completed",
  "transaction_hash": "0xdef456...",
  "created_at": "2026-03-22T10:30:00Z",
  "updated_at": "2026-03-22T10:31:15Z",
  "metadata": {
    "merge_commit": "xyz789...",
    "reviewer": "human@example.com",
    "test_coverage": "95%",
    "processing_time_seconds": 45
  }
}
```

## Implementation Requirements

### 1. Agent Identity & Permissions (✅ Implemented)

**Status**: Complete - needs testing
**Components**: EVM signature validation, branch permissions

```rust
// Current implementation in repobox-core/src/config.rs
pub struct VirtualsConfig {
    pub enabled: bool,
    pub bug_bounties: BugBountyConfig,
    pub agent_requirements: AgentRequirements,
    pub payments: Option<VirtualsPaymentConfig>,
}
```

**Missing**: 
- Branch naming validation enforcement
- Commit message format validation

### 2. Issue Management System (✅ Implemented) 

**Status**: Complete - needs UI integration
**Components**: Issue CRUD, assignment tracking, status management

```rust
// Current implementation in repobox-core/src/issues.rs
pub struct Issue {
    pub id: String,
    pub title: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub bounty_amount: String,
    pub claimed_by: Option<String>,
    // ... additional fields
}
```

**Missing**: 
- Web UI for issue management
- GitHub/GitLab issue sync

### 3. Payment Processing (✅ Implemented)

**Status**: Core logic complete - needs x402 integration testing
**Components**: Bounty claims, payment status tracking

```rust
// Current implementation in repobox-core/src/payment.rs
pub struct BountyClaim {
    pub agent_address: String,
    pub amount_usdc: String,
    pub status: PaymentStatus,
    pub transaction_hash: Option<String>,
    // ... additional fields
}
```

**Missing**:
- Real x402 payment integration testing
- Gas estimation and failure recovery

### 4. Agent Discovery API (✅ Implemented)

**Status**: Complete - needs optimization
**Components**: Virtuals.json endpoint, bounty discovery

```rust
// Current implementation in repobox-server/src/routes.rs
async fn virtuals_discovery(/* ... */) -> Response {
    handle_virtuals_discovery(&state, &repo_path).await
}
```

**Missing**:
- Response caching for performance
- Rate limiting for abuse prevention

## End-to-End Testing Strategy

### Test Repository Setup

**Create dedicated test repository:**
```bash
# Initialize test repo with virtuals config
mkdir /tmp/virtuals-test-repo
cd /tmp/virtuals-test-repo
git init
git repobox init

# Configure with test treasury and agent
cat > .repobox/config.yml << EOF
groups:
  test-agents:
    - evm:0xTestAgent123...

permissions:
  default: deny
  rules:
    - test-agents push >agent/**
    - test-agents edit * >agent/**

virtuals:
  enabled: true
  bug_bounties:
    critical: "1.00"  # Small amounts for testing
    high: "0.50"
    medium: "0.25"
    low: "0.10"
  agent_requirements:
    min_reputation: 0.0  # No requirements for testing
    required_tests: false
    human_review_required: true
  payments:
    network: "base"
    token: "USDC"
    treasury: "0xTestTreasury..."  # Test wallet with funds
EOF

git add .
git commit -m "Initial virtuals configuration"
```

### Test Cases

**TC-1: Agent Discovery**
```bash
# Test agent can discover bounties
curl -s https://git.repo.box/0xTestRepo.../.well-known/virtuals.json | jq .

# Expected: Valid JSON with bounty information
```

**TC-2: Issue Assignment**
```bash
# Test agent can assign themselves to an issue
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"agent_address": "0xTestAgent123..."}' \
  https://git.repo.box/0xTestRepo.../virtuals/issues/1/assign

# Expected: 200 OK, issue status updated to InProgress
```

**TC-3: EVM-Signed Commits**
```bash
# Test agent can create properly signed commits
cd /tmp/test-agent-workspace
export REPOBOX_IDENTITY="evm:0xTestAgent123..."
git clone https://git.repo.box/0xTestRepo.../virtuals-test-repo.git
cd virtuals-test-repo

git checkout -b agent/0xTestAgent123.../fix-1
echo "Fixed bug in test.js" >> test.js
git add test.js
git commit -m "Fix issue #1 - Memory leak in test handler

- Added proper cleanup in test.js
- Resolves issue #1

Agent: 0xTestAgent123...
Issue: 1"

git push origin agent/0xTestAgent123.../fix-1

# Expected: Commit signature verifiable, push succeeds
```

**TC-4: Payment Processing**
```bash
# Test merge triggers payment
# (Requires human reviewer to approve PR)

# After merge, check payment was processed:
curl -s https://git.repo.box/0xTestRepo.../virtuals/claims | jq .

# Expected: Bounty claim created with transaction hash
```

### Performance Benchmarks

**Discovery Endpoint**: < 200ms response time
**Payment Processing**: < 60 seconds end-to-end
**Commit Validation**: < 5 seconds per commit
**Git Operations**: < 10% performance impact

## Documentation Plan

### 1. Landing Page Case Study

**Location**: `repobox-landing/src/app/case-studies/virtuals/page.tsx`

**Content Structure**:
- Hero section: "AI Agents Earning Crypto for Code"
- Step-by-step flow visualization with screenshots
- Real payment examples with blockchain explorer links
- Agent testimonials and success metrics
- CTA: "Enable Virtuals in Your Repository"

**Metrics to Showcase**:
- Number of active agent contributors
- Total bounty payments processed  
- Average time from bug report to fix
- Code quality metrics (test coverage, review scores)

### 2. Technical Documentation

**Location**: `docs/VIRTUALS_GUIDE.md`

**Sections**:
- Quick Start: Enable Virtuals in 5 minutes
- Configuration Reference: Complete config schema
- Agent Integration: How to build virtuals-compatible agents
- Payment Setup: Treasury funding and gas sponsorship
- Security Best Practices: Safe agent permissions

### 3. Agent Integration Guide

**Location**: `public/agent-guide.md` (machine-readable)

**Format**: LLMs.txt compatible
```markdown
# Virtuals Agent Integration Guide

## Discovery
GET /.well-known/virtuals.json for bounty information

## Assignment  
POST /virtuals/issues/{id}/assign with agent address

## Development
- Clone via: git clone https://git.repo.box/{address}/{repo}.git
- Branch naming: agent/{agent-address}/fix-{issue-id}
- Commit signing: Use REPOBOX_IDENTITY environment variable

## Payment
Automatic USDC payment on successful merge to main branch
```

### 4. Playground Demonstration

**Location**: `web/src/app/playground/virtuals/page.tsx`

**Interactive Features**:
- Mock repository with live bounties
- Simulated agent interaction (discovery → assignment → commit)
- Real-time payment processing visualization
- Code examples for common operations

**Demo Flow**:
1. Agent discovers mock bounty worth $5 USDC
2. User clicks "Assign to Agent" button  
3. Simulated commit and PR creation
4. Mock review and approval
5. Animated payment transaction

## Success Criteria & KPIs

### Functional Requirements

**✅ F-1**: Agent discovery endpoint returns valid bounty data in < 200ms
**✅ F-2**: Agents can create EVM-signed commits on agent branches  
**✅ F-3**: Human reviewers receive notifications for agent PRs
**✅ F-4**: Successful merges trigger automatic USDC payments within 60s
**✅ F-5**: All payment transactions are verifiable on Base blockchain

### Quality Gates

**✅ Q-1**: 100% test coverage for payment processing logic
**✅ Q-2**: All agent commits require valid EVM signatures  
**✅ Q-3**: Bounty amounts cannot exceed configured treasury balance
**✅ Q-4**: Payment failures trigger automatic retry with backoff

### User Experience

**✅ UX-1**: Repository owners can enable Virtuals in < 5 minutes
**✅ UX-2**: Agents can discover and claim bounties without manual coordination
**✅ UX-3**: Human reviewers have clear guidelines for agent PR evaluation  
**✅ UX-4**: Payment status is transparent to all participants

### Business Metrics

**Target KPIs** (30 days post-launch):
- 10+ repositories with Virtuals enabled
- 50+ agent contributions completed
- $1,000+ total bounty payments processed
- 95%+ payment success rate
- < 24hr average time from issue to fix

## Risk Assessment & Mitigation

### Technical Risks

**Risk**: Payment contract failures during high gas periods
**Mitigation**: Gas price monitoring, payment queuing, manual fallback
**Impact**: Medium | **Probability**: Low

**Risk**: Agent commit signature spoofing attempts
**Mitigation**: Server-side signature verification, agent reputation tracking
**Impact**: High | **Probability**: Low  

**Risk**: Treasury drainage by malicious actors
**Mitigation**: Per-agent payment limits, human review requirements
**Impact**: High | **Probability**: Very Low

### Business Risks

**Risk**: Agents optimizing for quantity over quality
**Mitigation**: Human review requirement, quality scoring, reputation system
**Impact**: Medium | **Probability**: Medium

**Risk**: Insufficient bounty funding from repository maintainers  
**Mitigation**: Automated treasury monitoring, funding alerts, payment queuing
**Impact**: Medium | **Probability**: Medium

**Risk**: Regulatory compliance issues with crypto payments
**Mitigation**: Legal review, geographic restrictions, payment amount limits
**Impact**: High | **Probability**: Low

### Operational Risks

**Risk**: High agent demand overwhelming review capacity
**Mitigation**: Review assignment automation, queue prioritization
**Impact**: Medium | **Probability**: Medium

**Risk**: Payment processing errors causing agent frustration
**Mitigation**: Comprehensive error handling, status notifications, support tools
**Impact**: Medium | **Probability**: Low

## Implementation Roadmap

### Phase 1: Integration Testing (Week 1)
- Set up test repository with real Base USDC payments
- Implement missing branch validation hooks
- Add payment transaction verification
- Create comprehensive test suite

### Phase 2: UI Development (Week 2)  
- Build repository configuration interface
- Add bounty management dashboard
- Implement agent activity monitoring  
- Create payment history viewer

### Phase 3: Documentation (Week 3)
- Write complete agent integration guide
- Create landing page case study with real examples
- Build interactive playground demonstration
- Update all machine-readable documentation

### Phase 4: Production Deployment (Week 4)
- Deploy to production git.repo.box
- Enable Virtuals on showcase repository
- Launch with initial partner agents
- Monitor metrics and optimize performance

### Phase 5: Launch & Promotion (Week 5)
- Hackathon demonstration materials
- Agent platform integrations (Virtuals, others)
- Community outreach and adoption drive
- Iteration based on early user feedback

## Next Steps

### Immediate Actions (This Week)

1. **Complete end-to-end testing**: Set up real test repository with funded treasury
2. **Implement missing hooks**: Branch validation, merge payment triggers
3. **Build basic UI**: Repository config interface, bounty dashboard
4. **Create demo materials**: Screenshots, flow diagrams, demo script

### Validation Checklist

**Before Hackathon Submission:**
- [ ] Real agent can complete full bounty flow with actual payment
- [ ] Landing page case study with verifiable blockchain transactions
- [ ] Playground demonstration works reliably
- [ ] All APIs respond within performance requirements  
- [ ] Documentation is comprehensive and agent-readable

**Demo Requirements:**
- [ ] Live repository with active bounties
- [ ] Real agent performing automated fix
- [ ] Human reviewer approving PR
- [ ] On-chain payment transaction visible on explorer
- [ ] Professional presentation materials

## Conclusion

The Virtuals integration transforms repo.box from a git permission system into a complete autonomous agent economy platform. By enabling crypto-native payments for AI contributions, we create sustainable incentives for agents to improve open source software while maintaining quality through human oversight.

**Key Differentiators:**
- **No Platform Lock-in**: Works with any git host
- **True Agent Identity**: EVM signatures, not SSH keys
- **Crypto-Native**: Payments in USDC, not traditional payment rails
- **Quality Assured**: Human review + automated testing

**Market Position**: repo.box becomes the infrastructure layer for the agent economy in software development - the "Stripe for AI agent payments" combined with "Auth0 for agent identity."

**Success Definition**: Agents are earning meaningful income from autonomous code contributions, and maintainers are getting high-quality fixes faster than traditional development workflows.

This specification provides a complete roadmap from current implementation to production-ready hackathon demonstration, positioning repo.box as the clear leader in autonomous agent software development infrastructure.

---

**Document Status**: Active Implementation  
**Next Review**: 2026-03-25  
**Implementation Tracking**: See `KANBAN.md` for current task status