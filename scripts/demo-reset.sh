#!/bin/bash
set -euo pipefail

# repo.box Demo Reset Script
# Removes demo repositories and cleans up temporary files
#
# Usage:
#   ./scripts/demo-reset.sh [options]
#
# Options:
#   --all       Remove all demo-* repositories
#   --pattern   Remove repositories matching pattern (e.g., demo-hackathon-*)
#   --dry-run   Show what would be deleted without actually deleting

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOBOX_DATA_DIR="${REPOBOX_DATA_DIR:-/tmp/repobox-data}"
DRY_RUN=false
PATTERN=""
ALL_DEMOS=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
  echo -e "${BLUE}ℹ️  $1${NC}"
}

success() {
  echo -e "${GREEN}✅ $1${NC}"
}

warning() {
  echo -e "${YELLOW}⚠️  $1${NC}"
}

error() {
  echo -e "${RED}❌ $1${NC}" >&2
  exit 1
}

print_usage() {
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  --all              Remove all demo-* repositories"
  echo "  --pattern PATTERN  Remove repositories matching pattern"
  echo "  --dry-run          Show what would be deleted without deleting"
  echo "  --help            Show this help message"
  echo ""
  echo "Examples:"
  echo "  $0 --all                           # Remove all demo repos"
  echo "  $0 --pattern 'demo-hackathon-*'    # Remove specific pattern"
  echo "  $0 --dry-run --all                 # See what would be removed"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      ALL_DEMOS=true
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
  echo "Error: Must specify --all or --pattern"
  print_usage
  exit 1
fi

log "repo.box Demo Reset Script"
log "Data directory: $REPOBOX_DATA_DIR"
log "Pattern: $PATTERN"

if [[ "$DRY_RUN" == "true" ]]; then
  warning "DRY RUN MODE - No files will be deleted"
fi

# Find repositories to remove
declare -a repos_to_remove=()

if [[ -d "$REPOBOX_DATA_DIR" ]]; then
  log "Scanning for repositories matching pattern: $PATTERN"
  
  # Find all address directories
  for addr_dir in "$REPOBOX_DATA_DIR"/*; do
    if [[ -d "$addr_dir" && ! "$(basename "$addr_dir")" =~ ^(_staging|\..*) ]]; then
      addr_name=$(basename "$addr_dir")
      log "Checking address directory: $addr_name"
      
      # Check each repo in this address directory
      for repo_dir in "$addr_dir"/*.git; do
        if [[ -d "$repo_dir" ]]; then
          repo_name=$(basename "$repo_dir" .git)
          
          # Check if repo matches pattern
          if [[ "$repo_name" == $PATTERN ]]; then
            repos_to_remove+=("$addr_name/$repo_name")
            log "Found matching repository: $addr_name/$repo_name"
          fi
        fi
      done
    fi
  done
  
  # Also check staging area
  if [[ -d "$REPOBOX_DATA_DIR/_staging" ]]; then
    for staging_repo in "$REPOBOX_DATA_DIR/_staging"/*.git; do
      if [[ -d "$staging_repo" ]]; then
        repo_name=$(basename "$staging_repo" .git)
        if [[ "$repo_name" == $PATTERN ]]; then
          repos_to_remove+=("_staging/$repo_name")
          log "Found matching staging repository: _staging/$repo_name"
        fi
      fi
    done
  fi
else
  warning "Data directory does not exist: $REPOBOX_DATA_DIR"
fi

# Remove temporary directories
log "Scanning for temporary demo directories..."
declare -a temp_dirs=()

for temp_dir in /tmp/repobox-demo-*; do
  if [[ -d "$temp_dir" ]]; then
    temp_dirs+=("$temp_dir")
    log "Found temporary directory: $temp_dir"
  fi
done

# Summary
if [[ ${#repos_to_remove[@]} -eq 0 && ${#temp_dirs[@]} -eq 0 ]]; then
  success "No matching repositories or temporary directories found"
  exit 0
fi

echo ""
log "Summary of items to remove:"

if [[ ${#repos_to_remove[@]} -gt 0 ]]; then
  echo "Repositories:"
  for repo in "${repos_to_remove[@]}"; do
    echo "  - $repo"
  done
fi

if [[ ${#temp_dirs[@]} -gt 0 ]]; then
  echo "Temporary directories:"
  for dir in "${temp_dirs[@]}"; do
    echo "  - $dir"
  done
fi

if [[ "$DRY_RUN" == "true" ]]; then
  echo ""
  warning "DRY RUN: Would remove ${#repos_to_remove[@]} repositories and ${#temp_dirs[@]} temp directories"
  exit 0
fi

echo ""
read -p "Are you sure you want to remove these items? (y/N): " -r
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  log "Aborted by user"
  exit 0
fi

# Perform cleanup
removed_count=0

# Remove repositories
for repo in "${repos_to_remove[@]}"; do
  if [[ "$repo" == _staging/* ]]; then
    repo_path="$REPOBOX_DATA_DIR/$repo.git"
  else
    repo_path="$REPOBOX_DATA_DIR/$repo.git"
  fi
  
  if [[ -d "$repo_path" ]]; then
    log "Removing repository: $repo"
    rm -rf "$repo_path"
    ((removed_count++))
  fi
done

# Remove temporary directories
for dir in "${temp_dirs[@]}"; do
  if [[ -d "$dir" ]]; then
    log "Removing temporary directory: $dir"
    rm -rf "$dir"
    ((removed_count++))
  fi
done

# Clean up empty address directories
if [[ -d "$REPOBOX_DATA_DIR" ]]; then
  for addr_dir in "$REPOBOX_DATA_DIR"/*; do
    if [[ -d "$addr_dir" && ! "$(basename "$addr_dir")" =~ ^(_staging|\..*) ]]; then
      if [[ -z "$(ls -A "$addr_dir" 2>/dev/null || true)" ]]; then
        log "Removing empty address directory: $(basename "$addr_dir")"
        rmdir "$addr_dir"
      fi
    fi
  done
fi

success "Cleanup completed! Removed $removed_count items"

echo ""
log "🎯 Next Steps:"
echo "   • Run the demo again: ./scripts/demo-e2e.sh"
echo "   • Check remaining repos: ls -la $REPOBOX_DATA_DIR"