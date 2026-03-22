# Full Knowledge Sweep — Technical Specification

**Priority: P0 (Demo Blocker)**  
**Author:** PM Agent (0x9aBA6b1a5175CA8fd97D6c83c2Dd66dA6f47234b)  
**Created:** 2026-03-22  
**Status:** Ready for Implementation  

## Executive Summary

Comprehensive accuracy pass across ALL repo.box knowledge surfaces to eliminate contradictions, update stale content, and ensure demo-readiness. Current state has multiple inconsistencies between config file names, verb definitions, and feature descriptions that could confuse users and developers.

## Critical Inconsistencies Discovered

### 1. **CONFIG FILE NAME MISMATCH (P0)**
- **Rust parser expects:** `.repobox/config.yml` 
- **Landing page SKILL.md shows:** `.repobox-config` (without directory, wrong format)
- **Impact:** Complete feature breakdown - users follow landing page docs and CLI fails
- **Files affected:** `/repobox-landing/public/SKILL.md`, `/docs/SKILL.md`

### 2. **VERB INCONSISTENCIES (P1)**
- **Rust source has:** `read`, `push`, `merge`, `branch`, `create`, `delete`, `force-push`, `edit`, `write`, `append`
- **llms.txt omits:** `read`, `branch`, `create` verbs
- **Playground prompt omits:** `read` verb
- **Landing docs show:** `create` as `branch` in some places
- **Impact:** Inaccurate capability description, broken examples

### 3. **FEATURE CLAIMS NOT IMPLEMENTED (P1)**
- **llms.txt claims:** "address-less push routing", "on-chain group resolvers"
- **Current state:** Basic local-only implementation, no server deployment mentioned
- **Impact:** Oversells capabilities, demo will not match claims

### 4. **ENS SUPPORT UNDERDOCUMENTED (P2)**
- **Rust code has:** Full ENS parsing with `ens:` prefix and implicit detection
- **Docs show:** Only EVM addresses in examples
- **Impact:** Users unaware of ENS capability

### 5. **X402 PAYMENTS UNDERDOCUMENTED (P2)**
- **Rust config has:** Full X402Config struct for paid access
- **Knowledge surfaces:** No mention of payment features
- **Impact:** Hidden monetization capability

## Implementation Plan

### Phase 1: Core Accuracy (P0) — 2 hours

#### 1.1 Fix Config File Name Crisis
**Files:** `/docs/SKILL.md`, `/repobox-landing/public/SKILL.md`
**Change:** All references `.repobox-config` → `.repobox/config.yml`
**Test:** Grep for `.repobox-config` across entire repo, verify zero matches

#### 1.2 Update llms.txt with Current Capabilities
**File:** `/llms.txt`
**Changes:**
- Add missing verbs: `read`, `branch`, `create`
- Update verb table to match Rust enum exactly
- Remove/qualify unimplemented claims (address-less push, on-chain resolvers)
- Add accurate architecture: "CLI-only" not "server-hosted git"
- Update stats: latest commit count, test count, file count

#### 1.3 Fix README.md Accuracy
**File:** `/README.md`  
**Changes:**
- Align verb descriptions with Rust implementation
- Fix config file path references
- Remove server deployment claims if not implemented
- Update install script reference to match actual location

### Phase 2: Documentation Sync (P1) — 3 hours

#### 2.1 Playground Prompt Accuracy
**File:** `/repobox-landing/src/lib/repobox-prompt.ts`
**Changes:**
- Add missing `read` verb to system prompt
- Align verb descriptions with Rust enum
- Fix config file path in all examples
- Update target syntax to match parser exactly

#### 2.2 Landing Page Components Audit
**Files:** `/repobox-landing/src/components/landing/*.tsx`
**Changes:**
- Audit all code examples for config file paths
- Update verb demonstrations to match implementation
- Remove claims about unimplemented features

#### 2.3 Skills Documentation
**Files:** `/docs/SKILL.md`, `/skills/wall/SKILL.md`
**Changes:**
- Standardize on `.repobox/config.yml` throughout
- Add ENS examples: `ens:vitalik.eth`
- Document `read` verb capability
- Add X402 payment configuration section

### Phase 3: Feature Completeness (P1) — 2 hours

#### 3.1 AGENTS.md for Consumer Clarity
**File:** `/AGENTS.md` (create if missing)
**Content:**
- Clear "what agents need to know" guide
- Key generation workflow for agent developers
- Permission model for agent spawning
- Integration with existing agent frameworks

#### 3.2 Docs Structure Audit
**Directory:** `/docs/`
**Changes:**
- Remove "Coming Soon" from completed features
- Mark actual unimplemented features clearly
- Consolidate redundant documentation

### Phase 4: Cross-Reference Validation (P1) — 1 hour

#### 4.1 Rust Source Cross-Check
**Method:** Parse Rust enums and structs, compare against all docs
**Files:** Check against `/repobox-core/src/{config,parser,engine}.rs`
**Output:** Automated inconsistency report

#### 4.2 End-to-End Testing References
**Method:** Verify all documented examples work with actual CLI
**Test cases:**
- All config examples can be parsed
- All verb examples are valid
- All CLI commands exist and work

## Verification Criteria

### Phase 1 Success Metrics
- [ ] Zero grep matches for `.repobox-config` in entire repo
- [ ] llms.txt verb list exactly matches `Verb::parse()` implementation
- [ ] All capability claims in llms.txt are verifiable via tests
- [ ] README install flow works end-to-end

### Phase 2 Success Metrics
- [ ] Playground generates valid configs that CLI can parse
- [ ] All landing page code examples use correct file paths
- [ ] Skills documentation matches CLI behavior exactly

### Phase 3 Success Metrics
- [ ] AGENTS.md provides complete agent integration guide
- [ ] Documentation contains no "Coming Soon" for implemented features
- [ ] All documented features have working examples

### Phase 4 Success Metrics
- [ ] Automated cross-reference check passes 100%
- [ ] All documented config examples parse successfully with CLI
- [ ] Demo flow works end-to-end with current documentation

## Files Requiring Changes

### Immediate (P0)
1. `/llms.txt` — Complete rewrite for accuracy
2. `/docs/SKILL.md` — Config path fixes, verb accuracy
3. `/repobox-landing/public/SKILL.md` — Config path fixes
4. `/README.md` — Feature accuracy, installation flow

### Secondary (P1)
5. `/repobox-landing/src/lib/repobox-prompt.ts` — System prompt accuracy
6. `/skills/wall/SKILL.md` — Config path consistency
7. `/AGENTS.md` — Create comprehensive agent guide
8. `/docs/spec/*.md` — Remove stale specs or mark as implemented

### Validation (P1)
9. Create `/scripts/docs-consistency-check.sh` — Automated validation
10. Update `/test-config.yml` — Use as canonical example

## Risk Analysis

**High Risk:**
- Config file path mismatch breaks all user workflows
- Verb inconsistencies cause permission system failures
- Oversold features damage credibility in demo

**Medium Risk:** 
- Missing ENS documentation reduces adoption
- Incomplete agent integration guides slow developer adoption

**Low Risk:**
- Minor formatting inconsistencies in examples

## Implementation Notes

### Git Strategy
- One commit per phase for clean rollback
- Test each phase independently
- Use PM agent identity for all commits

### Quality Gates
- Each file change must pass CLI validation
- All config examples must parse with `repobox lint`
- Cross-reference script must pass before merge

### Demo Readiness Checklist
- [ ] All demo scenarios use documented syntax exactly
- [ ] No features mentioned that aren't implemented
- [ ] User journey from install → working config is friction-free
- [ ] Error messages match what docs claim

## Timeline

- **Phase 1:** 2 hours (critical fixes)
- **Phase 2:** 3 hours (documentation sync)
- **Phase 3:** 2 hours (completeness)  
- **Phase 4:** 1 hour (validation)
- **Total:** 8 hours implementation + 2 hours testing = 10 hours

**Demo Readiness:** After Phase 1 complete, safe for demo scenarios.
**Full Polish:** After Phase 4 complete, ready for public release.

## Success Definition

✅ **Demo-ready:** User can follow any doc → working setup with zero confusion  
✅ **Developer-ready:** Agent integration is clearly documented and tested  
✅ **Accurate:** Zero false claims about capabilities or features  
✅ **Complete:** No stale "Coming Soon" content for implemented features

---

*This specification was generated by analyzing current state vs. source of truth (Rust parser + config types). Implementation should proceed in phase order to maintain consistency and testability.*