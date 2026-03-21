# ENS Names in Permissions - Usage Guide

This guide shows how to use ENS names in repo.box permission configurations.

## Overview

You can now use human-readable ENS names like `vitalik.eth` instead of hex addresses in your `.repobox/config.yml` files. ENS names are resolved at permission evaluation time, so if an ENS name changes ownership, permissions automatically follow the new owner.

## Basic Usage

### Configuration Syntax

```yaml
groups:
  founders:
    - vitalik.eth          # Implicit ENS detection
    - alice.eth
    - ens:bob.eth          # Explicit ENS prefix (optional)
  
  core-team:
    - charlie.eth
    - evm:0x1234567890123456789012345678901234567890  # Mixed with EVM addresses

permissions:
  default: deny
  rules:
    # Direct ENS usage in permission rules
    - "vitalik.eth push >main"
    - "alice.eth edit contracts/**"
    
    # Group-based permissions with ENS names
    - "founders merge >*"
    - "core-team push >develop"
    
    # Mixed configurations
    - "bob.eth force-push >hotfix/**"
    - "evm:0x1234567890123456789012345678901234567890 push >feature/**"
```

### Supported ENS TLDs

- `.eth` (primary ENS TLD)
- `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`

### CLI Usage

```bash
# Check permissions for ENS names
repobox check vitalik.eth push main
repobox check alice.eth edit contracts/token.sol
repobox check ens:bob.eth merge develop

# Check ownership (all verbs)
repobox check vitalik.eth own main
```

## Configuration Examples

### Basic ENS Setup

```yaml
groups:
  maintainers:
    - vitalik.eth
    - alice.eth

permissions:
  default: allow
  rules:
    - "vitalik.eth push >main"
    - "alice.eth edit >main"
    - "maintainers push >develop"
```

### Advanced Mixed Configuration

```yaml
groups:
  # Static group with ENS names
  core-team:
    - vitalik.eth
    - alice.eth
    
  # Mixed EVM and ENS identities
  contributors:
    - bob.eth
    - charlie.eth
    - evm:0x1234567890123456789012345678901234567890
    
  # Include other groups
  all-developers:
    - core-team      # Group include
    - contributors   # Group include
    - david.eth      # Additional ENS member

permissions:
  default: deny
  rules:
    # Founders have full control
    - "vitalik.eth own >*"
    
    # Core team permissions
    - "core-team push >main"
    - "core-team merge >main"
    - "core-team edit contracts/**"
    
    # Contributors can work on features
    - "contributors push >feature/**"
    - "contributors branch >feature/**"
    - "contributors edit src/**"
    
    # File-specific permissions
    - "alice.eth edit .repobox/config.yml"
    - "bob.eth append .repobox/config.yml"
    
    # Deny overrides
    - "contributors not merge >main"
    - "contributors not edit .repobox/config.yml"
```

## ENS Resolution

### Automatic Resolution

ENS names are resolved automatically when:
- Checking permissions via CLI (`repobox check`)
- Evaluating git operations (push, merge, etc.)
- Processing group membership

### Resolution Requirements

- **Alchemy API Key**: Set `ALCHEMY_API_KEY` environment variable
- **Network Access**: Requires HTTPS access to `repo.box/api`
- **Caching**: Resolutions are cached for 60 seconds by default

### Resolution Flow

1. **Parse Time**: ENS names detected and validated
2. **Evaluation Time**: Names resolved to Ethereum addresses via Alchemy API
3. **Permission Check**: Resolved addresses compared for matches
4. **Caching**: Successful resolutions cached with TTL

### Error Handling

ENS resolution follows a **fail-closed** security policy:

```bash
# If ENS resolution fails, permission is denied
$ repobox check invalid.eth push main
❌ denied — network error resolving ENS name 'invalid.eth'

# Valid ENS names work normally
$ ALCHEMY_API_KEY=your_key repobox check vitalik.eth push main  
✅ allowed — ens:vitalik.eth push main
```

## Testing and Validation

### Configuration Validation

```bash
# Validate ENS names in config
repobox lint

# Example output:
✅ .repobox/config.yml is valid
   2 groups, 5 rules, default: Deny
   💡 'ens:vitalik.eth' has branch rules but no file rules
```

### Permission Testing

```bash
# Test specific permissions
repobox check alice.eth push feature/new-contract
repobox check bob.eth edit contracts/token.sol

# Test ownership
repobox check vitalik.eth own main
```

### Development Testing

Use the included test scripts:

```bash
# Basic ENS functionality
./test_ens_cli.sh

# Comprehensive testing
./test_ens_comprehensive.sh
```

## Migration from EVM Addresses

### Before (EVM only)
```yaml
groups:
  founders:
    - evm:0xd8da6bf26964af9d7eed9e03e53415d37aa96045
    - evm:0x123456789abcdef123456789abcdef1234567890

permissions:
  rules:
    - "founders push >*"
```

### After (ENS names)
```yaml
groups:
  founders:
    - vitalik.eth    # Resolves to 0xd8da6bf...
    - alice.eth      # Resolves to 0x12345678...

permissions:
  rules:
    - "founders push >*"
```

## Security Considerations

### ENS Name Ownership

- **Dynamic Resolution**: Permissions follow ENS ownership changes
- **Cache TTL**: 60-second cache limits exposure to ownership changes
- **Monitoring**: Monitor critical ENS names for ownership changes

### Fail-Closed Security

- **Resolution Errors**: Always deny permission on resolution failure
- **Network Issues**: No fallback to allow-by-default
- **Invalid Names**: Rejected at configuration parse time

### Best Practices

1. **Monitor Critical Names**: Watch for ownership changes of important ENS names
2. **Use Groups**: Organize ENS names into logical groups
3. **Test Configurations**: Validate configs with `repobox lint`
4. **API Key Security**: Protect Alchemy API key as a secret
5. **Backup Resolution**: Consider multiple resolution sources

## Troubleshooting

### Common Issues

**ENS name not recognized:**
```bash
$ repobox check invalid-name push main
error: invalid identity: invalid-name
```
*Solution*: Use valid ENS format (`name.eth`, `name.box`, etc.)

**Resolution failure:**
```bash
$ repobox check alice.eth push main
❌ denied — network error resolving ENS name 'alice.eth'
```
*Solution*: Check ALCHEMY_API_KEY and network connectivity

**Config validation errors:**
```bash
$ repobox lint
error: invalid ENS name format: test.invalid
```
*Solution*: Use supported TLDs (.eth, .box, .com, etc.)

### Debug Mode

Enable verbose resolution logging:
```bash
RUST_LOG=debug repobox check alice.eth push main
```

## API Reference

### Environment Variables

- `ALCHEMY_API_KEY`: Required for ENS resolution in production

### Configuration Schema

```yaml
groups:
  <group-name>:
    - <ens-name>                    # alice.eth
    - ens:<ens-name>               # ens:alice.eth  
    - evm:<address>                # evm:0x123...
    - <group-include>              # other-group

permissions:
  default: allow | deny
  rules:
    - "<subject> <verb> <target>"   # alice.eth push >main
```

### Supported TLDs

`.eth`, `.box`, `.com`, `.xyz`, `.org`, `.io`, `.dev`, `.app`

## Examples Repository

See `test_ens_comprehensive.sh` for working examples of all ENS features.

---

For more information, see the [full ENS permissions specification](./specs/ens-permissions.md).