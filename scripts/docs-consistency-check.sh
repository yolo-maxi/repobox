#!/bin/bash

# docs-consistency-check.sh
# Validates documentation consistency against Rust source of truth

set -e

echo "🔍 repo.box Documentation Consistency Check"
echo "============================================="

ERRORS=0
WARNINGS=0

# Check 1: Config file name consistency  
echo "📝 Checking config file name consistency..."

# Check docs and source files for old config references
BAD_REFS=$(find . -name "*.md" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.txt" | \
    grep -v ".git" | grep -v "FULL_KNOWLEDGE_SWEEP_SPEC.md" | \
    xargs grep -l "\.repobox-config" 2>/dev/null || true)

if [ -n "$BAD_REFS" ]; then
    echo "❌ Found references to old config file name '.repobox-config' in:"
    for file in $BAD_REFS; do
        echo "  $file"
    done
    ((ERRORS++))
else
    echo "✅ All config file references use '.repobox/config.yml'"
fi

# Check 2: Verb consistency against Rust source
echo "🔧 Checking verb consistency..."

# Extract verbs from Rust enum
if [ -f "repobox-core/src/config.rs" ]; then
    RUST_VERBS=$(grep -A 20 "pub enum Verb" repobox-core/src/config.rs | grep -E "^\s+(Read|Push|Merge|Branch|Create|Delete|ForcePush|Edit|Write|Append)," | sed 's/.*\(Read\|Push\|Merge\|Branch\|Create\|Delete\|ForcePush\|Edit\|Write\|Append\).*/\L\1/' | sed 's/forcepush/force-push/' | sort)
    
    echo "Rust source verbs: $(echo $RUST_VERBS | tr '\n' ' ')"
    
    # Check llms.txt has all verbs mentioned
    MISSING_VERBS=()
    for verb in $RUST_VERBS; do
        if ! grep -q "\b$verb\b" llms.txt; then
            MISSING_VERBS+=($verb)
        fi
    done
    
    if [ ${#MISSING_VERBS[@]} -eq 0 ]; then
        echo "✅ All Rust verbs documented in llms.txt"
    else
        echo "❌ Missing verbs in llms.txt: ${MISSING_VERBS[*]}"
        ((ERRORS++))
    fi
else
    echo "⚠️  Cannot find repobox-core/src/config.rs to check verb consistency"
    ((WARNINGS++))
fi

# Check 3: Example config file validity
echo "📋 Checking example configs..."

# Create temp dir for testing
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Test a simple config template for YAML validity
cat > "$TEMP_DIR/test-config.yml" << 'EOF'
groups:
  founders:
    - evm:0xAAA...123
  agents:
    - evm:0xBBB...456

permissions:
  default: allow
  rules:
    - founders own >*
    - agents push >feature/**
EOF

if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import yaml; yaml.safe_load(open('$TEMP_DIR/test-config.yml'))" 2>/dev/null; then
        echo "✅ Config template has valid YAML syntax"
    else
        echo "❌ Config template has invalid YAML syntax"
        ((ERRORS++))
    fi
else
    echo "⚠️  No python3 available for YAML validation"
    ((WARNINGS++))
fi

# Check 4: Verify install script exists
echo "📦 Checking install script..."

if [ -f "install.sh" ]; then
    if head -5 install.sh | grep -q "#!/bin/bash\|#!/bin/sh"; then
        echo "✅ install.sh exists and has proper shebang"
    else
        echo "❌ install.sh missing proper shebang"
        ((ERRORS++))
    fi
else
    echo "❌ install.sh not found"
    ((ERRORS++))
fi

# Check 5: Playground prompt verb consistency  
echo "🎮 Checking playground prompt..."

if [ -f "repobox-landing/src/lib/repobox-prompt.ts" ]; then
    if grep -q "read.*clone\|ACCESS VERBS.*read" repobox-landing/src/lib/repobox-prompt.ts; then
        echo "✅ Playground prompt includes 'read' verb"
    else
        echo "❌ Playground prompt missing 'read' verb documentation"
        ((ERRORS++))
    fi
else
    echo "⚠️  Playground prompt file not found"
    ((WARNINGS++))
fi

# Check 6: README verb table completeness
echo "📖 Checking README verb table..."

if [ -f "README.md" ]; then
    if grep -A 20 "### Verbs" README.md | grep -q "| \`read\`"; then
        echo "✅ README includes 'read' verb in table"
    else
        echo "❌ README missing 'read' verb in verb table"
        ((ERRORS++))
    fi
else
    echo "❌ README.md not found"
    ((ERRORS++))
fi

# Check 7: No "Coming Soon" for implemented features
echo "🚧 Checking for stale 'Coming Soon' content..."

if grep -r "Coming Soon\|TODO:\|FIXME:\|XXX:" . --exclude-dir=.git --exclude-dir=target --exclude="$0" | grep -v "# TODO" >/dev/null 2>&1; then
    echo "⚠️  Found potential stale development markers:"
    grep -r "Coming Soon\|TODO:\|FIXME:\|XXX:" . --exclude-dir=.git --exclude-dir=target --exclude="$0" | grep -v "# TODO" || true
    ((WARNINGS++))
else
    echo "✅ No stale development markers found"
fi

# Summary
echo ""
echo "📊 Summary:"
echo "Errors: $ERRORS"  
echo "Warnings: $WARNINGS"

if [ $ERRORS -eq 0 ]; then
    echo "✅ Documentation consistency check passed!"
    exit 0
else
    echo "❌ Documentation consistency check failed with $ERRORS errors"
    exit 1
fi