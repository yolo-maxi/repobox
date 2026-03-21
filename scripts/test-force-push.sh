#!/bin/bash
# Comprehensive test for force push handling in repo.box

set -e

REPOBOX_CHECK="/home/xiko/repobox/target/release/repobox-check"
TEST_DIR="/tmp/force-push-test-$(date +%s)"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Creating test environment...${NC}"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Test 1: Force push detection
echo -e "\n${YELLOW}Test 1: Force push detection${NC}"
mkdir test-repo && cd test-repo && git init -q
echo "initial" > file.txt
git add . && git commit -q -m "initial"
echo "second" > file.txt
git add . && git commit -q -m "second"
COMMIT2=$(git rev-parse HEAD)
git reset --hard HEAD~1 -q
echo "divergent" > file.txt
git add . && git commit -q -m "divergent"
COMMIT1=$(git rev-parse HEAD~1)
COMMIT3=$(git rev-parse HEAD)

# This should detect force push
result=$(echo "$COMMIT2 $COMMIT3 refs/heads/main" | "$REPOBOX_CHECK" 2>&1)
if [[ $result == *"Force push detected"* ]]; then
    echo -e "${GREEN}✓ Force push detection works${NC}"
else
    echo -e "${RED}✗ Force push detection failed${NC}"
    echo "Output: $result"
fi

# Test 2: Fast-forward detection (should not be force push)
echo -e "\n${YELLOW}Test 2: Fast-forward detection${NC}"
echo "third" >> file.txt
git add . && git commit -q -m "third"
COMMIT4=$(git rev-parse HEAD)

result=$(echo "$COMMIT3 $COMMIT4 refs/heads/main" | "$REPOBOX_CHECK" 2>&1)
if [[ -z "$result" ]]; then
    echo -e "${GREEN}✓ Fast-forward correctly ignored${NC}"
else
    echo -e "${RED}✗ Fast-forward incorrectly flagged${NC}"
    echo "Output: $result"
fi

# Test 3: New branch creation (should not be force push)
echo -e "\n${YELLOW}Test 3: New branch creation${NC}"
result=$(echo "0000000000000000000000000000000000000000 $COMMIT4 refs/heads/feature" | "$REPOBOX_CHECK" 2>&1)
if [[ -z "$result" ]]; then
    echo -e "${GREEN}✓ New branch creation correctly ignored${NC}"
else
    echo -e "${RED}✗ New branch creation incorrectly flagged${NC}"
    echo "Output: $result"
fi

# Test 4: Branch deletion (should not be force push)
echo -e "\n${YELLOW}Test 4: Branch deletion${NC}"
result=$(echo "$COMMIT4 0000000000000000000000000000000000000000 refs/heads/feature" | "$REPOBOX_CHECK" 2>&1)
if [[ -z "$result" ]]; then
    echo -e "${GREEN}✓ Branch deletion correctly ignored${NC}"
else
    echo -e "${RED}✗ Branch deletion incorrectly flagged${NC}"
    echo "Output: $result"
fi

# Test 5: Tag updates (should not be force push)
echo -e "\n${YELLOW}Test 5: Tag updates${NC}"
result=$(echo "$COMMIT3 $COMMIT4 refs/tags/v1.0" | "$REPOBOX_CHECK" 2>&1)
if [[ -z "$result" ]]; then
    echo -e "${GREEN}✓ Tag updates correctly ignored${NC}"
else
    echo -e "${RED}✗ Tag updates incorrectly flagged${NC}"
    echo "Output: $result"
fi

# Test 6: Permission system without config (should allow)
echo -e "\n${YELLOW}Test 6: No config file (should allow)${NC}"
export PUSHER="evm:0x1234567890123456789012345678901234567890"
result=$(echo "$COMMIT2 $COMMIT3 refs/heads/main" | "$REPOBOX_CHECK" 2>&1)
if [[ $result == *"Force push authorized"* ]]; then
    echo -e "${GREEN}✓ No config allows force push (fail-open)${NC}"
else
    echo -e "${RED}✗ No config blocks force push${NC}"
    echo "Output: $result"
fi

# Test 7: Permission system with config
echo -e "\n${YELLOW}Test 7: Config-based permissions${NC}"
mkdir .repobox
cat > .repobox/config.yml << 'EOF'
groups:
  founders:
    - evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00
  agents:
    - evm:0x1234567890123456789012345678901234567890

permissions:
  default: allow
  rules:
    - founders force-push >*
    - agents force-push >feature/**
    - agents not force-push >main
EOF

# Test 7a: Founder can force push to main
export PUSHER="evm:0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00"
result=$(echo "$COMMIT2 $COMMIT3 refs/heads/main" | "$REPOBOX_CHECK" 2>&1)
if [[ $result == *"Force push authorized"* ]]; then
    echo -e "${GREEN}✓ Founder can force push to main${NC}"
else
    echo -e "${RED}✗ Founder blocked from main${NC}"
    echo "Output: $result"
fi

# Test 7b: Agent cannot force push to main
export PUSHER="evm:0x1234567890123456789012345678901234567890"
set +e  # Allow command to fail
result=$(echo "$COMMIT2 $COMMIT3 refs/heads/main" | "$REPOBOX_CHECK" 2>&1)
exit_code=$?
set -e  # Resume strict mode
if [[ $exit_code -eq 1 && $result == *"Force push denied"* ]]; then
    echo -e "${GREEN}✓ Agent correctly denied force push to main${NC}"
else
    echo -e "${RED}✗ Agent should be denied force push to main${NC}"
    echo "Exit code: $exit_code"
    echo "Output: $result"
fi

# Test 7c: Agent can force push to feature branch
result=$(echo "$COMMIT2 $COMMIT3 refs/heads/feature/test" | "$REPOBOX_CHECK" 2>&1)
if [[ $result == *"Force push authorized"* ]]; then
    echo -e "${GREEN}✓ Agent can force push to feature branch${NC}"
else
    echo -e "${RED}✗ Agent blocked from feature branch${NC}"
    echo "Output: $result"
fi

echo -e "\n${GREEN}All tests completed!${NC}"

# Cleanup
cd /
rm -rf "$TEST_DIR"