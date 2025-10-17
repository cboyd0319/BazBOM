# SLSA Provenance Generation Guide

**Audience:** Security engineers, DevSecOps, release engineers
**Purpose:** Generate and verify SLSA provenance attestations for BazBOM artifacts
**Last Reviewed:** 2025-10-17

## TL;DR

BazBOM generates SLSA provenance v1.0 attestations for all build artifacts. Provenance is signed with Sigstore (cosign) and attached to releases for supply chain verification.

```bash
# Generate provenance
bazel build //:provenance_all

# Sign with Sigstore
cosign sign-blob --yes bazel-bin/app.provenance.json \
  --output-signature=bazel-bin/app.provenance.json.sig

# Verify signature
cosign verify-blob --signature=bazel-bin/app.provenance.json.sig \
  bazel-bin/app.provenance.json
```

## What is SLSA Provenance?

SLSA (Supply-chain Levels for Software Artifacts) provenance is a record of how an artifact was built. It includes:

- **Materials:** Source code, dependencies, build scripts used
- **Builder:** System that performed the build
- **Build process:** Commands, environment, parameters
- **Output:** Artifacts produced with hashes

This enables:
- **Verification:** Confirm artifact matches source
- **Auditability:** Trace artifact to exact build
- **Security:** Detect tampering or malicious builds

## SLSA Levels

BazBOM targets **SLSA Level 3**:

| Level | Requirements | BazBOM Status |
|-------|-------------|---------------|
| SLSA 1 | Provenance exists | ✅ Implemented |
| SLSA 2 | Signed provenance | ✅ Implemented (Sigstore) |
| SLSA 3 | Hardened build platform | ✅ GitHub-hosted runners |
| SLSA 4 | Two-party review | 🚧 Optional (via CODEOWNERS) |

## Provenance Schema (SLSA v1.0)

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "pkg:bazel/app@1.0.0",
      "digest": {
        "sha256": "a42edc9cab792e39fe39bb94f3fca655ed157ff87a8af78e1d6ba5b07c4a00ab"
      }
    }
  ],
  "predicateType": "https://slsa.dev/provenance/v1",
  "predicate": {
    "buildDefinition": {
      "buildType": "https://slsa.dev/bazel/v1",
      "externalParameters": {
        "repository": "https://github.com/cboyd0319/BazBOM",
        "ref": "refs/heads/main",
        "commit": "abc123...",
        "workflow": ".github/workflows/supplychain.yml"
      },
      "internalParameters": {
        "bazelVersion": "7.0.0",
        "targets": ["//app:deployable"]
      },
      "resolvedDependencies": [
        {
          "uri": "pkg:maven/com.google.guava/guava@31.1-jre",
          "digest": {"sha256": "..."}
        }
      ]
    },
    "runDetails": {
      "builder": {
        "id": "https://github.com/actions/runner/github-hosted"
      },
      "metadata": {
        "invocationId": "https://github.com/cboyd0319/BazBOM/actions/runs/123456",
        "startedOn": "2025-10-17T12:00:00Z",
        "finishedOn": "2025-10-17T12:15:00Z"
      },
      "byproducts": [
        {
          "name": "build.log",
          "uri": "https://github.com/cboyd0319/BazBOM/actions/runs/123456"
        }
      ]
    }
  }
}
```

## Generating Provenance

### Local Build

```bash
# Generate provenance for all targets
bazel build //:provenance_all \
  --define commit_sha=$(git rev-parse HEAD) \
  --define build_id=local-$(date +%s) \
  --define builder=local

# Generate provenance for single target
bazel build //app:app_provenance \
  --define commit_sha=$(git rev-parse HEAD)
```

Output: `bazel-bin/app/app.provenance.json`

### CI Build (GitHub Actions)

```yaml
- name: Generate SLSA provenance
  run: |
    bazel build //:provenance_all \
      --define commit_sha=${{ github.sha }} \
      --define build_id=${{ github.run_id }} \
      --define builder=github-actions \
      --define workflow=${{ github.workflow }}
```

### Custom Build Metadata

Pass additional context via `--define`:

```bash
bazel build //:provenance_all \
  --define commit_sha=$(git rev-parse HEAD) \
  --define build_id=$(uuidgen) \
  --define builder=$(hostname) \
  --define bazel_version=$(bazel version | head -1) \
  --define build_timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
```

## Signing Provenance

### Option 1: Sigstore (Keyless Signing)

**Recommended for CI/CD.** Uses OIDC identity for signing (no key management).

```bash
# Install cosign
curl -O -L https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign

# Sign provenance (keyless)
cosign sign-blob --yes \
  bazel-bin/app/app.provenance.json \
  --output-signature=bazel-bin/app/app.provenance.json.sig
```

In GitHub Actions:

```yaml
- name: Install cosign
  uses: sigstore/cosign-installer@v3

- name: Sign provenance with Sigstore
  env:
    COSIGN_EXPERIMENTAL: 1  # Enable keyless signing
  run: |
    for provenance in bazel-bin/**/*.provenance.json; do
      cosign sign-blob --yes "$provenance" \
        --output-signature="${provenance}.sig"
    done
```

### Option 2: GPG Signing

**For air-gapped environments or existing key infrastructure.**

```bash
# Sign with GPG key
gpg --armor --detach-sign \
  --output bazel-bin/app/app.provenance.json.asc \
  bazel-bin/app/app.provenance.json

# Verify
gpg --verify bazel-bin/app/app.provenance.json.asc \
  bazel-bin/app/app.provenance.json
```

### Option 3: In-Toto Attestation

**For full in-toto compliance.**

```bash
# Install in-toto
pip install in-toto

# Create attestation
in-toto-run \
  --step-name build \
  --key builder.pem \
  --materials WORKSPACE BUILD.bazel \
  --products bazel-bin/app/app.jar \
  -- bazel build //app:deployable

# Output: build.abc123.link (contains provenance)
```

## Verifying Provenance

### Verify Sigstore Signature

```bash
# Verify signature (requires transparency log entry)
cosign verify-blob \
  --signature=bazel-bin/app/app.provenance.json.sig \
  --certificate-identity=user@example.com \
  --certificate-oidc-issuer=https://github.com/login/oauth \
  bazel-bin/app/app.provenance.json
```

### Verify GPG Signature

```bash
# Import public key
gpg --import pubkey.asc

# Verify signature
gpg --verify bazel-bin/app/app.provenance.json.asc \
  bazel-bin/app/app.provenance.json
```

### Verify Provenance Contents

```bash
# Extract subject digest from provenance
EXPECTED_SHA=$(jq -r '.subject[0].digest.sha256' bazel-bin/app/app.provenance.json)

# Calculate actual artifact digest
ACTUAL_SHA=$(sha256sum bazel-bin/app/app.jar | awk '{print $1}')

# Compare
if [ "$EXPECTED_SHA" = "$ACTUAL_SHA" ]; then
  echo "✓ Provenance matches artifact"
else
  echo "✗ Provenance mismatch!"
  exit 1
fi
```

### Verify Build Metadata

```bash
# Check commit matches
jq -r '.predicate.buildDefinition.externalParameters.commit' \
  bazel-bin/app/app.provenance.json

# Check builder identity
jq -r '.predicate.runDetails.builder.id' \
  bazel-bin/app/app.provenance.json

# Check dependencies
jq -r '.predicate.buildDefinition.resolvedDependencies[] | .uri' \
  bazel-bin/app/app.provenance.json
```

## Attaching Provenance to Releases

### GitHub Releases

```bash
# Create release with provenance
gh release create v1.0.0 \
  bazel-bin/app/app.jar \
  bazel-bin/app/app.provenance.json \
  bazel-bin/app/app.provenance.json.sig \
  --notes "Release v1.0.0 with SLSA provenance"
```

In GitHub Actions:

```yaml
- name: Create release with provenance
  uses: softprops/action-gh-release@v1
  with:
    tag_name: ${{ github.ref_name }}
    files: |
      bazel-bin/**/*.jar
      bazel-bin/**/*.provenance.json
      bazel-bin/**/*.provenance.json.sig
```

### Container Images (OCI Annotations)

```bash
# Attach provenance as OCI annotation
cosign attach attestation \
  --predicate=bazel-bin/app/app.provenance.json \
  ghcr.io/cboyd0319/app:v1.0.0

# Verify attached attestation
cosign verify-attestation \
  --type=slsaprovenance \
  ghcr.io/cboyd0319/app:v1.0.0
```

## Integration with BazBOM Workflows

### Full Supply Chain Workflow

```yaml
name: Supply Chain with Provenance

on: [push, pull_request]

jobs:
  build-and-attest:
    runs-on: ubuntu-latest
    permissions:
      id-token: write  # For Sigstore
      contents: write  # For release upload

    steps:
      - uses: actions/checkout@v4

      # 1. Build artifacts
      - name: Build
        run: bazel build //app:deployable

      # 2. Generate SBOM
      - name: Generate SBOM
        run: bazel build //app:app_sbom

      # 3. Generate provenance
      - name: Generate provenance
        run: |
          bazel build //app:app_provenance \
            --define commit_sha=${{ github.sha }} \
            --define build_id=${{ github.run_id }}

      # 4. Sign provenance
      - uses: sigstore/cosign-installer@v3
      - name: Sign provenance
        env:
          COSIGN_EXPERIMENTAL: 1
        run: |
          cosign sign-blob --yes \
            bazel-bin/app/app.provenance.json \
            --output-signature=bazel-bin/app/app.provenance.json.sig

      # 5. Verify before release
      - name: Verify provenance
        run: |
          cosign verify-blob \
            --signature=bazel-bin/app/app.provenance.json.sig \
            bazel-bin/app/app.provenance.json

      # 6. Upload artifacts
      - name: Upload release artifacts
        if: startsWith(github.ref, 'refs/tags/')
        uses: softprops/action-gh-release@v1
        with:
          files: |
            bazel-bin/app/app.jar
            bazel-bin/app/app.sbom.spdx.json
            bazel-bin/app/app.provenance.json
            bazel-bin/app/app.provenance.json.sig
```

## Provenance Policy Enforcement

### Require Signed Provenance

```bash
#!/bin/bash
# verify-release.sh

ARTIFACT=$1
PROVENANCE="${ARTIFACT}.provenance.json"
SIGNATURE="${PROVENANCE}.sig"

# Check files exist
if [ ! -f "$PROVENANCE" ] || [ ! -f "$SIGNATURE" ]; then
  echo "✗ Missing provenance or signature"
  exit 1
fi

# Verify signature
cosign verify-blob --signature="$SIGNATURE" "$PROVENANCE" || exit 1

# Verify artifact matches provenance
EXPECTED=$(jq -r '.subject[0].digest.sha256' "$PROVENANCE")
ACTUAL=$(sha256sum "$ARTIFACT" | awk '{print $1}')

if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "✗ Artifact digest mismatch"
  exit 1
fi

echo "✓ Provenance verified"
```

### SLSA Verifier (Official Tool)

```bash
# Install SLSA verifier
go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@latest

# Verify artifact with provenance
slsa-verifier verify-artifact \
  --provenance-path=bazel-bin/app/app.provenance.json \
  --source-uri=github.com/cboyd0319/BazBOM \
  bazel-bin/app/app.jar
```

## Troubleshooting

### Issue: "No OIDC token found" during Sigstore signing

**Cause:** Running locally without OIDC provider.

**Fix:** Use `--key` for local signing or set up Fulcio locally:

```bash
# Generate local key pair
cosign generate-key-pair

# Sign with key
cosign sign-blob --key cosign.key \
  bazel-bin/app/app.provenance.json \
  --output-signature=bazel-bin/app/app.provenance.json.sig
```

### Issue: Provenance contains dynamic timestamps

**Cause:** Non-deterministic build metadata.

**Fix:** Use fixed timestamp for reproducibility:

```bash
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
bazel build //:provenance_all --define build_timestamp=$SOURCE_DATE_EPOCH
```

### Issue: Large provenance files (> 1MB)

**Cause:** Embedding entire dependency graph.

**Fix:** Use references instead of inline data:

```json
{
  "resolvedDependencies": [
    {
      "uri": "https://example.com/sbom/app.spdx.json#SPDXRef-Package-guava",
      "digest": {"sha256": "..."}
    }
  ]
}
```

## References

- [SLSA Specification](https://slsa.dev/spec/v1.0/)
- [Sigstore Documentation](https://docs.sigstore.dev/)
- [In-Toto Attestation Spec](https://github.com/in-toto/attestation)
- [SLSA Verifier](https://github.com/slsa-framework/slsa-verifier)
- [Cosign Examples](https://github.com/sigstore/cosign/tree/main/examples)
