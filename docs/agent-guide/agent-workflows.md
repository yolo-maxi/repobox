# Agent Workflows

Practical patterns and workflows for AI agents using repo.box.

## Standard Agent Workflow

### 1. Feature Development Pattern

The most common pattern for agent development work:

```yaml
# Configuration
permissions:
  default: deny
  rules:
    # Human maintainers
    - maintainers own >*
    
    # Agent workflow
    - agents read *                    # 1. Read existing code
    - agents branch >feature/**        # 2. Create feature branch
    - agents push >feature/**          # 3. Push development work
    - agents edit * >feature/**        # 4. Edit any file on feature branches
    - agents append CHANGELOG.md       # 5. Update changelog
    
    # Human oversight
    - agents not push >main            # Humans review before main
    - agents not merge >*              # Humans control merging
```

**Agent workflow steps:**
1. **Clone and read:** `git clone` + explore codebase
2. **Create feature branch:** `git checkout -b feature/new-api`
3. **Develop and commit:** Make changes, commit with EVM signature
4. **Push for review:** `git push origin feature/new-api`
5. **Human review and merge:** Maintainer reviews and merges to main

### 2. Documentation Agent Pattern

Specialized agent for documentation maintenance:

```yaml
permissions:
  default: deny  
  rules:
    - maintainers own >*
    
    # Documentation agent can work anywhere
    - doc-agent read *
    - doc-agent edit docs/** >*         # Edit docs on any branch
    - doc-agent edit *.md >*            # Edit README files anywhere
    - doc-agent append CHANGELOG.md     # Update changelog
    - doc-agent push >docs/**           # Push to docs branches
    - doc-agent branch >docs/**         # Create docs branches
    
    # Protection
    - doc-agent not edit src/**         # Cannot modify source code
    - doc-agent not edit .repobox/**    # Cannot modify permissions
```

### 3. Bug Fix Workflow

Quick bug fixes with limited scope:

```yaml
permissions:
  rules:
    # Hotfix workflow for critical bugs
    - bugfix-agent read *
    - bugfix-agent branch >hotfix/**
    - bugfix-agent push >hotfix/**
    - bugfix-agent edit src/** >hotfix/**
    - bugfix-agent edit tests/** >hotfix/**
    
    # Emergency merge capability (careful!)
    - senior-agents merge >main         # For critical fixes only
    
    # Standard fixes go through review
    - bugfix-agent not merge >main      # Normal process
```

## Branch Naming Conventions

### Standard Patterns

Use consistent branch naming for clarity:

```bash
# Feature development
feature/add-authentication
feature/improve-performance  
feature/user-dashboard

# Bug fixes
fix/login-validation
fix/memory-leak
fix/broken-tests

# Documentation  
docs/api-reference
docs/setup-guide
docs/troubleshooting

# Hotfixes (emergency)
hotfix/critical-security-fix
hotfix/production-crash
```

### Agent-Specific Naming

Include agent identifier for multi-agent teams:

```bash
# Agent-specific branches
claude/feature/user-auth
codex/fix/performance-bug
docs-agent/docs/api-update

# Or use standardized format
agent-claude/feature-user-auth
agent-codex/fix-performance
agent-docs/update-readme
```

### Bounty Hunting Format (Virtuals Integration)

**Required format** for automatic payment processing:

```bash
agent/{agent-id}/fix-{issue-number}

# Examples:
agent/0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-42
agent/claude.eth/fix-123
agent/0xAAc0.../fix-456
```

**Configuration for bounty hunting:**
```yaml
virtuals:
  enabled: true
  bug_bounties:
    critical: "50.00"
    high: "25.00"
    medium: "10.00" 
    low: "5.00"

permissions:
  rules:
    - bounty-hunters branch >agent/**
    - bounty-hunters push >agent/**  
    - bounty-hunters edit * >agent/**
    - bounty-hunters not force-push >agent/**  # Required for bounty validation
```

## Commit Message Patterns

### Standard Format
Use conventional commits for consistency:

```bash
# Format: type(scope): description
feat(auth): add OAuth2 integration
fix(api): resolve rate limiting bug  
docs(readme): update installation instructions
test(auth): add unit tests for login flow
refactor(db): optimize query performance
```

### Agent Identification
Include agent context in commit messages:

```bash
# Agent attribution
feat(auth): add OAuth2 integration

Implemented by: claude-agent
Context: User authentication feature request
Testing: All auth tests passing

# Or in footer  
feat(auth): add OAuth2 integration

Co-authored-by: claude-agent <claude@agents.example.com>
```

### Bounty Commit Format (Virtuals)

**Required format** for automatic bounty processing:

```bash
fix: description fixes #issue-number

# Examples:
fix: resolve authentication bug fixes #42  
fix: patch memory leak fixes #123
feat: implement user dashboard fixes #456
```

The commit message must:
- Use conventional commit format
- Include issue reference with `fixes #number`
- Match the branch's issue number

## Multi-Agent Collaboration

### Sequential Workflow
Agents work in sequence on the same codebase:

```yaml
# Agent pipeline: research → development → testing → docs
permissions:
  rules:
    # Research agent: read-only analysis
    - research-agent read *
    - research-agent append research-notes.md
    
    # Development agent: implements features  
    - dev-agent read *
    - dev-agent edit src/** >feature/**
    - dev-agent push >feature/**
    
    # Testing agent: adds tests
    - test-agent read *
    - test-agent edit tests/** >feature/**  
    - test-agent push >feature/**
    
    # Docs agent: updates documentation
    - docs-agent read *
    - docs-agent edit docs/** >feature/**
    - docs-agent push >feature/**
```

### Parallel Workflow
Multiple agents working simultaneously:

```yaml
# Different agents, different areas
permissions:
  rules:
    # Frontend agent
    - frontend-agent edit frontend/** >*
    - frontend-agent push >frontend/**
    
    # Backend agent  
    - backend-agent edit backend/** >*
    - backend-agent push >backend/**
    
    # Shared areas need coordination
    - "* edit shared/** >feature/**"    # Anyone can edit shared code
    - "* not edit shared/** >main"      # But only on feature branches
```

### Sub-Agent Specialization

Use the `+` notation for specialized sub-agents:

```yaml
groups:
  # Main agent
  claude:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
  
  # Specialized sub-agents
  claude-bugfix:
    - evm:0x1111...  # claude+bugfix
  claude-docs:  
    - evm:0x2222...  # claude+docs
  claude-tests:
    - evm:0x3333...  # claude+tests

permissions:
  rules:
    # Main agent: general development
    - claude edit src/** >feature/**
    - claude push >feature/**
    
    # Bugfix specialist: can work on hotfixes
    - claude-bugfix edit src/** >hotfix/**
    - claude-bugfix push >hotfix/**
    - claude-bugfix merge >main        # Emergency fixes
    
    # Docs specialist: documentation anywhere
    - claude-docs edit docs/** >*
    - claude-docs edit *.md >*
    - claude-docs push >docs/**
    
    # Test specialist: comprehensive testing
    - claude-tests edit tests/** >*
    - claude-tests edit src/**/*.test.* >*
    - claude-tests push >test/**
```

## Quality Assurance Patterns

### Code Review Workflow
Agents can participate in code review:

```yaml
permissions:
  rules:
    # Review agent: read-only analysis + feedback
    - review-agent read *
    - review-agent append review-comments.md
    - review-agent branch >review/**
    - review-agent push >review/**
    
    # Cannot modify the actual code under review
    - review-agent not edit src/**
    - review-agent not edit tests/**
```

### Testing Integration  
Agents can maintain comprehensive test suites:

```yaml
permissions:
  rules:
    # Test agent can modify tests anywhere
    - test-agent edit tests/** >*
    - test-agent edit **/*.test.* >*
    - test-agent edit **/*.spec.* >*
    
    # Can create test branches
    - test-agent branch >test/**
    - test-agent push >test/**
    
    # Can run CI/deploy test results
    - test-agent append test-results.md
    - test-agent edit .github/workflows/** >test/**
```

### Security Review Pattern
Specialized security analysis:

```yaml
permissions:
  rules:
    # Security agent: comprehensive read access
    - security-agent read *
    
    # Can document findings
    - security-agent append security-report.md
    - security-agent branch >security/**
    - security-agent push >security/**
    
    # Cannot modify code (analysis only)
    - security-agent not edit src/**
    - security-agent not edit tests/**
    - security-agent not merge >*
```

## Integration Patterns

### CI/CD Integration
Agents can integrate with automated systems:

```yaml
permissions:
  rules:
    # CI agent: automated operations
    - ci-agent read *
    - ci-agent push >release/**         # Release candidates
    - ci-agent append build-log.txt     # Build results
    - ci-agent edit dist/** >release/** # Built artifacts
    
    # Deployment permissions  
    - deploy-agent read *
    - deploy-agent push >deploy/**      # Deployment branches
    - deploy-agent edit deployment/** >deploy/**
```

### Monitoring Integration
Agents can maintain operational awareness:

```yaml
permissions:
  rules:
    # Monitoring agent: observability
    - monitor-agent read *
    - monitor-agent append metrics.md
    - monitor-agent edit monitoring/** >*
    - monitor-agent edit alerts/** >*
    
    # Can create incident response branches
    - monitor-agent branch >incident/**
    - monitor-agent push >incident/**
```

## Error Handling Patterns

### Permission Checking
Agents should verify permissions before attempting operations:

```bash
#!/bin/bash
# agent-push.sh - Safe push script for agents

BRANCH=$(git branch --show-current)
AGENT=$(git repobox whoami | cut -d' ' -f1)

# Check push permission
if git repobox check "$AGENT" push ">$BRANCH"; then
    echo "✓ Permission granted for push to $BRANCH"
    git push origin "$BRANCH"
else
    echo "❌ Permission denied for push to $BRANCH"
    echo "Agent: $AGENT, Branch: $BRANCH"
    exit 1
fi
```

### Graceful Degradation
Handle permission limitations gracefully:

```bash
#!/bin/bash
# agent-commit.sh - Intelligent commit handling

AGENT=$(git repobox whoami | cut -d' ' -f1)

# Try to commit to current branch
if git repobox check "$AGENT" edit "*" ">$(git branch --show-current)"; then
    git commit -m "$1"
else
    echo "Cannot commit to $(git branch --show-current)"
    echo "Switching to feature branch..."
    
    # Create feature branch if needed
    git checkout -b "feature/agent-work-$(date +%s)"
    git commit -m "$1"
    echo "Committed to feature branch: $(git branch --show-current)"
fi
```

### Fallback Strategies
Implement fallback approaches when primary actions are blocked:

```bash
#!/bin/bash  
# agent-document.sh - Documentation with fallbacks

AGENT=$(git repobox whoami | cut -d' ' -f1)
FILE="$1"
CONTENT="$2"

# Try direct edit first
if git repobox check "$AGENT" edit "$FILE"; then
    echo "$CONTENT" >> "$FILE"
    echo "✓ Updated $FILE directly"
    
# Try append if edit is denied
elif git repobox check "$AGENT" append "$FILE"; then
    echo "$CONTENT" >> "$FILE"
    echo "✓ Appended to $FILE"
    
# Create new file if existing file can't be modified
elif git repobox check "$AGENT" upload "$(dirname "$FILE")/agent-notes-$(basename "$FILE")"; then
    echo "$CONTENT" > "$(dirname "$FILE")/agent-notes-$(basename "$FILE")"
    echo "✓ Created new file: agent-notes-$(basename "$FILE")"
    
else
    echo "❌ No permission to document in this location"
    exit 1
fi
```

## Best Practices

### 1. Clear Role Definition
Define specific roles for each agent:

```yaml
# Good: Clear, specific roles
groups:
  api-developers:
    - backend-agent
  ui-developers:  
    - frontend-agent
  documentation-writers:
    - docs-agent

# Avoid: Generic, overlapping roles
groups:
  agents:
    - all-purpose-agent  # Too broad
```

### 2. Scope Limitation
Limit agent access to necessary areas:

```yaml
# Good: Scoped permissions
rules:
  - api-agent edit api/** >*
  - ui-agent edit frontend/** >*
  
# Avoid: Overprivileged access  
rules:
  - agent edit * >*  # Too broad
```

### 3. Human Oversight
Maintain human control over critical operations:

```yaml
rules:
  # Agents can develop
  - agents push >feature/**
  - agents edit src/** >feature/**
  
  # Humans control releases
  - humans merge >main
  - humans push >main
  - humans delete >*
```

### 4. Audit Trail
Ensure all agent actions are traceable:

```bash
# All commits are EVM-signed
git log --show-signature

# Verify specific agent commits
git verify-commit HEAD
git log --pretty=format:"%h %an %s" --grep="claude-agent"
```

### 5. Testing Before Production
Test agent workflows in safe environments:

```yaml
# Test repository permissions
permissions:
  rules:
    # Agents can experiment freely in test repos
    - agents own >*              # Full access for testing
    
    # But production repos are restricted
    - agents edit src/** >feature/**  # Limited scope
```

## Troubleshooting Agent Workflows

### Common Issues

**Permission denied during push:**
```bash
# Check current permissions
git repobox whoami
git repobox check $(git repobox whoami | cut -d' ' -f1) push ">$(git branch --show-current)"

# Switch to allowed branch if needed
git checkout -b feature/agent-work
```

**Identity confusion:**
```bash
# Verify agent identity
git repobox whoami
# Should show expected agent name and address

# Reset if wrong
git repobox use correct-agent-name
```

**Configuration errors:**
```bash
# Validate before agent runs
git repobox lint

# Test agent permissions  
git repobox check agent-name push ">feature/test"
git repobox check agent-name edit "src/**" ">feature/test"
```

### Monitoring Agent Activity

Track agent behavior:
```bash
# Show agent commits
git log --author="claude-agent" --oneline

# Show signature verification
git log --show-signature --grep="agent"

# Check current permissions
git repobox status
```

## Next Steps

**Learn about bounty hunting:** Continue to [Bounty Hunting](bounty-hunting.md) for Virtuals integration.

**Explore multi-agent patterns:** See [Multi-Agent Repos](multi-agent-repos.md) for team coordination.

**Set up monitoring:** Check [Troubleshooting](../user-guide/troubleshooting.md) for debugging workflows.

**Review examples:** See the [User Guide](../user-guide/permission-system.md) for more permission patterns.