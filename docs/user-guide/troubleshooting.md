# Troubleshooting

Common issues with repo.box and their solutions.

## Permission Denied Errors

### "Permission denied: cannot push to main"

**Cause:** Your identity doesn't have push permission for the main branch.

**Solution:**
1. **Check your identity:**
   ```bash
   git repobox whoami
   # → alice (evm:0x7D5b...)
   ```

2. **Test the specific permission:**
   ```bash
   git repobox check alice push main
   # → ✗ DENY: No matching rule
   ```

3. **Review the configuration:**
   ```bash
   git repobox status
   # Shows your identity, groups, and applicable rules
   ```

4. **Update configuration** to grant permission:
   ```yaml
   permissions:
     rules:
       - alice push >main  # Add this rule
   ```

### "Permission denied: cannot edit .repobox/config.yml"

**Cause:** Agents typically shouldn't edit the permission configuration.

**Solution:**
- **If you're human:** Switch to a maintainer identity with config edit rights
- **If you're an agent:** This is expected behavior - agents shouldn't modify permissions
- **If needed:** Add explicit permission in config (carefully):
  ```yaml
  rules:
    - trusted-agent edit .repobox/config.yml
  ```

### "No matching rule (default: deny)"

**Cause:** The action isn't covered by any permission rule and default policy is deny.

**Solution:**
1. **Check what rules exist:**
   ```bash
   git repobox status
   ```

2. **Add missing permission:**
   ```yaml
   permissions:
     rules:
       - your-identity <verb> <target>
   ```

3. **Or change default policy** (less secure):
   ```yaml
   permissions:
     default: allow  # Be careful with this
   ```

## Configuration Errors

### "Config parse error: unknown group 'agents'"

**Cause:** Rule references a group that doesn't exist.

**Solution:**
1. **Check group definition:**
   ```yaml
   groups:
     agents:  # Make sure this group is defined
       - evm:0x...
   ```

2. **Fix typos in group names:**
   ```yaml
   # Wrong
   - agent push >feature/**  # Typo: "agent" vs "agents"
   
   # Correct  
   - agents push >feature/**
   ```

3. **Use linter to find all issues:**
   ```bash
   git repobox lint
   # → Error: Unknown group 'agent' in rule 'agent push >feature/**'
   ```

### "YAML parse error"

**Cause:** Invalid YAML syntax in `.repobox/config.yml`.

**Common issues:**
- **Inconsistent indentation:**
  ```yaml
  groups:
    agents:
      - evm:0x...
     agents2:  # Wrong indentation
  ```

- **Missing quotes for special characters:**
  ```yaml
  # Wrong
  - agents edit * >feature/**
  
  # Correct
  - "agents edit * >feature/**"
  ```

**Solution:** Use `git repobox lint` to get specific line numbers and errors.

### "Unreachable rule detected"

**Cause:** A rule can never be matched because an earlier rule shadows it.

**Example:**
```yaml
rules:
  - agents push >feature/**        # Matches first
  - agents not push >feature/secret  # Never reached!
```

**Solution:** Reorder rules from specific to general:
```yaml
rules:
  - agents not push >feature/secret  # Specific denial first
  - agents push >feature/**          # General allow second
```

## Identity Issues

### "Identity not found: alice"

**Cause:** The alias doesn't exist or isn't set.

**Solution:**
1. **List available identities:**
   ```bash
   git repobox keys list
   ```

2. **Create missing alias:**
   ```bash
   git repobox alias set alice evm:0x7D5b...
   ```

3. **Or set identity directly:**
   ```bash
   git repobox use evm:0x7D5b67EE304af5be0d9C9FeAB43973ba7d811661
   ```

### "Key file not found"

**Cause:** Private key file is missing from `~/.repobox/keys/`.

**Solution:**
1. **Check key files:**
   ```bash
   ls ~/.repobox/keys/
   ```

2. **Regenerate if missing:**
   ```bash
   git repobox keys generate --alias alice
   ```

3. **Or import existing key** (if you have it):
   ```bash
   # Copy your private key to ~/.repobox/keys/alice_private.pem
   ```

### "Invalid signature"

**Cause:** Git commit signature doesn't match the claimed identity.

**Solution:**
1. **Check git configuration:**
   ```bash
   git config user.signingkey
   git config gpg.program
   ```

2. **Reconfigure git for repo.box:**
   ```bash
   git config user.signingkey "0xYourAddress"
   git config gpg.program "repobox"
   git config commit.gpgsign true
   ```

3. **Verify signature works:**
   ```bash
   echo "test" | git hash-object -w --stdin
   git commit --allow-empty -m "test signature"
   git verify-commit HEAD
   ```

## ENS Issues

### "ENS resolution failed for vitalik.eth"

**Cause:** Missing or invalid Alchemy API key.

**Solution:**
1. **Check API key:**
   ```bash
   echo $ALCHEMY_API_KEY
   # Should show your key
   ```

2. **Set API key:**
   ```bash
   export ALCHEMY_API_KEY="your-key-here"
   # Add to ~/.bashrc to persist
   ```

3. **Test resolution:**
   ```bash
   git repobox check vitalik.eth read *
   # Should resolve without error
   ```

4. **Check network connectivity:**
   ```bash
   curl -s https://eth-mainnet.alchemyapi.io/v2/$ALCHEMY_API_KEY \
     -X POST \
     -H "Content-Type: application/json" \
     -d '{"id":1,"jsonrpc":"2.0","method":"eth_blockNumber","params":[]}'
   ```

### "Invalid ENS name format"

**Cause:** ENS name has invalid characters or format.

**Common issues:**
- Double dots: `invalid..eth`
- Spaces: `bad name.eth`
- Invalid characters: `name@test.eth`

**Solution:** Use valid ENS name formats:
```yaml
groups:
  valid-names:
    - alice.eth         # ✓ Valid
    - company.com       # ✓ Valid  
    - project.xyz       # ✓ Valid
    - "valid-name.eth"  # ✓ Valid (quote if needed)
```

## Git Integration Issues

### "repo.box not found in PATH"

**Cause:** repo.box binary not properly installed or not in PATH.

**Solution:**
1. **Check installation:**
   ```bash
   which repobox
   # Should show /usr/local/bin/repobox or similar
   ```

2. **Reinstall if missing:**
   ```bash
   curl -sSf https://repo.box/install.sh | sh
   ```

3. **Add to PATH manually:**
   ```bash
   export PATH="/usr/local/bin:$PATH"
   # Add to ~/.bashrc to persist
   ```

### "Git hooks not working"

**Cause:** Git hooks not properly configured or repo not initialized.

**Solution:**
1. **Check if repo.box is initialized:**
   ```bash
   ls .repobox/
   # Should show config.yml and possibly hooks/
   ```

2. **Reinitialize if needed:**
   ```bash
   git repobox init
   ```

3. **Check git hook configuration:**
   ```bash
   git config core.hooksPath
   # Should show .repobox/hooks or similar
   ```

### "Git shim not intercepting commands"

**Cause:** Git not configured to use repo.box as shim.

**Solution:**
1. **Check git configuration:**
   ```bash
   git config alias.rb
   # Should show something like "!repobox"
   ```

2. **Reconfigure shim:**
   ```bash
   git repobox setup
   ```

3. **Test shim:**
   ```bash
   git rb status  # Should work
   git repobox status  # Should also work
   ```

## Performance Issues

### "Permission checks are slow"

**Cause:** ENS resolution or complex group lookups.

**Solutions:**
1. **Use EVM addresses instead of ENS for performance-critical cases:**
   ```yaml
   groups:
     fast-group:
       - evm:0x7D5b...  # Faster than alice.eth
   ```

2. **Increase cache TTL for on-chain groups:**
   ```yaml
   groups:
     token-holders:
       cache_ttl: 3600  # Cache for 1 hour
   ```

3. **Simplify group hierarchies:**
   ```yaml
   # Avoid deeply nested group includes
   groups:
     simple:
       - alice.eth
       - bob.eth
   ```

### "Large repository operations are slow"

**Cause:** File permission checks on large changesets.

**Solutions:**
1. **Use broader file patterns:**
   ```yaml
   # Instead of checking every file
   - agents edit src/specific/file.txt
   
   # Use patterns  
   - agents edit src/** >feature/**
   ```

2. **Reduce rule complexity** for common operations.

3. **Use `read` permission liberally** - it has minimal overhead.

## Common Gotchas

### Wrong Target Syntax

**File targets don't need `./` prefix:**
```yaml
# Wrong
- agents edit >./src/**

# Correct  
- agents edit src/**
```

**Branch targets need `>` prefix:**
```yaml
# Wrong
- agents push feature/**

# Correct
- agents push >feature/**
```

### Mixed Target Types
```yaml
# Wrong - can't mix without proper syntax
- agents edit src/** feature/**

# Correct - file + branch
- agents edit src/** >feature/**
```

### Case Sensitivity
All identifiers are case-sensitive:
- Group names: `Agents` ≠ `agents`
- Branch names: `Main` ≠ `main`  
- Verbs: `Push` ≠ `push`

### Default Policy Confusion
```yaml
# This means "deny everything not explicitly allowed"
permissions:
  default: deny
  
# This means "allow everything not explicitly denied"
permissions:
  default: allow
```

## Getting Help

### Diagnostic Commands
```bash
# Full status overview
git repobox status

# Validate configuration
git repobox lint

# Test specific permission
git repobox check <identity> <verb> <target>

# Show available keys
git repobox keys list

# Show current identity  
git repobox whoami
```

### Verbose Logging
Set environment variable for detailed output:
```bash
export RUST_LOG=debug
git repobox check alice push main
# Shows detailed rule evaluation
```

### Common Debug Workflow
1. **Identify the failing operation**
2. **Check current identity:** `git repobox whoami`
3. **Test permission explicitly:** `git repobox check <identity> <verb> <target>`
4. **Review configuration:** `git repobox status`
5. **Validate config:** `git repobox lint`
6. **Fix and test again**

### When to Contact Support
- Crypto/signature verification errors
- Binary crashes or segfaults  
- Unexpected behavior after following troubleshooting steps
- Performance issues not resolved by optimization

Include in support requests:
- Operating system and version
- Git version (`git --version`)
- repo.box version (`repobox --version` if available)
- Full error message
- Relevant config snippet (anonymized)