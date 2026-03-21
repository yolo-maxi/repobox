# repo.box Spec: x402 Paid Repository Access

## Overview

Enable pay-to-read access for private repositories using the x402 HTTP payment protocol with USDC on Base. When an unauthenticated or unauthorized agent tries to clone/fetch a private repo, the server returns HTTP 402 with x402 payment requirements. After payment, the agent's EVM address is added to a `paid-readers` group in the repo's `.repobox/config.yml`, granting permanent read access.

## Flow

1. Agent: `git clone https://git.repo.box/some-private-repo.git`
2. Server: recovers EVM address from auth header (existing auth.rs)
3. Server: checks read permissions (existing check_read_access)
4. If denied → check if repo has x402 pricing configured
5. If x402 enabled → return HTTP 402 with payment requirements header
6. Agent/client pays USDC on Base via x402 protocol
7. Server verifies payment on-chain
8. Server adds agent's EVM address to `paid-readers` group in repo config
9. Agent retries clone → now has read access → succeeds

## x402 Protocol Integration

### Response Headers (on 402)

Per the x402 spec, the server returns:
```
HTTP/1.1 402 Payment Required
X-Payment: {"scheme":"exact","network":"base","currency":"USDC","amount":"1000000","recipient":"0x...","memo":"read:reponame"}
```

### Payment Verification

Use the x402 facilitator approach:
- Agent includes `X-Payment-Response` header with the payment proof
- Server verifies the payment via Base RPC (check USDC transfer event)
- On success, grant access

### Config Extension

Add optional `x402` section to `.repobox/config.yml`:

```yaml
x402:
  read_price: "1.00"      # USDC per clone/read access (permanent)
  recipient: "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048"  # payment recipient
  network: "base"          # chain for payments

groups:
  paid-readers: []         # auto-populated by payment grants

permissions:
  default: deny
  rules:
    - paid-readers read >*
```

## Implementation Scope (Hackathon MVP)

### Must Have
1. **Config parser**: extend `repobox-core/src/config.rs` to parse `x402` section
2. **402 response**: in `check_read_access()`, when read is denied AND x402 config exists, return 402 with payment headers instead of 403
3. **Payment verification endpoint**: `POST /{repo}/x402/verify` that checks USDC transfer on Base and grants access
4. **Grant mechanism**: append EVM address to `paid-readers` group in repo config (write to git)

### Nice to Have (Post-Hackathon)
- Superfluid streaming payments for time-limited access
- Per-path pricing (pay for specific directories)
- Revenue sharing between contributors

## Technical Details

### USDC on Base
- Contract: `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913`
- Decimals: 6
- Verify transfers via `Transfer(from, to, amount)` event logs

### Files to Modify
- `repobox-core/src/config.rs` — add X402Config struct + parsing
- `repobox-core/src/parser.rs` — parse x402 section from YAML
- `repobox-server/src/routes.rs` — modify check_read_access to return 402, add verify endpoint
- `repobox-server/Cargo.toml` — add alloy/ethers for on-chain verification (or use reqwest + Base RPC)

### Demo Repository
Create a demo repo at `git.repo.box` with:
- A private repo with x402 pricing enabled
- Sample `.repobox/config.yml` with x402 section
- README explaining the pay-to-read flow
