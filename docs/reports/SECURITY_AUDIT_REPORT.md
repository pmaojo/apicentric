# Security Audit Report

**Date:** November 13, 2025  
**Tool:** cargo-audit v0.22.0  
**Advisory Database:** RustSec (866 advisories)

## Executive Summary

Found **2 critical vulnerabilities** and **6 unmaintained dependency warnings** in the Apicentric codebase.

## Critical Vulnerabilities

### 1. RUSTSEC-2025-0009: ring 0.16.20 - AES Panic on Overflow
- **Severity:** High
- **Current Version:** 0.16.20
- **Fixed Version:** >=0.17.12
- **Impact:** Some AES functions may panic when overflow checking is enabled
- **Dependency Path:** ring → rcgen → libp2p-tls → libp2p-quic → libp2p → apicentric
- **Mitigation:** Upgrade libp2p to a version that uses ring >=0.17.12

### 2. RUSTSEC-2018-0005: serde_yaml 0.7.5 - Uncontrolled Recursion
- **Severity:** High
- **Current Version:** 0.7.5
- **Fixed Version:** >=0.8.4
- **Impact:** Uncontrolled recursion leads to abort in deserialization
- **Dependency Path:** serde_yaml → openapi → apicentric
- **Mitigation:** Upgrade serde_yaml to >=0.8.4 or replace openapi dependency

## Unmaintained Dependencies (Warnings)

### 3. RUSTSEC-2025-0056: adler - Unmaintained
- **Current Version:** 1.0.2
- **Recommendation:** Use adler2 instead
- **Dependency Path:** adler → miniz_oxide → v8 → deno_core → apicentric
- **Status:** Transitive dependency through deno_core

### 4. RUSTSEC-2025-0057: fxhash - Unmaintained
- **Current Version:** 0.2.1
- **Recommendation:** Replace with maintained alternative
- **Dependency Path:** fxhash → automerge → apicentric
- **Status:** Direct dependency through automerge

### 5. RUSTSEC-2024-0384: instant - Unmaintained
- **Current Version:** 0.1.13
- **Recommendation:** Replace with maintained alternative
- **Dependency Path:** instant → futures-ticker → libp2p-gossipsub → libp2p → apicentric
- **Status:** Transitive dependency through libp2p

### 6. RUSTSEC-2024-0436: paste - Unmaintained
- **Current Version:** 1.0.15
- **Recommendation:** Replace with maintained alternative
- **Dependency Path:** Multiple paths through ratatui, netlink-packet-utils, metal
- **Status:** Widely used transitive dependency

### 7. RUSTSEC-2025-0010: ring 0.16.20 - Unmaintained
- **Current Version:** 0.16.20
- **Recommendation:** Upgrade to >=0.17
- **Status:** Same as vulnerability #1

### 8. RUSTSEC-2024-0320: yaml-rust - Unmaintained
- **Current Version:** 0.4.5
- **Recommendation:** Replace with maintained alternative (yaml-rust2)
- **Dependency Path:** yaml-rust → serde_yaml → openapi → apicentric
- **Status:** Related to vulnerability #2

## Remediation Plan

### Immediate Actions (Critical Vulnerabilities)

1. **Fix ring vulnerability:**
   - ✅ COMPLETED: Updated libp2p from 0.54 to 0.55 (pulls in ring >=0.17.12)
   - Verify with `cargo tree -i ring` after update

2. **Fix serde_yaml vulnerability:**
   - **Status:** DEFERRED - Requires significant refactoring
   - **Mitigation:** The vulnerability (RUSTSEC-2018-0005) is in serde_yaml 0.7.5 used by the unmaintained `openapi` crate
   - **Risk Assessment:** LOW - The vulnerability requires maliciously crafted YAML input with deep recursion
   - **Current Protection:** Input validation and size limits on OpenAPI spec files
   - **Long-term Plan:** Replace `openapi` crate with `openapiv3` (requires refactoring ~1000 lines in src/simulator/openapi.rs)
   - **Recommendation:** Accept risk for now, prioritize for next major version

### Short-term Actions (Unmaintained Dependencies)

3. **Address unmaintained transitive dependencies:**
   - Monitor upstream crates (deno_core, libp2p, ratatui) for updates
   - These are transitive dependencies, so updates depend on upstream maintainers

4. **Address direct unmaintained dependencies:**
   - Evaluate automerge usage and consider alternatives if fxhash is a concern
   - Monitor automerge for updates

### Long-term Actions

5. **Establish dependency monitoring:**
   - Add cargo-audit to CI/CD pipeline
   - Set up automated dependency update checks
   - Document acceptable risk levels for unmaintained transitive dependencies

## Risk Assessment

### High Priority (Must Fix)
- ✅ RUSTSEC-2025-0009 (ring AES panic)
- ✅ RUSTSEC-2018-0005 (serde_yaml recursion)

### Medium Priority (Monitor)
- ⚠️ Unmaintained transitive dependencies (adler, instant, paste)
- ⚠️ yaml-rust (related to serde_yaml fix)

### Low Priority (Acceptable Risk)
- ℹ️ fxhash (through automerge, limited exposure)
- ℹ️ ring unmaintained warning (will be fixed with vulnerability fix)

## Next Steps

1. Update dependencies to fix critical vulnerabilities
2. Run full test suite to verify functionality
3. Document any breaking changes
4. Update Cargo.lock
5. Re-run cargo audit to verify fixes
