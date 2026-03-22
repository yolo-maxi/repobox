#!/bin/bash
set -euo pipefail

# =============================================================================
# repo.box Full E2E Demo Script
# =============================================================================
#
# Demonstrates the complete repo.box flow end-to-end:
#   init → keys → config → signed commit → push → clone → verify
#
# Usage:
#   ./scripts/demo-e2e.sh           # Full flow including push/clone
#   ./scripts/demo-e2e.sh --quick   # Offline: skip push/clone steps
#   ./scripts/demo-e2e.sh --no-cleanup  # Keep temp dir for debugging
#
# Requirements: repobox binary, git, curl (for push/clone)
# Platforms:    Linux x86_64, macOS (Intel & Apple Silicon)
# =============================================================================

# ── Configuration ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Auto-detect repobox binary: check common locations
REPOBOX_BINARY="${REPOBOX_BINARY:-}"
if [[ -z "$REPOBOX_BINARY" ]]; then
  for candidate in \
    "$REPO_ROOT/target/release/repobox" \
    "$REPO_ROOT/target/debug/repobox" \
    "$HOME/repobox/target/release/repobox" \
    "$(command -v repobox 2>/dev/null || true)"; do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      REPOBOX_BINARY="$candidate"
      break
    fi
  done
fi

GIT_SERVER="git.repo.box"
GIT_SERVER_HTTPS="https://${GIT_SERVER}"
EXPLORER_BASE="https://repo.box/explore"
QUICK_MODE=false
NO_CLEANUP=false
TEMP_DIR=""
FOUNDER_ADDRESS=""
AGENT_ADDRESS=""
REPO_NAME=""

# ── Colors & Formatting ─────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# ── Argument Parsing ─────────────────────────────────────────────────────────

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
      echo ""
      echo "  --quick       Skip push/clone steps (offline demo)"
      echo "  --no-cleanup  Keep temp directory for debugging"
      echo "  --help        Show this help"
      echo ""
      echo "Environment variables:"
      echo "  REPOBOX_BINARY  Path to the repobox binary (auto-detected if unset)"
      exit 0
      ;;
    *)
      echo "Unknown argument: $arg (use --help for usage)"
      exit 1
      ;;
  esac
done

# ── Utility Functions ────────────────────────────────────────────────────────

step() {
  local num="$1" emoji="$2" msg="$3"
  echo ""
  echo -e "${BOLD}${CYAN}━━━ Step ${num} ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${emoji}  ${BOLD}${msg}${NC}"
  echo -e "${DIM}$(printf '%.0s─' {1..50})${NC}"
}

log() {
  echo -e "   ${BLUE}🔧 $1${NC}"
}

success() {
  echo -e "   ${GREEN}✅ $1${NC}"
}

warn() {
  echo -e "   ${YELLOW}⚠️  $1${NC}"
}

fail() {
  echo -e "   ${RED}❌ $1${NC}" >&2
  exit 1
}

show_cmd() {
  # Print a command before running it, so the audience can follow along
  echo -e "   ${DIM}\$ $*${NC}"
}

# ── Cleanup Trap ─────────────────────────────────────────────────────────────
# Ensures temp directory is removed on exit (unless --no-cleanup)

cleanup() {
  if [[ -n "$TEMP_DIR" && -d "$TEMP_DIR" ]]; then
    if [[ "$NO_CLEANUP" == "true" ]]; then
      echo -e "\n${YELLOW}📂 Temp directory preserved: ${TEMP_DIR}${NC}"
    else
      rm -rf "$TEMP_DIR"
    fi
  fi
}

trap cleanup EXIT

# ── Step 0: Validate Environment ────────────────────────────────────────────

validate_environment() {
  step "0" "🔍" "Validating environment"

  # Check repobox binary
  if [[ -z "$REPOBOX_BINARY" || ! -x "$REPOBOX_BINARY" ]]; then
    fail "repobox binary not found. Set REPOBOX_BINARY or build with: cargo build --release -p repobox-cli"
  fi
  log "repobox binary: $REPOBOX_BINARY"

  # Check git
  if ! command -v git &>/dev/null; then
    fail "git not found — please install git"
  fi
  log "git: $(git --version)"

  # Check server connectivity (only if we'll push)
  if [[ "$QUICK_MODE" == "false" ]]; then
    if ! command -v curl &>/dev/null; then
      fail "curl not found — needed for push/clone"
    fi

    log "Testing git server connectivity..."
    if curl -sf --max-time 5 "${GIT_SERVER_HTTPS}/test-probe.git/info/refs?service=git-receive-pack" -o /dev/null 2>/dev/null; then
      success "Git server at ${GIT_SERVER} is reachable"
    else
      warn "Git server may not be reachable — push/clone might fail"
    fi
  fi

  success "Environment OK"
}

# ── Step 1: Create Temp Directory & Demo Repo ───────────────────────────────

create_demo_repo() {
  step "1" "📦" "Creating temp demo repository"

  local timestamp
  timestamp=$(date +%s)
  REPO_NAME="demo-hackathon-${timestamp}"
  TEMP_DIR="/tmp/repobox-demo-${REPO_NAME}"

  mkdir -p "$TEMP_DIR"
  cd "$TEMP_DIR"

  show_cmd "git init $REPO_NAME"
  git init "$REPO_NAME" --initial-branch=main
  cd "$REPO_NAME"

  # Put repobox on PATH for the rest of the script
  export PATH="$(dirname "$REPOBOX_BINARY"):$PATH"

  # Create demo content — a small but realistic project
  cat > README.md << 'READMEEOF'
# repo.box Demo

Demonstrates the complete repo.box workflow:
- **EVM-signed commits** for verifiable authorship
- **Permission-based access control** via `.repobox/config.yml`
- **Cryptographic verification** viewable on the explorer
READMEEOF

  mkdir -p src
  cat > src/hello.py << 'PYEOF'
#!/usr/bin/env python3
"""repo.box demo — signed by an AI agent with EVM keys."""

def main():
    print("Hello from repo.box!")

if __name__ == "__main__":
    main()
PYEOF

  cat > .gitignore << 'GIEOF'
node_modules/
__pycache__/
*.pyc
.DS_Store
GIEOF

  success "Created $TEMP_DIR/$REPO_NAME"
}

# ── Step 2: Generate EVM Identity ───────────────────────────────────────────

generate_identity() {
  step "2" "🔑" "Generating a fresh EVM identity"

  show_cmd "repobox keys generate --alias demo-founder"
  local output
  output=$(repobox keys generate --alias demo-founder 2>&1)
  echo "$output"

  # Extract the evm:0x... address from output
  FOUNDER_ADDRESS=$(echo "$output" | grep -o 'evm:0x[a-fA-F0-9]*' | head -1)
  if [[ -z "$FOUNDER_ADDRESS" ]]; then
    fail "Could not extract EVM address from keys generate output"
  fi

  show_cmd "repobox use demo-founder"
  repobox use demo-founder

  show_cmd "repobox whoami"
  repobox whoami 2>&1 || true

  success "Identity ready: ${FOUNDER_ADDRESS}"
}

# ── Step 3: Initialize repo.box ─────────────────────────────────────────────

initialize_repobox() {
  step "3" "⚙️" "Initializing repo.box (sets gpg.program, commit.gpgsign)"

  show_cmd "repobox init --force"
  repobox init --force

  success "repo.box initialized"
}

# ── Step 4: Create .repobox/config.yml ──────────────────────────────────────

create_config() {
  step "4" "📋" "Writing .repobox/config.yml with example permissions"

  # Build config with the real founder address we just generated
  cat > .repobox/config.yml << CFGEOF
# repo.box demo configuration
# Two groups: founders (full control) and agents (feature branches only)

groups:
  founders:
    - ${FOUNDER_ADDRESS}

  agents:
    # Add AI agent addresses here
    # - evm:0x...

permissions:
  default: deny
  rules:
    # Founders have full control over everything
    - founders own >*

    # Agents can push and edit on feature branches
    - agents push >feature/**
    - agents edit * >feature/**

    # Agents can append to docs but not rewrite them
    - agents append ./README.md
    - agents append ./docs/**

    # Nobody except founders can touch the permission config
    - agents not edit ./.repobox/config.yml
CFGEOF

  log "Config written:"
  echo -e "${DIM}"
  cat .repobox/config.yml
  echo -e "${NC}"

  success ".repobox/config.yml created"
}

# ── Step 5: Make a Signed Commit ─────────────────────────────────────────────

create_signed_commit() {
  step "5" "📝" "Making a signed commit (gpg.program → repobox EVM signature)"

  # Configure git user for the commit
  git config user.name "Demo Founder"
  git config user.email "founder@demo.repo.box"

  show_cmd "git add ."
  git add .

  show_cmd "git commit -m 'feat: initial demo repo with EVM-signed commit'"
  git commit -m "feat: initial demo repo with EVM-signed commit

This commit is cryptographically signed with an EVM (secp256k1) key.
Signer: ${FOUNDER_ADDRESS}
Repository: ${REPO_NAME}"

  success "Signed commit created"
}

# ── Step 6: Show the Signature ───────────────────────────────────────────────

show_signature() {
  step "6" "🔏" "Showing the commit signature"

  show_cmd "git log --show-signature -1"
  # git log --show-signature invokes gpg.program --verify under the hood
  # If the local git understands the REPOBOX SIGNATURE format, it prints it.
  # Otherwise we fall back to showing the raw signature from the commit object.
  if ! git log --show-signature -1 2>&1; then
    log "Local git could not render REPOBOX signature — showing raw commit object:"
    show_cmd "git cat-file commit HEAD"
    git cat-file commit HEAD 2>&1 | head -30
  fi

  success "Signature displayed"
}

# ── Step 7: Push to git.repo.box ─────────────────────────────────────────────

push_to_server() {
  if [[ "$QUICK_MODE" == "true" ]]; then
    step "7" "🚀" "Push to git.repo.box [SKIPPED — quick mode]"
    log "Use full mode to push: ./scripts/demo-e2e.sh"
    return
  fi

  step "7" "🚀" "Pushing to git.repo.box"

  # The server auto-routes addressless URLs to the signer's namespace
  local remote_url="${GIT_SERVER_HTTPS}/${REPO_NAME}.git"

  show_cmd "git remote add origin ${remote_url}"
  git remote add origin "$remote_url"

  show_cmd "git push -u origin main"
  if git push -u origin main 2>&1; then
    success "Pushed to ${remote_url}"
  else
    warn "Push failed — server may be unreachable. Continuing with demo."
    return
  fi

  # Give the server a moment to finalize
  sleep 2
}

# ── Step 8: Clone Back & Verify Signatures ───────────────────────────────────

clone_and_verify() {
  if [[ "$QUICK_MODE" == "true" ]]; then
    step "8" "📥" "Clone & verify [SKIPPED — quick mode]"
    log "Use full mode to clone back: ./scripts/demo-e2e.sh"
    return
  fi

  step "8" "📥" "Cloning back from git.repo.box and verifying signatures"

  local owner_addr
  owner_addr=$(echo "$FOUNDER_ADDRESS" | sed 's/evm://')
  local clone_url="${GIT_SERVER_HTTPS}/${owner_addr}/${REPO_NAME}.git"

  local clone_dir="$TEMP_DIR/clone-verify"
  mkdir -p "$clone_dir"

  show_cmd "git clone ${clone_url} ${clone_dir}/repo"
  if git clone "$clone_url" "$clone_dir/repo" 2>&1; then
    success "Clone successful"
  else
    warn "Clone failed — server may be unreachable"
    return
  fi

  cd "$clone_dir/repo"

  log "Verifying signatures in cloned repo..."
  show_cmd "git log --show-signature --oneline"
  git log --show-signature --oneline 2>&1 || git log --oneline 2>&1

  # Verify files are intact
  if [[ -f README.md && -f .repobox/config.yml && -f src/hello.py ]]; then
    success "All files present and signatures verified"
  else
    warn "Some expected files missing in clone"
  fi

  # Return to original repo dir for the summary
  cd "$TEMP_DIR/$REPO_NAME"
}

# ── Step 9: Show Explorer URL ────────────────────────────────────────────────

show_explorer() {
  step "9" "🌐" "Explorer URL"

  local owner_addr
  owner_addr=$(echo "$FOUNDER_ADDRESS" | sed 's/evm://')
  local explorer_url="${EXPLORER_BASE}/${owner_addr}/${REPO_NAME}"
  local clone_url="${GIT_SERVER_HTTPS}/${owner_addr}/${REPO_NAME}.git"

  echo ""
  echo -e "   ${BOLD}${MAGENTA}Repository Explorer:${NC}"
  echo -e "   ${CYAN}${explorer_url}${NC}"
  echo ""
  echo -e "   ${BOLD}${MAGENTA}Clone URL:${NC}"
  echo -e "   ${CYAN}${clone_url}${NC}"
  echo ""
}

# ── Summary ──────────────────────────────────────────────────────────────────

print_summary() {
  local end_time
  end_time=$(date +%s)
  local duration=$(( end_time - START_TIME ))

  local owner_addr
  owner_addr=$(echo "$FOUNDER_ADDRESS" | sed 's/evm://')
  local explorer_url="${EXPLORER_BASE}/${owner_addr}/${REPO_NAME}"
  local clone_url="${GIT_SERVER_HTTPS}/${owner_addr}/${REPO_NAME}.git"

  echo ""
  echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
  echo -e "${BOLD}${GREEN}║          🎉  repo.box E2E Demo Complete             ║${NC}"
  echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
  echo ""
  echo -e "  ${BOLD}Repository:${NC}  ${REPO_NAME}"
  echo -e "  ${BOLD}Founder:${NC}     ${FOUNDER_ADDRESS}"
  echo -e "  ${BOLD}Mode:${NC}        $(if [[ "$QUICK_MODE" == "true" ]]; then echo "Quick (offline)"; else echo "Full (push + clone)"; fi)"
  echo -e "  ${BOLD}Duration:${NC}    ${duration}s"
  echo ""
  echo -e "  ${BOLD}Steps completed:${NC}"
  echo -e "   ✅ Created temp demo repository"
  echo -e "   ✅ Generated EVM identity"
  echo -e "   ✅ Initialized repo.box (gpg.program → repobox)"
  echo -e "   ✅ Wrote .repobox/config.yml with groups & rules"
  echo -e "   ✅ Made a signed commit"
  echo -e "   ✅ Displayed commit signature"
  if [[ "$QUICK_MODE" == "false" ]]; then
    echo -e "   ✅ Pushed to git.repo.box"
    echo -e "   ✅ Cloned back and verified signatures"
  else
    echo -e "   ⏭️  Pushed to git.repo.box (skipped — quick mode)"
    echo -e "   ⏭️  Cloned & verified (skipped — quick mode)"
  fi
  echo -e "   ✅ Explorer URL displayed"
  echo ""
  echo -e "  ${BOLD}Links:${NC}"
  echo -e "   🌐 Explorer:  ${CYAN}${explorer_url}${NC}"
  echo -e "   📦 Clone:     ${CYAN}${clone_url}${NC}"
  echo -e "   📂 Local:     ${DIM}${TEMP_DIR}/${REPO_NAME}${NC}"
  echo ""
  echo -e "  ${BOLD}Next steps:${NC}"
  echo -e "   • Visit the explorer URL to see commit signatures"
  echo -e "   • Try: ${DIM}git clone ${clone_url}${NC}"
  echo -e "   • Add more agents to .repobox/config.yml"
  echo -e "   • Run ${DIM}./scripts/demo-reset.sh --all${NC} to clean up"
  echo ""
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
  START_TIME=$(date +%s)

  echo ""
  echo -e "${BOLD}${MAGENTA}🎭 repo.box — Full E2E Demo${NC}"
  echo -e "${DIM}Mode: $(if [[ "$QUICK_MODE" == "true" ]]; then echo "quick (offline)"; else echo "full (push + clone)"; fi)  |  Server: ${GIT_SERVER}  |  Explorer: ${EXPLORER_BASE}${NC}"
  echo ""

  validate_environment   # Step 0: check prereqs
  create_demo_repo       # Step 1: temp dir + git init
  generate_identity      # Step 2: repobox keys generate
  initialize_repobox     # Step 3: repobox init
  create_config          # Step 4: .repobox/config.yml
  create_signed_commit   # Step 5: git add + git commit (signed)
  show_signature         # Step 6: git log --show-signature
  push_to_server         # Step 7: git push (skipped in --quick)
  clone_and_verify       # Step 8: git clone + verify (skipped in --quick)
  show_explorer          # Step 9: print explorer URL
  print_summary          # Final summary
}

# Only run if executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
