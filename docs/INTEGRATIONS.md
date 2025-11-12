# Integrations

Connect BazBOM with external tools and platforms for end-to-end supply chain security.

## Overview

BazBOM integrates with:
- **Scanning tools** - Syft, OSV-Scanner, Semgrep, CodeQL
- **Platforms** - Dependency-Track, GUAC, GitHub Security
- **IDEs** - IntelliJ IDEA, VS Code
- **Container registries** - Docker Hub, ECR, GCR, ACR
- **CI systems** - GitHub Actions, GitLab CI, Jenkins, CircleCI

## Syft (Container Scanning)

**Use case:** Scan Docker/OCI images for non-JVM dependencies

**Integration:**

```bash
# BazBOM for JVM deps (build-time)
bazbom scan .

# Syft for container layers (runtime)
syft myapp:latest -o spdx-json > container-sbom.json

# Merge SBOMs (manual or tooling)
# Result: Complete SBOM (JVM + OS packages + Python/Node)
```

**Why combine?**
- BazBOM: Accurate JVM dependency graph from build
- Syft: OS packages, Python/Node modules in image

**Full guide:** See [Container Scanning Guide](features/container-scanning.md)

## OSV-Scanner

**Use case:** Cross-check vulnerability findings

**Integration:**

```bash
# Generate SBOM with BazBOM
bazbom scan . --format spdx

# Scan SBOM with OSV
osv-scanner --sbom sbom.spdx.json

# Compare findings
diff sca_findings.json osv_findings.json
```

**Why:** Validate BazBOM's CVE detection against another tool

## Dependency-Track

**Use case:** Centralized SBOM management and risk tracking

**Integration:**

```bash
# Upload SBOM to Dependency-Track
curl -X POST "$DTRACK_URL/api/v1/bom" \
  -H "X-Api-Key: $DTRACK_API_KEY" \
  -F "project=myapp" \
  -F "bom=@sbom.cyclonedx.json"
```

**Configuration:**

```yaml
# bazbom.yml
integrations:
  dependency_track:
    url: https://dtrack.example.com
    api_key: ${DTRACK_API_KEY}
    project: myapp
    auto_upload: true
```

**GitHub Actions:**

```yaml
- name: Upload to Dependency-Track
  run: |
    bazbom scan . --format cyclonedx
    curl -X POST "${{ secrets.DTRACK_URL }}/api/v1/bom" \
      -H "X-Api-Key: ${{ secrets.DTRACK_API_KEY }}" \
      -F "project=myapp" \
      -F "bom=@sbom.cyclonedx.json"
```

**Why:** Track vulnerabilities across projects and time

## GUAC

**Use case:** Supply chain graph analysis

**Integration:**

```bash
# Generate SBOM
bazbom scan .

# Ingest into GUAC
guacone collect files sbom.spdx.json

# Query GUAC
guacone query path --path-subject pkg:maven/log4j-core@2.17.0
```

**Recipe:** TBD - requires GUAC deployment

**Why:** Cross-repository supply chain insights

## GitHub Security (Code Scanning)

**Use case:** Native GitHub vulnerability alerts

**Integration:**

```yaml
# .github/workflows/security.yml
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: sca_findings.sarif
```

**Result:**
- Alerts in **Security** tab
- PR comments with findings
- Code scanning API access

**Configuration:**

```yaml
permissions:
  contents: read
  security-events: write
```

**Why:** Native GitHub integration, no external platform needed

## Semgrep (Static Analysis)

**Use case:** Combine SCA with SAST

**Integration:**

```bash
# Run both tools
bazbom scan . --with-semgrep

# Output: Merged SARIF with SCA + SAST findings
```

**Configuration:**

```toml
# bazbom.toml
[integrations.semgrep]
enabled = true
ruleset = "p/security-audit"
```

**Curated rules:** 10 high-impact JVM security rules (no noise)

**Why:** Single SARIF for GitHub Security tab

**Details:** [orchestrated-scan.md](integrations/orchestrated-scan.md)

## CodeQL (Deep SAST)

**Use case:** Enterprise-grade static analysis

**Integration:**

```bash
# Run CodeQL + BazBOM
bazbom scan . --with-codeql=security-extended

# Output: Merged SARIF
```

**Configuration:**

```toml
# bazbom.toml
[integrations.codeql]
enabled = false  # Expensive, use on main branch only
query_suite = "security-extended"
```

**GitHub Actions:**

```yaml
- name: CodeQL + BazBOM Scan
  run: bazbom scan . --with-codeql=default
```

**Why:** Catch vulnerabilities missed by SCA alone

**Details:** [orchestrated-scan.md](integrations/orchestrated-scan.md)

## IntelliJ IDEA Plugin

**Use case:** Real-time vulnerability warnings in IDE

**Installation:**

1. Open IntelliJ IDEA
2. Go to **Settings → Plugins → Marketplace**
3. Search "BazBOM"
4. Click **Install**
5. Restart IDE

**Features:**
- Real-time dependency scanning
- Inline vulnerability warnings
- One-click fix suggestions
- Policy violation alerts

**Status:** Code complete, marketplace submission pending

**Details:** [ide-integration.md](integrations/ide/ide-integration.md)

## VS Code Extension

**Use case:** Lightweight IDE integration via LSP

**Installation:**

1. Open VS Code
2. Go to **Extensions** (Ctrl+Shift+X)
3. Search "BazBOM"
4. Click **Install**

**Features:**
- Diagnostics in Problems panel
- Code actions for fixes
- Commands palette integration
- LSP-based (reusable across editors)

**Status:** Code complete, marketplace submission pending

**Details:** [ide-integration.md](integrations/ide/ide-integration.md)

## LLM Integration (Experimental)

**Use case:** AI-assisted vulnerability analysis

**Integration:**

```bash
# Generate AI-friendly report
bazbom scan . --llm-report > findings.txt

# Feed to LLM
cat findings.txt | llm "Explain these vulnerabilities and suggest fixes"
```

**Features:**
- Markdown-formatted findings
- Context-rich explanations
- Fix suggestions with rationale

**Status:** Experimental

**Details:** [llm-integration.md](integrations/llm-integration.md)

## Slack / Teams / Email Notifications

**Use case:** Alert team on critical vulnerabilities

**GitHub Actions:**

```yaml
- name: Check for Critical CVEs
  id: scan
  run: |
    bazbom scan . --format json
    CRITICAL_COUNT=$(jq '.vulnerabilities | map(select(.severity == "CRITICAL")) | length' sca_findings.json)
    echo "critical_count=$CRITICAL_COUNT" >> $GITHUB_OUTPUT

- name: Notify Slack
  if: steps.scan.outputs.critical_count > 0
  uses: slackapi/slack-github-action@v1
  with:
    payload: |
      {
        "text": "Critical vulnerabilities found in ${{ github.repository }}!",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "*Critical CVEs:* ${{ steps.scan.outputs.critical_count }}"
            }
          }
        ]
      }
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

**Recipe:** Adapt for Teams, email, PagerDuty, etc.

## Container Registries

**Use case:** Attach SBOMs to container images

**Docker Hub:**

```bash
# Build image
docker build -t myapp:latest .

# Generate SBOM
bazbom scan . --format spdx

# Attach as OCI artifact (requires ORAS)
oras attach myapp:latest \
  --artifact-type application/spdx+json \
  sbom.spdx.json
```

**ECR/GCR/ACR:**

Similar process with registry-specific tooling

**Why:** SBOMs travel with images

## CI/CD Platforms

See [CI.md](CI.md) for complete workflows:
- GitHub Actions
- GitLab CI
- Jenkins
- CircleCI

## Custom Integrations

**Webhook:**

```bash
# Send SBOM to webhook endpoint
curl -X POST https://api.example.com/sbom \
  -H "Content-Type: application/json" \
  -d @sbom.spdx.json
```

**REST API:**

```python
# Python example
import requests

response = requests.post(
    "https://api.example.com/sbom",
    json={"project": "myapp", "sbom": sbom_data},
    headers={"Authorization": f"Bearer {api_token}"}
)
```

**Configuration:**

```toml
# bazbom.toml
[integrations.webhook]
url = "https://api.example.com/sbom"
headers = { "Authorization" = "Bearer ${API_TOKEN}" }
on_scan = true
```

## Integration Matrix

| Tool | Purpose | Status | Docs |
|------|---------|--------|------|
| Syft | Container scanning | Guide | [container-scanning.md](features/container-scanning.md) |
| OSV-Scanner | CVE validation | Recipe | — |
| Dependency-Track | SBOM management | Recipe | Above |
| GUAC | Supply chain graph | Stub | Above |
| GitHub Security | Native alerts | Production | [CI.md](CI.md) |
| Semgrep | SAST | Production | [orchestrated-scan.md](integrations/orchestrated-scan.md) |
| CodeQL | Deep SAST | Production | [orchestrated-scan.md](integrations/orchestrated-scan.md) |
| IntelliJ IDEA | IDE | Beta | [ide-integration.md](integrations/ide/ide-integration.md) |
| VS Code | IDE | Beta | [ide-integration.md](integrations/ide/ide-integration.md) |
| LLM | AI analysis | Experimental | [llm-integration.md](integrations/llm-integration.md) |

## Next Steps

- [Container scanning guide](features/container-scanning.md)
- [IDE setup](integrations/ide/ide-integration.md)
- [Orchestrated scanning](integrations/orchestrated-scan.md)
- [CI integration](CI.md)
