#!/bin/bash
set -euo pipefail

# =============================================================================
# repo.box Demo Reset Script
# =============================================================================
#
# Cleans up demo repositories from the server and local temp directories.
#
# Usage:
#   ./scripts/demo-reset.sh --all                # Remove all demo-* repos
#   ./scripts/demo-reset.sh --pattern 'demo-*'   # Remove matching pattern
#   ./scripts/demo-reset.sh --dry-run --all       # Preview what would be deleted
#
# =============================================================================

REPOBOX_DATA_DIR="${REPOBOX_DATA_DIR:-/tmp/repobox-data}"
DRY_RUN=false
PATTERN=""

# ── Colors ───────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()     { echo -e "  ${BLUE}ℹ️  $1${NC}"; }
success() { echo -e "  ${GREEN}✅ $1${NC}"; }
warn()    { echo -e "  ${YELLOW}⚠️  $1${NC}"; }
fail()    { echo -e "  ${RED}❌ $1${NC}" >&2; exit 1; }

print_usage() {
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  --all              Remove all demo-* repositories and temp dirs"
  echo "  --pattern PATTERN  Remove repos matching glob pattern (e.g. 'demo-hackathon-*')"
  echo "  --dry-run          Show what would be deleted without deleting"
  echo "  --help             Show this help"
  echo ""
  echo "Environment:"
  echo "  REPOBOX_DATA_DIR   Server data directory (default: /tmp/repobox-data)"
}

# ── Argument Parsing ─────────────────────────────────────────────────────────

while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      PATTERN="demo-*"
      shift
      ;;
    --pattern)
      PATTERN="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --help|-h)
      print_usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      print_usage
      exit 1
      ;;
  esac
done

if [[ -z "$PATTERN" ]]; then
  fail "Must specify --all or --pattern <glob>. Use --help for usage."
fi

echo ""
echo -e "🧹 ${BLUE}repo.box Demo Reset${NC}"
echo -e "   Pattern: ${PATTERN}"
echo -e "   Data dir: ${REPOBOX_DATA_DIR}"
if [[ "$DRY_RUN" == "true" ]]; then
  warn "DRY RUN — nothing will be deleted"
fi
echo ""

# ── Collect items to remove ──────────────────────────────────────────────────

declare -a items=()

# 1) Server-side bare repos: $REPOBOX_DATA_DIR/<address>/<repo>.git
if [[ -d "$REPOBOX_DATA_DIR" ]]; then
  for addr_dir in "$REPOBOX_DATA_DIR"/*/; do
    [[ -d "$addr_dir" ]] || continue
    for repo_dir in "$addr_dir"*.git; do
      [[ -d "$repo_dir" ]] || continue
      repo_name=$(basename "$repo_dir" .git)
      # shellcheck disable=SC2053
      if [[ "$repo_name" == $PATTERN ]]; then
        items+=("repo:$repo_dir")
        log "Server repo: $(basename "$addr_dir")/${repo_name}"
      fi
    done
  done

  # Also check staging area
  if [[ -d "$REPOBOX_DATA_DIR/_staging" ]]; then
    for repo_dir in "$REPOBOX_DATA_DIR/_staging"/*.git; do
      [[ -d "$repo_dir" ]] || continue
      repo_name=$(basename "$repo_dir" .git)
      # shellcheck disable=SC2053
      if [[ "$repo_name" == $PATTERN ]]; then
        items+=("repo:$repo_dir")
        log "Staging repo: _staging/${repo_name}"
      fi
    done
  fi
fi

# 2) Local temp directories: /tmp/repobox-demo-*
for tmp in /tmp/repobox-demo-*; do
  if [[ -d "$tmp" ]]; then
    items+=("tmp:$tmp")
    log "Temp dir: $tmp"
  fi
done

# ── Summary & Confirmation ───────────────────────────────────────────────────

if [[ ${#items[@]} -eq 0 ]]; then
  success "Nothing to clean up — no matching repos or temp dirs found"
  exit 0
fi

echo ""
log "Found ${#items[@]} item(s) to remove"

if [[ "$DRY_RUN" == "true" ]]; then
  warn "DRY RUN complete — ${#items[@]} item(s) would be removed"
  exit 0
fi

# Prompt for confirmation (skip if stdin is not a terminal)
if [[ -t 0 ]]; then
  read -rp "  Remove ${#items[@]} item(s)? (y/N): " confirm
  if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    log "Aborted"
    exit 0
  fi
fi

# ── Perform Cleanup ──────────────────────────────────────────────────────────

removed=0
for item in "${items[@]}"; do
  path="${item#*:}"
  if [[ -d "$path" ]]; then
    rm -rf "$path"
    (( removed++ ))
  fi
done

# Remove empty address directories
if [[ -d "$REPOBOX_DATA_DIR" ]]; then
  for addr_dir in "$REPOBOX_DATA_DIR"/*/; do
    [[ -d "$addr_dir" ]] || continue
    basename_dir=$(basename "$addr_dir")
    [[ "$basename_dir" == _staging ]] && continue
    if [[ -z "$(ls -A "$addr_dir" 2>/dev/null)" ]]; then
      rmdir "$addr_dir" 2>/dev/null || true
    fi
  done
fi

echo ""
success "Removed ${removed} item(s)"
echo ""
echo -e "  ${BLUE}Next:${NC} run the demo again with ./scripts/demo-e2e.sh"
