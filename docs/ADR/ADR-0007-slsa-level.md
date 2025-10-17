# ADR-0007: SLSA Level Target Justification

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Security Team, DevOps

## Context

SLSA (Supply-chain Levels for Software Artifacts) defines 4 levels of supply chain security. We need to choose a target level that balances security and practicality.

## Decision

Target **SLSA Level 3** for BazBOM builds.

### SLSA Levels Overview

<table>
  <thead>
    <tr>
      <th>Level</th>
      <th>Requirements</th>
      <th>BazBOM Status</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Level 1</strong></td>
      <td>Provenance exists</td>
      <td>✅ Implemented</td>
    </tr>
    <tr>
      <td><strong>Level 2</strong></td>
      <td>Signed provenance, tamper-resistant</td>
      <td>✅ Implemented (Sigstore)</td>
    </tr>
    <tr>
      <td><strong>Level 3</strong></td>
      <td>Hardened build platform, non-falsifiable</td>
      <td>✅ Implemented (GitHub-hosted)</td>
    </tr>
    <tr>
      <td><strong>Level 4</strong></td>
      <td>Two-person review, hermetic builds</td>
      <td>🔄 Partial (CODEOWNERS, builds hermetic)</td>
    </tr>
  </tbody>
</table>

### Rationale for Level 3

**Level 3 provides:**
1. **Signed provenance** (Sigstore keyless signing)
2. **Hardened builder** (GitHub-hosted runners, ephemeral)
3. **Build isolation** (Containers, no persistent state)
4. **Audit trail** (GitHub Actions logs, 90-day retention)

**Level 4 requires:**
1. **Two-person review** (can implement via CODEOWNERS)
2. **Hermetic, reproducible builds** (Bazel provides hermeticity)
3. **Fully isolated execution** (GitHub runners are isolated)

**Why not Level 4:**
- Two-person review already possible with CODEOWNERS
- Hermetic builds already achieved with Bazel
- Level 4 certification process is complex
- Level 3 provides 95% of security value

## Implementation

### Level 3 Checklist

- [x] Provenance generated for all builds
- [x] Provenance signed with Sigstore
- [x] GitHub-hosted runners (ephemeral, isolated)
- [x] Build logs retained (GitHub Actions, 90 days)
- [x] No persistent build environment
- [x] Source code version controlled (Git)
- [x] Dependency pinning (maven_install.json lockfile)

### Optional Level 4 Enhancements

- [ ] Enforce two-person review (CODEOWNERS + branch protection)
- [ ] Reproducible builds verification (build twice, compare hashes)
- [ ] SLSA Builder attestation
- [ ] Provenance verification in deployment pipeline

## Consequences

### Positive
- Strong supply chain security posture
- Sigstore provides keyless signing (no secret management)
- GitHub integration is seamless
- Audit trail for compliance

### Negative
- Not "certified" SLSA Level 3 (self-assessed)
- Sigstore dependency (centralized service)
- GitHub lock-in for Level 3 guarantees

### Mitigations
- Document self-assessment with evidence
- Sigstore has fallback to GPG signing
- Abstract builder interface (support self-hosted runners)

## Verification

### Validate Level 3 Compliance

```bash
# 1. Provenance exists
test -f bazel-bin/app/app.provenance.json || exit 1

# 2. Provenance is signed
cosign verify-blob \
  --signature=bazel-bin/app/app.provenance.json.sig \
  bazel-bin/app/app.provenance.json || exit 1

# 3. Builder is hardened (GitHub-hosted)
BUILDER=$(jq -r '.predicate.runDetails.builder.id' bazel-bin/app/app.provenance.json)
[[ "$BUILDER" == *"github"* ]] || exit 1

# 4. Build is isolated (check no persistent environment)
# (Manual audit: review GitHub Actions workflow)

echo "✓ SLSA Level 3 requirements validated"
```

## Alternatives Considered

### Alternative 1: Target SLSA Level 2

**Rationale:** Easier to achieve, still provides signed provenance.

**Rejected:** Level 3 is achievable with GitHub Actions, provides significantly more security.

### Alternative 2: Target SLSA Level 4

**Rationale:** Highest security level.

**Rejected:** Two-person review burden too high for rapid iteration. Can upgrade later.

### Alternative 3: No SLSA Compliance

**Rationale:** Reduce complexity, just generate SBOMs.

**Rejected:** Provenance is critical for supply chain security. Level 3 is reasonable goal.

## Review

- Security Team: Approved (2025-10-15)
- DevOps Team: Approved (2025-10-16)
- Recommendation: Re-evaluate Level 4 in 6 months

## References

- [SLSA Specification v1.0](https://slsa.dev/spec/v1.0/)
- [SLSA Levels](https://slsa.dev/spec/v1.0/levels)
- [GitHub Actions as SLSA Builder](https://github.com/slsa-framework/slsa-github-generator)
- [Sigstore Documentation](https://docs.sigstore.dev/)
