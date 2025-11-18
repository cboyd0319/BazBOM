# GitHub Dependency Snapshot Format

BazBOM's support for GitHub's dependency submission API format.

## Overview

**Format:** GitHub Dependency Snapshot (JSON)
**Spec:** <https://docs.github.com/en/rest/dependency-graph/dependency-submission>
**Output:** `github-snapshot.json`
**Status:** Production-ready

**Why GitHub Snapshot?**
- Direct integration with GitHub's Dependency Graph
- Automatic Dependabot alerts for detected vulnerabilities
- Security Insights and dependency review
- Required for GitHub Advanced Security features
- No external SBOM upload tools needed

## Generation

```bash
# Generate GitHub dependency snapshot
bazbom scan --format github-snapshot

# Output: github-snapshot.json
```

## Uploading to GitHub

### Manual Upload via API

```bash
# Generate snapshot
bazbom scan --format github-snapshot -o .

# Upload to GitHub
curl -X POST \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Content-Type: application/json" \
  --data @github-snapshot.json \
  https://api.github.com/repos/OWNER/REPO/dependency-graph/snapshots
```

### GitHub Actions Workflow

```yaml
name: Dependency Snapshot

on:
  push:
    branches: [ main ]

jobs:
  snapshot:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - name: Install BazBOM
        run: |
          curl -sSL https://bazbom.io/install.sh | sh

      - name: Generate dependency snapshot
        run: bazbom scan --format github-snapshot -o .

      - name: Submit dependency snapshot
        uses: advanced-security/maven-dependency-submission-action@v3
        with:
          snapshot-path: github-snapshot.json
```

## Format Structure

GitHub dependency snapshot format includes:

### Required Fields

| Field | Description | Example |
|-------|-------------|---------|
| `version` | Snapshot schema version | `0` |
| `sha` | Git commit SHA | `"32cd5e744497deb3d9c44f676d0105183a88fe68"` |
| `ref` | Git ref | `"refs/heads/main"` |
| `job.id` | Unique job identifier | `"5d50ce65-10e7-45aa-a943-fd72b23be554"` |
| `job.correlator` | Job correlation ID | `"bazbom_scan"` |
| `detector.name` | Tool name | `"BazBOM"` |
| `detector.version` | Tool version | `"6.5.0"` |
| `detector.url` | Tool URL | `"https://github.com/cboyd0319/BazBOM"` |
| `scanned` | Scan timestamp | `"2025-11-18T23:10:50.658289+00:00"` |
| `manifests` | Dependency manifests | See below |

### Manifest Structure

Each detected ecosystem creates a manifest entry:

```json
{
  "pom.xml": {
    "name": "pom.xml",
    "file": {
      "source_location": "pom.xml"
    },
    "metadata": {},
    "resolved": {
      "log4j-core@2.17.0": {
        "package_url": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0",
        "relationship": "direct",
        "scope": "runtime",
        "dependencies": [],
        "metadata": {}
      }
    }
  }
}
```

### Ecosystem Mapping

BazBOM maps detected ecosystems to manifest names:

| Ecosystem | Manifest Name | Package URL Format |
|-----------|---------------|-------------------|
| Maven | `pom.xml` | `pkg:maven/{group}/{artifact}@{version}` |
| Maven (Bazel) | `pom.xml` | `pkg:maven/{group}/{artifact}@{version}` |
| npm | `package-lock.json` | `pkg:npm/{name}@{version}` |
| Python | `requirements.txt` | `pkg:pypi/{name}@{version}` |
| Rust | `Cargo.lock` | `pkg:cargo/{name}@{version}` |
| Go | `go.mod` | `pkg:golang/{module}@{version}` |
| Ruby | `Gemfile.lock` | `pkg:gem/{name}@{version}` |
| PHP | `composer.lock` | `pkg:composer/{vendor}/{name}@{version}` |

## Complete Example

```json
{
  "version": 0,
  "sha": "32cd5e744497deb3d9c44f676d0105183a88fe68",
  "ref": "refs/heads/main",
  "job": {
    "id": "5d50ce65-10e7-45aa-a943-fd72b23be554",
    "correlator": "bazbom_scan"
  },
  "detector": {
    "name": "BazBOM",
    "version": "6.5.0",
    "url": "https://github.com/cboyd0319/BazBOM"
  },
  "scanned": "2025-11-18T23:10:50.658289+00:00",
  "metadata": {},
  "manifests": {
    "pom.xml": {
      "name": "pom.xml",
      "file": {
        "source_location": "pom.xml"
      },
      "metadata": {},
      "resolved": {
        "grpc-all@1.51.1": {
          "package_url": "pkg:maven/io.grpc/grpc-all@1.51.1",
          "relationship": "direct",
          "scope": "runtime",
          "dependencies": [],
          "metadata": {}
        },
        "guava@31.1-android": {
          "package_url": "pkg:maven/com.google.guava/guava@31.1-android",
          "relationship": "direct",
          "scope": "runtime",
          "dependencies": [],
          "metadata": {}
        }
      }
    },
    "package-lock.json": {
      "name": "package-lock.json",
      "file": {
        "source_location": "package-lock.json"
      },
      "metadata": {},
      "resolved": {
        "express@4.18.0": {
          "package_url": "pkg:npm/express@4.18.0",
          "relationship": "direct",
          "scope": "runtime",
          "dependencies": [],
          "metadata": {}
        }
      }
    }
  }
}
```

## Benefits

### Automatic Security Alerts

Once uploaded, GitHub automatically:
- Detects vulnerable dependencies
- Creates Dependabot alerts
- Suggests version updates
- Integrates with Security Insights dashboard

### Dependency Review

GitHub shows:
- Full dependency graph visualization
- License compliance information
- Dependency tree with direct/transitive relationships
- Historical dependency changes

### Integration with GitHub Advanced Security

- Works with secret scanning
- Correlates with code scanning results
- Enables security policy enforcement
- Provides executive reporting

## Limitations

### Current Limitations

| Feature | Status | Notes |
|---------|--------|-------|
| Dependency graph | Basic | Direct dependencies only (no full transitive tree) |
| Scopes | Limited | Only `runtime` scope currently |
| Metadata | Minimal | Future: license info, vulnerability counts |

### Future Enhancements

- Full transitive dependency tree
- License information per package
- Multiple scopes (dev, test, build)
- Vulnerability pre-filtering
- SBOM attestation signing

## Differences from SPDX/CycloneDX

| Feature | GitHub Snapshot | SPDX 2.3 | CycloneDX 1.5 |
|---------|----------------|----------|---------------|
| Purpose | GitHub integration | Universal SBOM | Universal SBOM |
| Checksums | Not included | Optional | Optional |
| Licenses | Not included | Full support | Full support |
| Vulnerability data | GitHub provides | External | Inline optional |
| Dependency tree | Flat | Full graph | Full graph |
| File-level analysis | No | Optional | Optional |

**When to use GitHub snapshot:**
- GitHub-hosted repositories
- Want automatic Dependabot alerts
- Using GitHub Advanced Security
- Need dependency graph visualization

**When to use SPDX/CycloneDX:**
- Compliance requirements (NTIA, etc.)
- Multi-platform SBOM sharing
- Detailed dependency graphs
- License compliance tracking
- Non-GitHub repositories

## Validation

GitHub validates snapshots on upload. Common errors:

| Error | Cause | Fix |
|-------|-------|-----|
| Invalid SHA | SHA doesn't match repo | Check git SHA is pushed |
| Invalid ref | Ref doesn't exist | Use `refs/heads/branch` format |
| Missing required fields | Incomplete snapshot | Update BazBOM version |
| Invalid PURL | Malformed package URL | Report BazBOM bug |

## Next Steps

- [SPDX format](../FORMAT_SPDX.md) - Comprehensive SBOM format
- [CycloneDX format](cyclonedx.md) - Alternative SBOM format
- [GitHub Actions integration](../CI.md) - CI/CD workflows
- [Dependency-Track](../INTEGRATIONS.md#dependency-track) - External SBOM management
