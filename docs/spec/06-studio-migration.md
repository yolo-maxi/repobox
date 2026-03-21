# repo.box Spec: Studio Projects Migration

## Overview

This specification details the migration of all studio projects to repo.box, adding `.repobox/config.yml` configuration files and pushing to git.repo.box with signed commits. The migration establishes a multi-agent development pipeline with proper EVM-based permissions and identity management.

## Studio Projects Inventory

### Primary Projects

| Project | Path | Type | Description |
|---------|------|------|-------------|
| **SSS (Streaming Superfluid Systems)** | `/home/xiko/sss` | Monorepo (pnpm) | DeFi superfluid streaming platform with contracts + website |
| **Oceangram Daemon** | `/home/xiko/oceangram-daemon` | Node.js Service | Standalone Telegram client daemon (gramjs over HTTP) |
| **Oceangram Tray** | `/home/xiko/oceangram-tray` | Desktop App | System tray interface for Oceangram |
| **Streme Frontend** | `/home/xiko/streme-frontend` | Next.js App | Token launcher UI for Superfluid streams |
| **Beamr Economy** | `/home/xiko/beamr-economy` | React/Vite | Economic visualization dashboard |

### Supporting Projects

| Project | Path | Type | Description |
|---------|------|------|-------------|
| **Rikai UI** | `/home/xiko/rikai-ui` | Next.js App | Language reading assistant interface |
| **Rikai Admin Bot** | `/home/xiko/rikai-admin-bot` | Node.js Bot | Administrative bot for Rikai |
| **Prompster** | `/home/xiko/prompster` | Web App | Prompt engineering tool |
| **Prompster Bot** | `/home/xiko/prompster-bot` | Node.js Bot | Bot interface for Prompster |
| **Watson Web** | `/home/xiko/watson-web` | Web App | Watson interface |
| **Pool Admin** | `/home/xiko/pool-admin` | Web App | Pool administration interface |
| **Takopi** | `/home/xiko/takopi` | Application | Project management tool |
| **Banger** | `/home/xiko/banger` | Application | Audio/music tool |
| **Markdown Kanban** | `/home/xiko/markdown-kanban` | Tool | Kanban board implementation |

### Experimental Projects

| Project | Path | Type | Description |
|---------|------|------|-------------|
| **SSS GitHub** | `/home/xiko/sss-github` | Integration | GitHub integration for SSS |
| **Tradestrategy Work** | `/home/xiko/tradestrategy-work` | Contracts | Trading strategy smart contracts |
| **Test Repo** | `/home/xiko/test-repo` | Testing | Testing repository |

## Agent Identity Matrix

### Core Agents

| Agent Type | EVM Address | Alias | Role |
|------------|-------------|-------|------|
| **PM Agent** | `0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b` | `@pm-agent` | Product management, specs, kanban |
| **Claude Agent** | `0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00` | `@claude-agent` | General development, features |
| **Codex Agent** | `0x82240a161Fea724D059a74F948C8E18674c0fA09` | `@codex-agent` | Code generation, refactoring |
| **Reviewer Agent** | `0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307` | `@reviewer-agent` | Code review, QA |
| **Mergeooor (Ocean)** | `0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048` | `@ocean-mergeooor` | Final merge authority |

### Human Founders

| Human | EVM Address | Alias | Role |
|-------|-------------|-------|------|
| **Fran** | `0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b` | `@fran-founder` | Founder, final authority |

## .repobox/config.yml Templates

### Template 1: High-Security Monorepo (SSS, Streme, Core Projects)

For projects with smart contracts, production deployments, or sensitive configurations.

```yaml
# .repobox/config.yml — High-Security Monorepo
# Four-tier agent pipeline with strict file controls

groups:
  founders:
    - evm:0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b  # Fran
    - evm:0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048  # Ocean (Mergeooor)

  product:
    - evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b  # PM agent

  devs:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00  # Claude
    - evm:0x82240a161Fea724D059a74F948C8E18674c0fA09  # Codex

  reviewers:
    - evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307  # Reviewer agent

permissions:
  default: deny
  rules:
    # ── Founders ── Full control ─────────────────
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders delete >*
    - founders force-push >*
    - founders edit *

    # ── PM ── Documentation and project management only ──
    - product push >pm/**
    - product create >pm/**
    - product edit ./README.md
    - product edit ./docs/**
    - product edit ./KANBAN.md
    - product edit ./*.md
    - product append ./CHANGELOG.md
    - product not edit ./packages/contracts/**
    - product not edit ./src/**
    - product not edit ./package.json
    - product not edit ./.env*
    - product not edit .repobox/config.yml

    # ── Devs ── Feature development branches ──────
    - devs push >feature/**
    - devs push >fix/**
    - devs push >chore/**
    - devs create >feature/**
    - devs create >fix/**
    - devs create >chore/**
    - devs edit * >feature/**
    - devs edit * >fix/**
    - devs edit * >chore/**
    - devs append ./KANBAN.md >feature/**
    - devs append ./KANBAN.md >fix/**
    - devs append ./CHANGELOG.md >feature/**
    - devs not edit .repobox/config.yml
    - devs not edit ./.env.production
    - devs not merge >main
    - devs not merge >staging

    # ── Reviewers ── Review branches and logging ──
    - reviewers push >review/**
    - reviewers create >review/**
    - reviewers merge >feature/** >review/**
    - reviewers merge >fix/** >review/**
    - reviewers edit * >review/**
    - reviewers append ./REVIEWS.md
    - reviewers append ./KANBAN.md
    - reviewers not edit .repobox/config.yml
    - reviewers not merge >main
    - reviewers not merge >staging
```

**Applies to:** SSS, Streme Frontend, Beamr Economy

### Template 2: Standard Service (Oceangram, Bots)

For Node.js services, bots, and standalone applications with moderate security needs.

```yaml
# .repobox/config.yml — Standard Service
# Streamlined agent pipeline for service development

groups:
  founders:
    - evm:0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b  # Fran
    - evm:0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048  # Ocean (Mergeooor)

  product:
    - evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b  # PM agent

  devs:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00  # Claude
    - evm:0x82240a161Fea724D059a74F948C8E18674c0fA09  # Codex

  reviewers:
    - evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307  # Reviewer agent

permissions:
  default: allow
  rules:
    # ── Founders ── Full control ─────────────────
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders delete >*
    - founders edit *

    # ── PM ── Documentation focus ────────────────
    - product edit ./README.md
    - product edit ./docs/**
    - product edit ./KANBAN.md
    - product edit ./*.md
    - product append ./CHANGELOG.md
    - product not edit ./package.json
    - product not edit ./.env*
    - product not edit .repobox/config.yml

    # ── Devs ── Feature branches with src access ─
    - devs push >feature/**
    - devs push >fix/**
    - devs create >feature/**
    - devs create >fix/**
    - devs edit * >feature/**
    - devs edit * >fix/**
    - devs append ./KANBAN.md
    - devs append ./CHANGELOG.md
    - devs not edit .repobox/config.yml
    - devs not merge >main

    # ── Reviewers ── Review and merge feature work 
    - reviewers merge >feature/** >main
    - reviewers merge >fix/** >main
    - reviewers append ./REVIEWS.md
    - reviewers not edit .repobox/config.yml
```

**Applies to:** Oceangram Daemon, Oceangram Tray, Rikai Admin Bot, Prompster Bot

### Template 3: Lightweight Tool (Utils, Experiments)

For tools, utilities, and experimental projects with minimal restrictions.

```yaml
# .repobox/config.yml — Lightweight Tool
# Minimal restrictions for rapid development

groups:
  founders:
    - evm:0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b  # Fran
    - evm:0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048  # Ocean (Mergeooor)

  agents:
    - evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b  # PM agent
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00  # Claude
    - evm:0x82240a161Fea724D059a74F948C8E18674c0fA09  # Codex
    - evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307  # Reviewer agent

permissions:
  default: allow
  rules:
    # ── Founders ── Full control ─────────────────
    - founders push >*
    - founders merge >*
    - founders create >*
    - founders delete >*
    - founders edit *

    # ── Agents ── Feature development ────────────
    - agents push >feature/**
    - agents push >fix/**
    - agents push >dev
    - agents create >feature/**
    - agents create >fix/**
    - agents merge >feature/** >dev
    - agents merge >fix/** >dev
    - agents edit *
    - agents not edit .repobox/config.yml
    - agents not push >main
    - agents not merge >main
```

**Applies to:** Markdown Kanban, Takopi, Banger, Watson Web, Pool Admin, Test Repo

## Migration Steps

### Phase 1: Core Infrastructure Setup

#### 1.1 Create Agent Aliases

```bash
# Set up local aliases for all agent identities
git repobox alias add pm-agent evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b
git repobox alias add claude-agent evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
git repobox alias add codex-agent evm:0x82240a161Fea724D059a74F948C8E18674c0fA09
git repobox alias add reviewer-agent evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307
git repobox alias add ocean-mergeooor evm:0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048
git repobox alias add fran-founder evm:0x69C2920CA309577bcd79e4e6e3afdda93287Cc8b
```

#### 1.2 Verify repo.box Remote Connectivity

```bash
# Test connection to git.repo.box
git ls-remote https://git.repo.box/test.git
```

### Phase 2: Project-by-Project Migration

For each project in the inventory, execute the following migration script:

#### 2.1 Migration Script Template

```bash
#!/bin/bash
# migrate-project.sh <project_path> <template_type>

PROJECT_PATH="$1"
TEMPLATE_TYPE="$2"  # high-security | standard-service | lightweight-tool
PROJECT_NAME=$(basename "$PROJECT_PATH")

cd "$PROJECT_PATH" || exit 1

echo "🚀 Migrating $PROJECT_NAME to repo.box..."

# 1. Initialize repo.box in project
git repobox init

# 2. Create .repobox directory if it doesn't exist
mkdir -p .repobox

# 3. Copy appropriate config template
case "$TEMPLATE_TYPE" in
  "high-security")
    cp /home/xiko/repobox/docs/spec/templates/high-security-config.yml .repobox/config.yml
    ;;
  "standard-service")
    cp /home/xiko/repobox/docs/spec/templates/standard-service-config.yml .repobox/config.yml
    ;;
  "lightweight-tool")
    cp /home/xiko/repobox/docs/spec/templates/lightweight-tool-config.yml .repobox/config.yml
    ;;
  *)
    echo "❌ Invalid template type: $TEMPLATE_TYPE"
    exit 1
    ;;
esac

# 4. Set PM agent identity for the configuration commit
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=user.signingkey
export GIT_CONFIG_VALUE_0=evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b

# 5. Add and commit the configuration
git add .repobox/config.yml
git commit -m "feat: add repo.box configuration

- Add .repobox/config.yml with $TEMPLATE_TYPE template
- Establish EVM-based permissions for multi-agent development
- Enable signed commits with agent identities"

# 6. Add repo.box remote
git remote add repobox https://git.repo.box/${PROJECT_NAME}.git

# 7. Push to repo.box with signed commits
git push -u repobox main

echo "✅ $PROJECT_NAME migrated successfully to repo.box"
echo "📍 Repository URL: https://git.repo.box/${PROJECT_NAME}.git"
```

#### 2.2 High-Security Projects Migration

```bash
# SSS (Superfluid Streaming Systems)
./migrate-project.sh /home/xiko/sss high-security

# Streme Frontend
./migrate-project.sh /home/xiko/streme-frontend high-security

# Beamr Economy
./migrate-project.sh /home/xiko/beamr-economy high-security
```

#### 2.3 Standard Service Projects Migration

```bash
# Oceangram Daemon
./migrate-project.sh /home/xiko/oceangram-daemon standard-service

# Oceangram Tray
./migrate-project.sh /home/xiko/oceangram-tray standard-service

# Rikai UI
./migrate-project.sh /home/xiko/rikai-ui standard-service

# Rikai Admin Bot
./migrate-project.sh /home/xiko/rikai-admin-bot standard-service

# Prompster
./migrate-project.sh /home/xiko/prompster standard-service

# Prompster Bot
./migrate-project.sh /home/xiko/prompster-bot standard-service
```

#### 2.4 Lightweight Tool Projects Migration

```bash
# Watson Web
./migrate-project.sh /home/xiko/watson-web lightweight-tool

# Pool Admin
./migrate-project.sh /home/xiko/pool-admin lightweight-tool

# Takopi
./migrate-project.sh /home/xiko/takopi lightweight-tool

# Banger
./migrate-project.sh /home/xiko/banger lightweight-tool

# Markdown Kanban
./migrate-project.sh /home/xiko/markdown-kanban lightweight-tool

# Test Repo (experimental)
./migrate-project.sh /home/xiko/test-repo lightweight-tool
```

### Phase 3: Advanced Configuration

#### 3.1 Project-Specific Permission Rules

Some projects may require custom permission rules beyond the templates:

**SSS Monorepo Custom Rules:**
```yaml
# Additional rules for SSS contract management
- devs not edit ./packages/contracts/src/**
- devs not edit ./packages/contracts/foundry.toml
- reviewers edit ./packages/contracts/test/**
- reviewers not edit ./packages/contracts/src/**
```

**Oceangram Daemon Custom Rules:**
```yaml
# Daemon-specific restrictions
- devs not edit ./src/auth/**
- devs not edit ./config/production.json
- reviewers edit ./src/auth/** >review/**
```

#### 3.2 CI/CD Integration Permissions

For projects with automated deployment:

```yaml
# CI/CD bot permissions (future enhancement)
ci-bots:
  - evm:0x[CI_BOT_ADDRESS]

# CI/CD specific rules
- ci-bots push >deploy/**
- ci-bots create >deploy/**
- ci-bots not edit .repobox/config.yml
- ci-bots not push >main
```

### Phase 4: Agent Workflow Setup

#### 4.1 PM Agent Git Configuration

```bash
# Global PM agent configuration
git config --global user.name "pm-agent"
git config --global user.email "pm@repo.box"
git config --global user.signingkey "evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b"
git config --global commit.gpgsign true
```

#### 4.2 Development Agent Configurations

**Claude Agent:**
```bash
git config --global user.name "claude-agent"
git config --global user.email "claude@repo.box" 
git config --global user.signingkey "evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00"
```

**Codex Agent:**
```bash
git config --global user.name "codex-agent"
git config --global user.email "codex@repo.box"
git config --global user.signingkey "evm:0x82240a161Fea724D059a74F948C8E18674c0fA09"
```

**Reviewer Agent:**
```bash
git config --global user.name "reviewer-agent"
git config --global user.email "reviewer@repo.box"
git config --global user.signingkey "evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307"
```

## Verification Process

### 5.1 Configuration Validation

For each migrated project, run comprehensive checks:

```bash
#!/bin/bash
# verify-migration.sh <project_path>

PROJECT_PATH="$1"
PROJECT_NAME=$(basename "$PROJECT_PATH")

cd "$PROJECT_PATH" || exit 1

echo "🔍 Verifying $PROJECT_NAME migration..."

# 1. Check .repobox/config.yml exists and is valid
if [[ ! -f .repobox/config.yml ]]; then
  echo "❌ Missing .repobox/config.yml"
  exit 1
fi

# 2. Lint the configuration
if ! git repobox lint; then
  echo "❌ Configuration lint failed"
  exit 1
fi

# 3. Test permission checks for all agent types
echo "Testing PM agent permissions..."
git repobox check evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b edit ./README.md
git repobox check evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b edit ./src/main.rs

echo "Testing dev agent permissions..."
git repobox check evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00 push ">feature/test"
git repobox check evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00 push ">main"

echo "Testing reviewer permissions..."
git repobox check evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307 merge ">feature/test >main"

# 4. Verify repo.box remote
if ! git remote | grep -q repobox; then
  echo "❌ Missing repobox remote"
  exit 1
fi

# 5. Test push connectivity
if ! git ls-remote repobox; then
  echo "❌ Cannot connect to repobox remote"
  exit 1
fi

# 6. Check commit signatures
LAST_COMMIT=$(git rev-parse HEAD)
if ! git verify-commit "$LAST_COMMIT"; then
  echo "❌ Last commit signature verification failed"
  exit 1
fi

echo "✅ $PROJECT_NAME migration verified successfully"
```

### 5.2 End-to-End Agent Workflow Test

Test the complete agent workflow with a sample feature:

```bash
#!/bin/bash
# test-agent-workflow.sh <project_path>

PROJECT_PATH="$1"
cd "$PROJECT_PATH" || exit 1

echo "🧪 Testing complete agent workflow..."

# 1. PM agent creates specification
export GIT_CONFIG_COUNT=1
export GIT_CONFIG_KEY_0=user.signingkey
export GIT_CONFIG_VALUE_0=evm:0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b

git checkout -b pm/test-workflow-spec
echo "# Test Feature Specification" > docs/test-feature-spec.md
echo "This is a test specification created by PM agent." >> docs/test-feature-spec.md
git add docs/test-feature-spec.md
git commit -m "spec: add test feature specification"
git push repobox pm/test-workflow-spec

# 2. Dev agent implements feature
export GIT_CONFIG_VALUE_0=evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00

git checkout main
git checkout -b feature/test-implementation
echo "// Test implementation by Claude agent" > src/test-feature.js
git add src/test-feature.js
git commit -m "feat: implement test feature

Based on specification in docs/test-feature-spec.md
- Add basic test feature implementation
- Ready for review"
git push repobox feature/test-implementation

# 3. Reviewer agent reviews and creates review branch
export GIT_CONFIG_VALUE_0=evm:0xe4D4438Fd215c2befe8ef3fB78E72e14e011C307

git checkout -b review/test-implementation-review
echo "# Code Review: Test Implementation" > REVIEWS.md
echo "- Implementation follows specification ✓" >> REVIEWS.md
echo "- Code quality acceptable ✓" >> REVIEWS.md
echo "- Ready for merge ✓" >> REVIEWS.md
git add REVIEWS.md
git commit -m "review: approve test implementation

All checks passed:
- Specification compliance
- Code quality
- Test coverage"

# 4. Founder/Mergeooor merges to main (manual step)
echo "✅ Workflow test complete. Manual merge to main required by founder."
echo "To complete: git checkout main && git merge feature/test-implementation"
```

### 5.3 Security Audit

Run security checks on all configurations:

```bash
#!/bin/bash
# audit-security.sh

echo "🔒 Running security audit on all migrated projects..."

for project in /home/xiko/sss /home/xiko/oceangram-daemon /home/xiko/streme-frontend; do
  if [[ -f "$project/.repobox/config.yml" ]]; then
    echo "Auditing $(basename $project)..."
    
    # Check for overly permissive rules
    if grep -q "default: allow" "$project/.repobox/config.yml" && grep -q "edit \*" "$project/.repobox/config.yml"; then
      echo "⚠️  $(basename $project): Broad edit permissions with default allow"
    fi
    
    # Check config file protection
    if ! grep -q "not edit .repobox/config.yml" "$project/.repobox/config.yml"; then
      echo "❌ $(basename $project): Config file not protected from agents"
    fi
    
    # Check main branch protection
    if ! grep -q "not.*>main" "$project/.repobox/config.yml"; then
      echo "⚠️  $(basename $project): Main branch may not be protected"
    fi
    
    echo "✅ $(basename $project) security check complete"
  fi
done

echo "🔒 Security audit complete"
```

## Project-Specific Configurations

### SSS (Superfluid Streaming Systems)

**Special Requirements:**
- Smart contract source protection (`packages/contracts/src/**`)
- Environment configuration protection (`.env.production`, `.env.mainnet`)
- Package.json modification restrictions
- Separate test contract editing permissions

**Additional Config:**
```yaml
# SSS-specific rules (append to high-security template)
- devs not edit ./packages/contracts/src/**
- devs not edit ./packages/contracts/foundry.toml
- devs not edit ./.env.production
- devs not edit ./.env.mainnet
- reviewers edit ./packages/contracts/test/** >review/**
- reviewers append ./packages/contracts/AUDIT.md
```

### Oceangram Projects

**Special Requirements:**
- Authentication module protection
- Production configuration protection
- API key and session management restrictions

**Additional Config:**
```yaml
# Oceangram-specific rules
- devs not edit ./src/auth/**
- devs not edit ./config/production.json
- devs not edit ./src/session-manager.ts
- reviewers edit ./src/auth/** >review/**
```

### Streme Frontend

**Special Requirements:**
- Environment variable protection
- Deployment configuration restrictions
- Farcaster integration protection

**Additional Config:**
```yaml
# Streme-specific rules  
- devs not edit ./.env.production
- devs not edit ./vercel.json
- devs not edit ./src/lib/auth/**
- reviewers edit ./src/lib/auth/** >review/**
```

## Implementation Timeline

### Week 1: Infrastructure & High-Security Projects
- Day 1-2: Set up agent aliases and test repo.box connectivity
- Day 3-4: Migrate SSS, Streme Frontend, Beamr Economy (high-security)
- Day 5: Verification and security audit of high-security projects

### Week 2: Standard Services
- Day 1-3: Migrate Oceangram projects, Rikai projects, Prompster projects
- Day 4-5: Verification and workflow testing of standard services

### Week 3: Lightweight Tools & Testing
- Day 1-2: Migrate remaining tools and utilities
- Day 3-4: End-to-end workflow testing across all projects
- Day 5: Final security audit and documentation updates

### Week 4: Agent Training & Optimization
- Day 1-3: Train agents on repo.box workflows
- Day 4-5: Optimize configurations based on usage patterns

## Success Metrics

### Technical Metrics
- ✅ All 15+ studio projects have valid `.repobox/config.yml` files
- ✅ All projects successfully push to git.repo.box with signed commits
- ✅ Permission system blocks unauthorized actions (tested via `git repobox check`)
- ✅ All agent identities can perform their designated roles
- ✅ Main branch protection prevents direct agent pushes

### Workflow Metrics
- ✅ PM agent can edit documentation and specifications
- ✅ Dev agents can create feature branches and implement changes
- ✅ Reviewer agent can review and approve changes
- ✅ Founder/Mergeooor can merge approved changes to main
- ✅ Configuration changes require founder approval

### Security Metrics
- ✅ No agent can modify `.repobox/config.yml` (except founders)
- ✅ No agent can directly push to main branch
- ✅ All commits are cryptographically signed with agent identities
- ✅ Permission escalation is structurally impossible
- ✅ Audit trail shows all changes with agent attribution

## Post-Migration Operations

### Daily Operations
1. **PM Agent**: Updates project documentation, kanban boards, specifications
2. **Dev Agents**: Creates feature branches, implements changes, maintains code
3. **Reviewer Agent**: Reviews pull requests, approves merges, maintains quality
4. **Mergeooor**: Final approval and merge to main branches

### Weekly Maintenance
1. **Security Review**: Audit permission configurations for any unauthorized changes
2. **Workflow Optimization**: Review agent performance and adjust permissions if needed
3. **Documentation Updates**: Keep specifications current with actual implementations

### Emergency Procedures
1. **Agent Compromise**: Revoke agent's EVM address from all group configurations
2. **Permission Escalation**: Review audit trail, revert unauthorized config changes
3. **System Recovery**: Restore from git history, re-verify all signatures

## Future Enhancements

### Planned Features
- **CI/CD Integration**: Add automated testing bots with restricted permissions
- **Read Access Control**: Implement repository privacy levels
- **Tag Management**: Add semantic versioning controls
- **Branch Policies**: Implement required reviews and status checks

### Scaling Considerations
- **Additional Agents**: Framework supports unlimited agent identities
- **Project Templates**: Create specialized templates for new project types
- **Cross-Project Permissions**: Enable agents to work across multiple repositories
- **Integration APIs**: Connect with external project management tools

---

This specification establishes a comprehensive framework for migrating all studio projects to repo.box with proper EVM-based identity and permission management. The multi-tier agent pipeline ensures secure, auditable development while maintaining development velocity and code quality.