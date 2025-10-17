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
      <td>SLSA 1</td>
      <td>Provenance exists</td>
      <td>✅ Implemented</td>
    </tr>
    <tr>
      <td>SLSA 2</td>
      <td>Signed provenance</td>
      <td>✅ Implemented (Sigstore)</td>
    </tr>
    <tr>
      <td>SLSA 3</td>
      <td>Hardened build platform</td>
      <td>✅ GitHub-hosted runners</td>
    </tr>
    <tr>
      <td>SLSA 4</td>
      <td>Two-party review</td>
      <td>🚧 Optional (via CODEOWNERS)</td>
    </tr>
  </tbody>
</table>

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

## SBOM Attestation & Transparency Logs

**Status:** ✅ Implemented (Phase 1)

BazBOM now provides comprehensive SBOM attestation with cryptographic signing and transparency logging using Sigstore.

### Features

1. **Keyless Signing** - Sign SBOMs without managing private keys (OIDC-based)
2. **Rekor Transparency Log** - Public audit trail of all signed SBOMs
3. **in-toto Attestations** - Industry-standard attestation bundles
4. **Public Verification** - Anyone can verify SBOM authenticity

### Quick Start

#### Sign an SBOM

```bash
# Sign with Sigstore (keyless, uses GitHub OIDC)
bazel run //tools/supplychain:sbom_signing -- sign \
  bazel-bin/app.spdx.json \
  --output-dir=bazel-bin/signatures

# Creates:
#   - app.sig (detached signature)
#   - app.bundle.json (signature bundle with Rekor entry)
```

#### Verify an SBOM

```bash
# Verify signature
bazel run //tools/supplychain:verify_sbom -- \
  bazel-bin/app.spdx.json \
  --bundle bazel-bin/signatures/app.bundle.json \
  --cert-identity "https://github.com/cboyd0319/BazBOM/.github/workflows/supplychain.yml@refs/heads/main" \
  --cert-oidc-issuer "https://token.actions.githubusercontent.com"

# Output:
# ✓ Signature verification PASSED
# ✓ Rekor entry verified
# ✓ Attestation structure valid
```

### Signing Workflow

#### 1. Generate SBOM
```bash
bazel build //:sbom_all
```

#### 2. Sign SBOM
```bash
# Sign with cosign (keyless)
python3 tools/supplychain/sbom_signing.py sign \
  bazel-bin/workspace.spdx.json \
  --output-dir=bazel-bin/signatures
```

**What happens:**
1. Cosign requests OIDC token (GitHub Actions provides automatically)
2. Ephemeral signing key generated
3. SBOM is signed
4. Signature logged to Rekor transparency log
5. Signing certificate issued by Fulcio CA
6. All artifacts bundled together

#### 3. Create Attestation
```bash
# Generate in-toto attestation
python3 tools/supplychain/intoto_attestation.py bundle \
  bazel-bin/workspace.spdx.json \
  --signature "$(cat bazel-bin/signatures/workspace.sig)" \
  --certificate bazel-bin/signatures/workspace.cert \
  --rekor-entry "https://rekor.sigstore.dev/api/v1/log/entries/abc123" \
  --output bazel-bin/attestations/workspace.attestation.json
```

### Rekor Transparency Log

All signed SBOMs are logged to Rekor, Sigstore's public transparency log.

#### Query Rekor Entry

```bash
# Get entry by UUID
python3 tools/supplychain/rekor_integration.py get abc123def456

# Search by SBOM hash
python3 tools/supplychain/rekor_integration.py search \
  $(sha256sum bazel-bin/workspace.spdx.json | cut -d' ' -f1)

# Verify entry inclusion
python3 tools/supplychain/rekor_integration.py verify abc123def456
```

#### Rekor Entry Structure

```json
{
  "uuid": "24296fb24b8ad77a8c90f96f3e82b1b77fef00a9f16f6f6f6f6f6f6f6f6f6f6f",
  "logIndex": 123456,
  "integratedTime": 1642000000,
  "body": {
    "kind": "hashedrekord",
    "spec": {
      "signature": {
        "content": "MEUCIQBase64SignatureHere...",
        "publicKey": {
          "content": "-----BEGIN CERTIFICATE-----\n..."
        }
      },
      "data": {
        "hash": {
          "algorithm": "sha256",
          "value": "a42edc9cab792e39fe39bb94f3fca655ed157ff87a8af78e1d6ba5b07c4a00ab"
        }
      }
    }
  }
}
```

### in-toto Attestation Bundles

Attestation bundles combine the SBOM, signature, certificate, and Rekor entry.

#### Bundle Structure

```json
{
  "attestation": {
    "_type": "https://in-toto.io/Statement/v1",
    "subject": [
      {
        "name": "workspace.spdx.json",
        "digest": {
          "sha256": "a42edc9cab792e39fe39bb94f3fca655..."
        }
      }
    ],
    "predicateType": "https://slsa.dev/provenance/v1",
    "predicate": {
      "buildDefinition": {
        "buildType": "https://github.com/cboyd0319/BazBOM/bazel-sbom-build@v1",
        "externalParameters": {
          "workflow": "SBOM Generation",
          "repository": "cboyd0319/BazBOM",
          "ref": "refs/heads/main"
        }
      },
      "runDetails": {
        "builder": {
          "id": "https://github.com/cboyd0319/BazBOM"
        },
        "metadata": {
          "invocationId": "123456789"
        }
      }
    }
  },
  "signature": {
    "keyid": "",
    "sig": "MEUCIQBase64Signature..."
  },
  "signing_cert": "-----BEGIN CERTIFICATE-----\n...",
  "rekor_entry": "https://rekor.sigstore.dev/api/v1/log/entries/abc123",
  "bundle_version": "1.0"
}
```

### Verification

#### Complete Verification

```bash
# Comprehensive verification (all steps)
python3 tools/supplychain/verify_sbom.py \
  bazel-bin/workspace.spdx.json \
  --bundle bazel-bin/signatures/workspace.bundle.json \
  --cert-identity "https://github.com/cboyd0319/BazBOM/.github/workflows/supplychain.yml@refs/heads/main" \
  --cert-oidc-issuer "https://token.actions.githubusercontent.com"

# Output:
============================================================
SBOM Verification Summary
============================================================
SBOM: bazel-bin/workspace.spdx.json
Overall Status: PASSED

Verification Steps:
  Signature:    ✓ PASS
  Rekor Log:    ✓ PASS
  Attestation:  ✓ PASS
============================================================
```

#### Verification Steps Explained

1. **Signature Verification**
   - Validates cryptographic signature using cosign
   - Verifies signing certificate chain
   - Checks certificate identity matches expected workflow
   - Ensures OIDC issuer is GitHub Actions

2. **Rekor Transparency Log Verification**
   - Confirms entry exists in Rekor
   - Validates Merkle tree inclusion proof
   - Verifies entry timestamp

3. **Attestation Structure Validation**
   - Checks in-toto statement format
   - Validates SLSA predicate structure
   - Ensures subject digest matches SBOM

#### Public Verification (Anyone)

Third parties can verify SBOMs without credentials:

```bash
# Download SBOM and bundle from GitHub Release
curl -L https://github.com/cboyd0319/BazBOM/releases/download/v1.0.0/workspace.spdx.json -o sbom.json
curl -L https://github.com/cboyd0319/BazBOM/releases/download/v1.0.0/workspace.bundle.json -o bundle.json

# Verify with cosign
cosign verify-blob sbom.json \
  --bundle bundle.json \
  --certificate-identity "https://github.com/cboyd0319/BazBOM/.github/workflows/supplychain.yml@refs/heads/main" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com"

# Verify Rekor entry
python3 verify_sbom.py sbom.json --bundle bundle.json
```

### GitHub Actions Integration

SBOMs are automatically signed in CI/CD:

```yaml
# .github/workflows/supplychain.yml
permissions:
  contents: read
  id-token: write  # Required for OIDC token (keyless signing)

jobs:
  sign-sboms:
    runs-on: ubuntu-latest
    steps:
      - name: Install Cosign
        uses: sigstore/cosign-installer@v3
      
      - name: Sign SBOMs
        env:
          COSIGN_EXPERIMENTAL: "1"
        run: |
          bazel run //tools/supplychain:sbom_signing -- sign \
            bazel-bin/**/*.spdx.json \
            --output-dir=bazel-bin/signatures
      
      - name: Upload to Release
        uses: actions/upload-artifact@v3
        with:
          name: signed-sboms
          path: |
            bazel-bin/**/*.spdx.json
            bazel-bin/signatures/**
```

### Security Properties

#### Keyless Signing Benefits

- **No private key management** - Ephemeral keys generated per signing
- **No key storage** - Keys destroyed after use
- **OIDC-based identity** - Cryptographically tied to GitHub workflow
- **Certificate transparency** - All certificates logged publicly
- **Automatic rotation** - New key for every signature

#### Transparency Log Benefits

- **Tamper-evident** - Merkle tree prevents backdating
- **Public audit trail** - Anyone can query and verify
- **Timestamp proof** - Rekor provides trusted timestamps
- **Immutable record** - Entries cannot be deleted or modified

#### Threat Mitigation

| Threat | Mitigation |
|--------|-----------|
| SBOM tampering | Signature verification fails |
| Backdated signatures | Rekor timestamp prevents |
| Key compromise | Keyless signing (ephemeral keys) |
| Certificate forgery | Fulcio CA + certificate transparency |
| Man-in-the-middle | End-to-end cryptographic verification |

### Troubleshooting

#### Cosign Not Found

```bash
# Install cosign
wget https://github.com/sigstore/cosign/releases/download/v2.2.0/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
```

#### OIDC Token Error

```bash
# In GitHub Actions, ensure id-token: write permission
permissions:
  id-token: write

# Locally, use interactive mode (opens browser)
export COSIGN_EXPERIMENTAL=1
cosign sign-blob sbom.json --yes
```

#### Signature Verification Failed

```bash
# Ensure correct certificate identity
# Format: https://github.com/OWNER/REPO/.github/workflows/WORKFLOW.yml@refs/heads/BRANCH

# Check actual identity in certificate
cosign verify-blob sbom.json --bundle bundle.json
```

#### Rekor Entry Not Found

```bash
# Extract UUID from bundle
jq -r '.rekor_entry' bundle.json

# Query Rekor directly
curl https://rekor.sigstore.dev/api/v1/log/entries/{UUID}
```

### API Reference

See tool help for complete API documentation:

```bash
# Signing
python3 tools/supplychain/sbom_signing.py --help

# Rekor
python3 tools/supplychain/rekor_integration.py --help

# Attestation
python3 tools/supplychain/intoto_attestation.py --help

# Verification
python3 tools/supplychain/verify_sbom.py --help
```

## References

- [SLSA Specification](https://slsa.dev/spec/v1.0/)
- [Sigstore Documentation](https://docs.sigstore.dev/)
- [In-Toto Attestation Spec](https://github.com/in-toto/attestation)
- [SLSA Verifier](https://github.com/slsa-framework/slsa-verifier)
- [Cosign Examples](https://github.com/sigstore/cosign/tree/main/examples)
- [Rekor API Documentation](https://docs.sigstore.dev/rekor/API/)
- [Fulcio Certificate Transparency](https://docs.sigstore.dev/fulcio/certificate-authority/)
