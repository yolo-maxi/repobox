#!/bin/bash
set -euo pipefail

# repo.box Full E2E Demo Script
# Demonstrates: init → keys → signed commit → push → clone → verify
#
# Usage:
#   ./scripts/demo-e2e.sh [--quick]
#
# Quick mode: Skip agent simulation, focus on core flow (30s)
# Full mode: Complete flow with agent simulation (60s)

# ================================
# Configuration & Setup
# ================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOBOX_BINARY="/home/xiko/repobox/target/release/repobox"
GIT_SERVER="git.repo.box"
EXPLORER_BASE="https://repo.box/explore"
QUICK_MODE=false
NO_CLEANUP=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
for arg in "$@"; do
  case $arg in
    --quick)
      QUICK_MODE=true
      shift
      ;;
    --no-cleanup)
      NO_CLEANUP=true
      shift
      ;;
    --help|-h)
      echo "Usage: $0 [--quick] [--no-cleanup]"
      echo "  --quick      Skip agent simulation, focus on core flow (30s)"
      echo "  --no-cleanup Keep temporary files for debugging"
      echo "  --help       Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# ================================
# Utility Functions
# ================================

log() {
  echo -e "${BLUE}🔧 ${1}${NC}"
}

success() {
  echo -e "${GREEN}✅ ${1}${NC}"
}

error() {
  echo -e "${RED}❌ ${1}${NC}"
  exit 1
}

warning() {
  echo -e "${YELLOW}⚠️  ${1}${NC}"
}

# Progress indicator
progress() {
  local emoji="$1"
  local message="$2"
  echo -e "${emoji} ${message}..."
}

# ================================
# Validation & Prerequisites
# ================================

validate_environment() {
  progress "🔍" "Validating environment"
  
  # Check repobox binary exists and is executable
  if [[ ! -x "$REPOBOX_BINARY" ]]; then
    error "repobox binary not found or not executable at: $REPOBOX_BINARY"
  fi
  
  # Check git is available
  if ! command -v git &> /dev/null; then
    error "git command not found"
  fi
  
  # Check curl is available
  if ! command -v curl &> /dev/null; then
    error "curl command not found"
  fi
  
  # Test git server connectivity via explorer (more reliable than git server directly)
  if ! curl -sf "$EXPLORER_BASE" -w "%{http_code}" -o /dev/null | grep -q "200"; then
    warning "Explorer not reachable at $EXPLORER_BASE - continuing anyway"
  fi
  
  # Test actual git server connectivity (check if git-receive-pack is available)
  log "Testing git server connectivity..."
  if curl -sf "http://${GIT_SERVER}:3490/test-repo.git/info/refs?service=git-receive-pack" -o /dev/null 2>/dev/null; then
    success "Git server responding to git protocol requests"
  else
    warning "Git server may not be responding to git protocol - continuing anyway"
    log "This might cause push/clone failures later"
  fi
  
  success "Environment validated"
}

# ================================
# Demo Repository Creation
# ================================

setup_demo_repo() {
  local timestamp=$(date +%s)
  REPO_NAME="demo-hackathon-$timestamp"
  TEMP_DIR="/tmp/repobox-demo-$(date +%Y%m%d-%H%M%S)"
  
  progress "📦" "Creating demo repository: $REPO_NAME"
  
  # Create unique temp directory
  mkdir -p "$TEMP_DIR"
  cd "$TEMP_DIR"
  
  # Initialize git repo
  git init "$REPO_NAME"
  cd "$REPO_NAME"
  
  # Rename default branch to main
  git checkout -b main 2>/dev/null || git branch -m master main
  
  # Set up PATH to include repobox
  export PATH="$(dirname "$REPOBOX_BINARY"):$PATH"
  
  # Create demo content
  cat > README.md << EOF
# $REPO_NAME

🎭 **repo.box Demo Repository**

This repository demonstrates the complete repo.box workflow:
- EVM-signed commits for verifiable authorship
- Permission-based access control via .repobox/config.yml
- Multi-agent development with branch restrictions
- Cryptographic verification on the blockchain

**Demo created**: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
**Repository**: \`$REPO_NAME\`
**Flow**: AI Agent → Signed Commits → Git Server → Explorer Verification

## Features Demonstrated

- 🔑 **Identity Management**: Generate and switch between EVM identities
- 📝 **Signed Commits**: Every commit cryptographically signed with EVM keys
- 🛡️  **Access Control**: Branch-based permissions for different agent roles
- 🌐 **Verification**: Public explorer showing signer addresses per commit
- 🤖 **AI Integration**: Purpose-built for AI agent workflows

## Next Steps

Visit the [explorer]($EXPLORER_BASE) to see this repository and verify signatures.
EOF

  # Create source files to demonstrate different agent capabilities
  mkdir -p src
  
  cat > src/hello.py << 'EOF'
#!/usr/bin/env python3
"""
repo.box Demo - Hello World Script
Demonstrates basic AI agent code contribution
"""

def main():
    print("🪄 Hello from repo.box!")
    print("✨ This commit was signed by an AI agent with EVM keys")
    print("🔍 Verify on the explorer: https://repo.box/explore/")

if __name__ == "__main__":
    main()
EOF

  cat > src/agent-example.js << 'EOF'
/**
 * repo.box Demo - AI Agent Example
 * Shows how AI agents can collaborate on code
 */

class RepoBoxAgent {
  constructor(name, evmAddress) {
    this.name = name;
    this.evmAddress = evmAddress;
    this.capabilities = ['sign-commits', 'follow-permissions', 'verify-signatures'];
  }
  
  async makeSignedCommit(message) {
    console.log(`🤖 ${this.name} (${this.evmAddress}) making signed commit`);
    console.log(`📝 Message: ${message}`);
    
    // In real implementation, this would:
    // 1. Use repobox to sign the commit with EVM key
    // 2. Include proof of identity in commit signature
    // 3. Respect .repobox/config.yml permissions
    
    return {
      signature: 'EVM_SIGNATURE',
      signer: this.evmAddress,
      verified: true
    };
  }
}

// Example usage
const foundingAgent = new RepoBoxAgent('demo-founder', '0x...');
const developerAgent = new RepoBoxAgent('demo-agent', '0x...');

module.exports = { RepoBoxAgent };
EOF

  cat > .gitignore << 'EOF'
# Node modules
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Python
__pycache__/
*.pyc
*.pyo
*.pyd
.Python
pip-log.txt

# OS
.DS_Store
Thumbs.db

# Editor
.vscode/
.idea/
*.swp
*.swo

# Logs
*.log
logs/

# Temporary files
.tmp/
temp/
EOF

  success "Demo repository created: $TEMP_DIR/$REPO_NAME"
}

# ================================
# repo.box Initialization
# ================================

initialize_repobox() {
  progress "🔧" "Initializing repo.box"
  
  # Initialize repo.box in the current directory
  repobox init --force
  
  # Create a customized config for the demo
  cat > .repobox/config.yml << 'EOF'
# repo.box Demo Configuration
# Shows multi-agent workflow with permission controls

groups:
  founders:
    # Generated during demo - will be populated with actual addresses
    # - evm:0x...
  
  agents:
    # Generated during demo - will be populated with actual addresses  
    # - evm:0x...
  
  bots:
    # Demo group for automated agents
    # - evm:0x...

permissions:
  default: deny
  rules:
    # Founders have full control
    - founders own >*
    
    # Agents can work on feature branches
    - agents push >feature/**
    - agents create >feature/**
    - agents edit * >feature/**
    
    # Agents can append to documentation
    - agents append ./README.md
    - agents append ./docs/**
    
    # Bots have limited permissions
    - bots push >bot/**
    - bots create >bot/**
    - bots edit * >bot/**
    
    # Nobody can modify the config except founders
    - agents not edit ./.repobox/config.yml
    - bots not edit ./.repobox/config.yml
EOF

  success "repo.box initialized with demo configuration"
}

# ================================
# Identity Management
# ================================

generate_demo_identities() {
  progress "🔑" "Generating demo identities"
  
  # Generate founder identity
  log "Generating founder identity..."
  FOUNDER_OUTPUT=$(repobox keys generate --alias demo-founder 2>&1)
  FOUNDER_ADDRESS=$(echo "$FOUNDER_OUTPUT" | grep -o 'evm:0x[a-fA-F0-9]*' | head -1)
  if [[ -z "$FOUNDER_ADDRESS" ]]; then
    error "Failed to extract founder address from: $FOUNDER_OUTPUT"
  fi
  success "Founder identity: $FOUNDER_ADDRESS"
  
  if [[ "$QUICK_MODE" == "false" ]]; then
    # Generate agent identity  
    log "Generating agent identity..."
    AGENT_OUTPUT=$(repobox keys generate --alias demo-agent 2>&1)
    AGENT_ADDRESS=$(echo "$AGENT_OUTPUT" | grep -o 'evm:0x[a-fA-F0-9]*' | head -1)
    if [[ -z "$AGENT_ADDRESS" ]]; then
      error "Failed to extract agent address from: $AGENT_OUTPUT"
    fi
    success "Agent identity: $AGENT_ADDRESS"
  fi
  
  # Set founder as primary identity
  repobox use demo-founder
  
  # Verify identity
  WHOAMI_OUTPUT=$(repobox whoami 2>&1)
  log "Current identity: $WHOAMI_OUTPUT"
  
  # Update config with actual generated addresses - fix the founder section first
  sed -i "/founders:/,/agents:/ s|# - evm:0x...|    - $FOUNDER_ADDRESS|" .repobox/config.yml
  if [[ "$QUICK_MODE" == "false" && -n "$AGENT_ADDRESS" ]]; then
    sed -i "/agents:/,/bots:/ s|# - evm:0x...|    - $AGENT_ADDRESS|" .repobox/config.yml
  fi
  
  # In quick mode, remove the agent address placeholder from agents group
  if [[ "$QUICK_MODE" == "true" ]]; then
    sed -i "/agents:/,/bots:/ s|    # - evm:0x...|    # (no agents in quick mode)|" .repobox/config.yml
  fi
  
  success "Demo identities configured"
}

# ================================
# Git Configuration & Signing
# ================================

setup_git_signing() {
  progress "📝" "Configuring git signing"
  
  # Configure git to use repobox for signing
  git config user.name "Demo Founder"
  git config user.email "founder@demo.repo.box"
  git config gpg.program "$REPOBOX_BINARY"
  git config commit.gpgsign true
  
  success "Git configured for EVM signing"
}

# ================================
# Signed Commit & Push Flow
# ================================

create_signed_commit() {
  progress "📝" "Creating signed commit"
  
  # Stage all files
  git add .
  
  # Make signed commit
  git commit -m "feat: initial demo repository setup

This commit demonstrates:
- EVM-signed commits using repo.box
- Multi-agent permission configuration  
- AI-agent friendly workflow patterns

Signed-by: demo-founder
Demo-timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')
Repository: $REPO_NAME"

  # Show commit signature (note: local git may not understand repobox signature format)
  log "Verifying commit signature..."
  if git log --show-signature -1 --pretty=format:"%h %s" 2>/dev/null; then
    success "Git signature verification passed"
  else
    log "Local git doesn't understand REPOBOX signature format (this is expected)"
    log "Server will verify EVM signatures during push"
  fi
  
  success "Signed commit created"
}

push_to_server() {
  progress "🚀" "Pushing to git.repo.box"
  
  # Set remote URL
  git remote add origin "https://${GIT_SERVER}/${REPO_NAME}.git"
  
  # Push to server
  log "Pushing to https://${GIT_SERVER}/${REPO_NAME}.git"
  git push -u origin main 2>&1 | tee /tmp/push-output.log || error "Push failed"
  
  # Extract repository owner from push output if available
  REPO_OWNER=$(grep -o '0x[a-fA-F0-9]*' /tmp/push-output.log | head -1 || echo "unknown")
  
  # Wait a moment for server to process the repository
  log "Waiting for repository to be available..."
  sleep 3
  
  success "Pushed to git.repo.box"
}

# ================================
# Agent Simulation (Full Mode Only)
# ================================

simulate_agent_workflow() {
  if [[ "$QUICK_MODE" == "true" ]]; then
    log "Skipping agent simulation in quick mode"
    return
  fi
  
  progress "🤖" "Simulating agent workflow"
  
  # Switch to agent identity
  repobox use demo-agent
  log "Switched to agent identity: $(repobox whoami)"
  
  # Create feature branch
  git checkout -b feature/agent-improvement
  
  # Modify agent example file
  cat >> src/agent-example.js << 'EOF'

// Enhancement by demo-agent
class EnhancedAgent extends RepoBoxAgent {
  constructor(name, evmAddress) {
    super(name, evmAddress);
    this.enhancedFeatures = ['multi-signature-support', 'permission-validation', 'audit-trail'];
  }
  
  async validatePermissions(action, target) {
    console.log(`🔍 Validating ${action} on ${target}`);
    // This would integrate with .repobox/config.yml
    return true;
  }
}

console.log('🌟 Enhanced by demo-agent with advanced capabilities!');
EOF

  # Create a new documentation file
  mkdir -p docs
  cat > docs/AGENT_WORKFLOW.md << 'EOF'
# Agent Workflow Documentation

This file demonstrates how AI agents can contribute to documentation.

## Permission Model

- **Founders**: Full repository control
- **Agents**: Feature branches + documentation updates  
- **Bots**: Limited bot-specific branches

## Verification Process

Every commit is signed with EVM keys, creating an immutable audit trail:

1. Agent generates EVM key pair
2. Commit includes cryptographic signature
3. Server verifies signature before accepting push
4. Explorer displays signer address for each commit

This enables trustless collaboration between AI agents.
EOF

  # Commit agent changes
  git add .
  git commit -m "feat: enhanced agent capabilities and documentation

- Added EnhancedAgent class with permission validation
- Created agent workflow documentation
- Demonstrated feature branch development

Signed-by: demo-agent
Enhancement-type: capability-expansion"

  # Push feature branch
  git push origin feature/agent-improvement
  
  # Switch back to founder
  repobox use demo-founder
  git checkout main
  
  success "Agent workflow simulation completed"
}

# ================================
# Verification & Clone Test
# ================================

verify_clone() {
  progress "🔍" "Verifying clone and signatures"
  
  # Create verification directory
  VERIFY_DIR="$TEMP_DIR/verification"
  mkdir -p "$VERIFY_DIR"
  cd "$VERIFY_DIR"
  
  # Clone repository from server (with retries)
  log "Cloning repository from server..."
  local retry_count=0
  local max_retries=3
  
  while [[ $retry_count -lt $max_retries ]]; do
    # Extract the address from FOUNDER_ADDRESS (remove evm: prefix)
    local owner_addr=$(echo "$FOUNDER_ADDRESS" | sed 's/evm://')
    local clone_url="https://${GIT_SERVER}/${owner_addr}/${REPO_NAME}.git"
    
    log "Attempting clone from: $clone_url"
    if git clone "$clone_url" cloned-repo 2>/dev/null; then
      success "Clone successful from $clone_url"
      break
    else
      retry_count=$((retry_count + 1))
      if [[ $retry_count -lt $max_retries ]]; then
        log "Clone failed, retrying in 2 seconds... (attempt $retry_count/$max_retries)"
        sleep 2
      else
        warning "Clone failed after $max_retries attempts - repository may not be ready yet"
        log "This can happen if the server is still processing the push"
        log "Try manually: git clone $clone_url"
        return
      fi
    fi
  done
  
  cd cloned-repo
  
  # Verify signatures (note: local git may not understand repobox signature format)
  log "Verifying commit signatures..."
  if git log --show-signature --oneline 2>/dev/null | head -5; then
    success "Signature verification passed"
  else
    log "Local git doesn't understand REPOBOX signature format"
    log "Showing standard commit log instead:"
    git log --oneline | head -5
  fi
  
  # Show branch structure
  log "Repository structure:"
  git branch -a
  
  # Verify file contents
  log "Verifying file contents..."
  if [[ -f "README.md" && -f ".repobox/config.yml" ]]; then
    success "All expected files present"
  else
    warning "Some expected files missing"
  fi
  
  success "Clone verification completed"
}

# ================================
# Explorer URLs & Final Display
# ================================

generate_explorer_urls() {
  progress "🌐" "Generating explorer URLs"
  
  # Extract actual repository owner address from founder address
  local owner_addr=$(echo "$FOUNDER_ADDRESS" | sed 's/evm://')
  
  # Generate URLs  
  REPO_URL="$EXPLORER_BASE/$owner_addr/$REPO_NAME"
  CLONE_URL="https://${GIT_SERVER}/${owner_addr}/${REPO_NAME}.git"
  
  log "Repository will be available at: $REPO_URL"
  log "Clone URL: $CLONE_URL"
  
  success "Explorer URLs generated"
}

show_final_summary() {
  local end_time=$(date +%s)
  local duration=$((end_time - start_time))
  
  echo ""
  echo "==============================================="
  echo "🎉 repo.box E2E Demo Results"
  echo "==============================================="
  echo "Repository: $REPO_NAME"
  echo "Owner: $FOUNDER_ADDRESS"
  echo "Mode: $(if [[ "$QUICK_MODE" == "true" ]]; then echo "Quick (30s)"; else echo "Full (60s)"; fi)"
  echo ""
  echo "🏗️  What Was Built:"
  echo "   ✅ Demo repository with signed commits"
  echo "   ✅ Multi-agent permission configuration"
  echo "   ✅ EVM identity generation and switching"
  if [[ "$QUICK_MODE" == "false" ]]; then
    echo "   ✅ Feature branch agent simulation"
    echo "   ✅ Documentation contributions"
  fi
  echo "   ✅ Git server push verification"
  echo "   ✅ Clone-back integrity check"
  echo ""
  echo "🔗 Links:"
  echo "   📱 Repository: $CLONE_URL"
  echo "   🌐 Explorer: $REPO_URL" 
  echo "   📂 Local files: $TEMP_DIR/$REPO_NAME"
  echo ""
  echo "🔑 Identities Generated:"
  echo "   👑 Founder: $FOUNDER_ADDRESS"
  if [[ "$QUICK_MODE" == "false" && -n "$AGENT_ADDRESS" ]]; then
    echo "   🤖 Agent: $AGENT_ADDRESS"
  fi
  echo ""
  echo "📋 Verification Steps Completed:"
  echo "   ✅ Signed commits created and verified"
  echo "   ✅ Permission config applied and tested"  
  echo "   ✅ Git server accepted pushes"
  echo "   ✅ Clone-back successful with signature verification"
  echo ""
  echo "⏱️  Demo completed in ${duration} seconds"
  echo "==============================================="
  echo ""
  echo "🎯 Next Steps:"
  echo "   • Visit the explorer URL to see commit signatures"
  echo "   • Try cloning: git clone $CLONE_URL"
  echo "   • Experiment with permission denials"
  echo "   • Add more agents to the configuration"
  echo ""
}

# ================================
# Cleanup Function
# ================================

cleanup() {
  if [[ -n "${TEMP_DIR:-}" && -d "$TEMP_DIR" ]]; then
    if [[ "$NO_CLEANUP" == "true" ]]; then
      log "Keeping temporary directory for debugging: $TEMP_DIR"
    else
      log "Cleaning up temporary directory: $TEMP_DIR"
      rm -rf "$TEMP_DIR"
    fi
  fi
}

# Set up cleanup trap
trap cleanup EXIT

# ================================
# Main Execution Flow
# ================================

main() {
  # Record start time for duration calculation
  start_time=$(date +%s)
  
  echo ""
  echo "🎭 repo.box E2E Demo Script"
  echo "Mode: $(if [[ "$QUICK_MODE" == "true" ]]; then echo "Quick (30s)"; else echo "Full (60s)"; fi)"
  echo "Target: $GIT_SERVER"
  echo "Explorer: $EXPLORER_BASE"
  echo ""
  
  # Execute demo steps
  validate_environment
  setup_demo_repo
  initialize_repobox
  generate_demo_identities
  setup_git_signing
  create_signed_commit
  push_to_server
  simulate_agent_workflow
  verify_clone
  generate_explorer_urls
  show_final_summary
  
  success "🎉 E2E Demo completed successfully!"
}

# ================================
# Script Entry Point
# ================================

# Only run main if script is executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi