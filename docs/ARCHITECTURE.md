# Architecture

This document describes the high-level architecture of BazBOM and how its components interact.

## Overview

BazBOM is a memory-safe, Rust-first JVM supply chain security toolkit that generates Software Bill of Materials (SBOM) and performs Software Composition Analysis (SCA) for Maven, Gradle, and Bazel projects. The architecture emphasizes:

- **Memory Safety**: Rust-first implementation with managed JVM helpers for reachability
- **Zero Telemetry**: No background network calls; explicit offline sync
- **Offline-First**: Advisory databases cached locally with deterministic updates
- **Build-Native Accuracy**: Integration at build-time for authoritative dependency graphs

## Architecture Diagram

```mermaid
graph TD
    A[bazbom CLI (Rust)] -->|Detect| B[Build System Detection]
    B -->|Maven| C1[Maven Plugin]
    B -->|Gradle| C2[Gradle Plugin]
    B -->|Bazel| C3[Bazel Aspects]
    
    C1 --> D[Normalized Graph]
    C2 --> D
    C3 --> D
    
    D --> E[Enrichment Engine]
    E -->|Local Cache| F[Advisory DB Sync]
    F --> G[OSV/NVD/GHSA/KEV/EPSS]
    
    E --> H[Policy Engine]
    H --> I[SBOM Exporters]
    I --> J1[SPDX 2.3]
    I --> J2[CycloneDX 1.5]
    I --> J3[SARIF 2.1.0]
    
    E -->|Optional| K[Reachability OPAL]
    K -->|Bytecode| L[Call Graph]
    L --> E
    
    style A fill:#e67e22
    style D fill:#3498db
    style J1 fill:#2ecc71
    style J2 fill:#2ecc71
    style J3 fill:#2ecc71
```

For the Mermaid source, see [diagrams/architecture.mmd](diagrams/architecture.mmd).

## Core Components

### 1. Rust CLI (`bazbom`)

**Location**: `crates/bazbom/`

The primary user interface and orchestration layer:

- **Commands**: `scan`, `policy check`, `fix`, `db sync`
- **Build System Detection**: Auto-detects Maven, Gradle, or Bazel
- **Output Management**: Generates SBOM, findings, and SARIF
- **Zero Config**: Works out-of-the-box with sensible defaults

**Key Features**:
- Single binary distribution (no Python runtime required)
- Memory-safe implementation (no unsafe blocks)
- Cross-platform support (macOS, Linux, Windows)
- Signed releases with SLSA provenance

### 2. Build System Integration Layer

**Location**: 
- Maven: `crates/bazbom-maven/` (planned)
- Gradle: `crates/bazbom-gradle/` (planned)
- Bazel: `tools/supplychain/aspects.bzl` (existing)

#### Maven Plugin (Planned)
- Emits authoritative JSON including effective POM, BOM resolution, scopes, conflicts
- Captures shading/relocation mappings from maven-shade-plugin
- Per-module and reactor-aggregate modes

#### Gradle Plugin (Planned)
- Per-configuration/variant graphs using Gradle's resolution API
- Android support (flavors, build types)
- Shadow plugin detection and mapping

#### Bazel Aspects (Existing, Being Enhanced)
- Traverses build graph for `java_*`, Kotlin, and broader JVM rules
- bzlmod + rules_jvm_external integration
- Incremental analysis using target diffs

### 3. Dependency Graph Normalization

**Location**: `crates/bazbom-graph/`

Converts build-system-specific outputs into a canonical graph model:

- **Input**: Build system JSON (Maven/Gradle/Bazel)
- **Process**: Normalizes coordinates, deduplicates, resolves conflicts
- **Output**: Unified dependency graph with PURLs and metadata

**Key Features**:
- Build-system agnostic internal representation
- Conflict resolution tracking
- Scope/configuration fidelity

### 4. Advisory Intelligence & Enrichment

**Location**: `crates/bazbom-advisories/`

Manages vulnerability data from multiple sources:

- **Sources**: OSV, NVD, GHSA, CISA KEV, EPSS
- **Merge Engine**: Deduplicates CVE/GHSA/OSV identifiers
- **Enrichment**: Adds KEV presence, EPSS probability, canonical severity
- **Priority Scoring**: Computes P0–P4 based on severity, EPSS, KEV, reachability

**Offline Sync (`bazbom db sync`)**:
```
.bazbom/cache/
├── advisories/
│   ├── osv.json
│   ├── nvd.json
│   ├── ghsa.json
│   ├── kev.json
│   └── epss.csv
└── manifest.json (blake3 hashes)
```

### 5. Reachability Analysis (OPAL-based)

**Location**: `bazbom-reachability.jar` (JVM helper, invoked via CLI)

Optional bytecode-level call graph analysis:

- **Input**: Compiled classes + runtime classpath
- **Engine**: OPAL (Scala-based static analysis framework)
- **Output**: Reachable/unreachable tags + method-level traces
- **Integration**: Invoked by Rust CLI via `java -jar` when `--reachability` flag set

**Performance**:
- Cached call graphs per module/target
- Scoped analysis to avoid full workspace scans

### 6. Policy Engine

**Location**: `crates/bazbom-policy/`

Enforces security and compliance policies:

- **Policy Language**: YAML (core), optional Rego/CUE
- **Rules**: Severity thresholds, license allow/deny, KEV/EPSS gates, reachability requirements
- **VEX Workflow**: Auto-generate VEX statements for unreachable findings (when policy allows)
- **CI Integration**: Exit codes for gating, PR comments, SARIF annotations

### 7. SBOM & Report Exporters

**Location**: `crates/bazbom-formats/`

Standards-compliant output generation:

- **SPDX 2.3 JSON** (primary format)
- **CycloneDX 1.5** (optional)
- **SARIF 2.1.0** (GitHub Code Scanning integration)
- **CSV** (business/compliance reports)
- **CSAF VEX** (false positive suppression)

**Validation**:
- JSON Schema validation for all formats
- Golden file tests for determinism

## Legacy Components (Transition Phase)

### Python-Based Supply Chain Tools

**Location**: `tools/supplychain/`

Existing Python implementation used during Rust migration:

- Bazel aspect integration (`aspects.bzl`)
- Dependency extraction (`extract_maven_deps.py`)
- SBOM generation (`write_sbom.py`)
- SCA scanning (`osv_query.py`, `ghsa_enrichment.py`)
- SARIF adapters (`sarif_adapter.py`)

**Migration Status**: 
- Rust CLI is primary distribution
- Python tools remain for:
  - Bazel integration (until Rust Bazel rules mature)
  - Dev utilities and prototyping
  - Backwards compatibility during transition

## Data Flow

### End-to-End Scan Flow

```
User: bazbom scan .
  ↓
1. Build System Detection (Maven/Gradle/Bazel)
  ↓
2. Build-Native Graph Extraction
   - Maven: effective POM + dependency:tree
   - Gradle: configurations API
   - Bazel: aspects traversal
  ↓
3. Graph Normalization (canonical model)
  ↓
4. Advisory Enrichment (local cache)
   - OSV/NVD/GHSA merge
   - KEV + EPSS tagging
   - Canonical severity
  ↓
5. [Optional] Reachability Analysis (OPAL)
   - Bytecode call graph
   - Tag reachable/unreachable
  ↓
6. Policy Checks
   - Apply rules
   - Generate VEX (if applicable)
  ↓
7. Export
   - sbom.spdx.json
   - sca_findings.json
   - sca_findings.sarif
```

## Security Architecture

### Supply Chain Guarantees

- **SLSA Level 3 Provenance**: Build provenance for all releases
- **Sigstore Signing**: Keyless signing with transparency log
- **Hermetic Builds**: No network access during scan (use local cache)
- **Deterministic Outputs**: Same inputs → same outputs (bit-for-bit)

### Privacy Model

- **Zero Telemetry**: No analytics, tracking, or phoning home
- **Explicit Sync**: Advisory updates only via `bazbom db sync` command
- **Offline Operation**: Full functionality without network (after initial sync)
- **Local-Only Data**: All caches and artifacts remain on user's machine

## Build System-Specific Details
    {"spdxElementId": "root", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "guava"},
    {"spdxElementId": "guava", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "failureaccess"}
  ]
}
```

**Enhanced Features**:
- SHA256 checksums for all packages
- Package URLs (PURLs) for ecosystem identification
- Proper transitive relationships (not just root dependencies)
- Direct/transitive dependency distinction

### 4. Vulnerability Scanner (SCA)

**Location**: `tools/supplychain/osv_query.py`

Queries the OSV (Open Source Vulnerabilities) database:

- **Input**: SPDX SBOM files
- **Process**: Queries OSV API for each package
- **Output**: SARIF vulnerability reports

**OSV Integration**:
- Queries by package name and version
- Supports ecosystem-specific queries (Maven, npm, PyPI, etc.)
- Rate-limited and cached for efficiency
- Severity mapping (Critical, High, Medium, Low)

### 4. SARIF Report Generation

**Location**: `tools/supplychain/sarif_adapter.py`

Converts vulnerability data to SARIF format for GitHub Code Scanning:

- **Input**: OSV vulnerability data (enriched or unenriched)
- **Process**: Formats as SARIF 2.1.0 with enriched context
- **Output**: `.sarif.json` files compatible with GitHub

**SARIF Benefits**:
- Native GitHub Code Scanning integration
- Rich security alert UI with KEV/EPSS context
- PR annotations for new vulnerabilities
- Trend tracking over time

### 5. RipGrep-Accelerated Discovery (Optional)

**Location**: `tools/supplychain/dependency_scanner.py`, `license_scanner.py`, `container_scanner.py`, etc.

BazBOM optionally leverages [RipGrep](https://github.com/BurntSushi/ripgrep) for 100-1000x faster file discovery and pattern matching in large monorepos.

**Key Features:**
- **Fast Dependency Discovery**: Find BUILD files, pom.xml, and dependency references 100x faster
- **License Header Scanning**: Scan 10,000+ source files for license headers in ~2 seconds
- **Incremental Analysis**: Quickly identify changed targets in PRs (6.25x speedup)
- **Container Scanning**: Fast JAR and OS package discovery in container images (10.9x faster)
- **CVE Tracking**: Find CVE references in code, comments, and VEX statements
- **Dependency Verification**: Detect unused or undeclared dependencies

**Architecture:**

```mermaid
graph TD
    A[RipGrep rg] -->|Fast Pattern Matching| B[Build Files]
    A -->|License Headers| C[Source Files]
    A -->|JAR Discovery| D[Container Layers]
    A -->|CVE References| E[Code & Docs]
    
    B --> F[Dependency Scanner]
    C --> G[License Scanner]
    D --> H[Container Scanner]
    E --> I[CVE Tracker]
    
    F --> J[maven_install.json Verification]
    G --> K[License Compliance Report]
    H --> L[Container SBOM]
    I --> M[VEX Cross-Reference]
    
    style A fill:#90EE90
    style F fill:#87CEEB
    style G fill:#87CEEB
    style H fill:#87CEEB
    style I fill:#87CEEB
```

**Integration Points:**

1. **Dependency Scanner** (`dependency_scanner.py`)
   - `find_maven_dependencies()` - Extract Maven deps from pom.xml files
   - `find_gradle_dependencies()` - Extract Gradle deps from build files
   - `find_bazel_maven_jars()` - Find @maven// references in BUILD files

2. **Incremental Analyzer** (`incremental_analyzer.py`)
   - `get_changed_build_files_fast()` - Filter changed BUILD files (RipGrep)
   - `find_affected_targets_fast()` - Find targets in changed packages

3. **License Scanner** (`license_scanner.py`)
   - `scan_license_headers()` - Find license headers by pattern (Apache, MIT, GPL, etc.)
   - `find_unlicensed_files()` - Identify files without license headers
   - `check_copyleft_licenses()` - Flag GPL/LGPL dependencies

4. **Container Scanner** (`container_scanner.py`)
   - `extract_jars_from_image()` - Find JAR files in container layers
   - `find_os_packages()` - Locate dpkg/rpm/apk manifests

5. **Dependency Verifier** (`dependency_verifier.py`)
   - `find_unused_dependencies()` - Detect deps in lockfile but not referenced
   - `find_undeclared_dependencies()` - Detect deps referenced but not declared

6. **CVE Tracker** (`cve_tracker.py`)
   - `find_cve_references()` - Search for CVE-YYYY-NNNN patterns
   - `find_vex_statements()` - Locate VEX files with CVE references
   - `cross_reference_with_sbom()` - Compare code CVEs with SBOM findings

**Performance Benchmarks (5000-target monorepo):**

| Task | Traditional Method | RipGrep Method | Speedup |
|------|-------------------|----------------|---------|
| Find BUILD files | 12.3s | 0.09s | **136x** |
| Find @maven// refs | 8.7s | 0.14s | **62x** |
| License header scan (10K files) | 34s | 1.8s | **18.9x** |
| Incremental PR analysis | 45s | 7.2s | **6.25x** |
| Container JAR discovery | 23s | 2.1s | **10.9x** |

**Graceful Degradation:**

All RipGrep-accelerated tools check for availability and provide helpful error messages when RipGrep is not installed. The system falls back to standard methods or clearly indicates the missing dependency.

```python
def check_ripgrep_available() -> bool:
    """Check if RipGrep is installed and available."""
    try:
        subprocess.run(['rg', '--version'], ...)
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False
```

**CLI Integration:**

The BazBOM CLI exposes all RipGrep-accelerated features:

```bash
# Fast dependency discovery
bazbom scan . --fast-discovery

# License compliance scanning
bazbom license-report --output licenses.csv

# Container image scanning
bazbom scan-container myapp:latest --output sbom.json

# Dependency verification
bazbom verify --check-unused

# CVE reference tracking
bazbom find-cves --output cves.json
```

See [RIPGREP_INTEGRATION.md](RIPGREP_INTEGRATION.md) for complete integration details.

### 6. Vulnerability Enrichment Pipeline

**Location**: `tools/supplychain/vulnerability_enrichment.py`

Multi-source enrichment pipeline that enhances vulnerability findings with actionable intelligence:

#### Enrichment Modules

**KEV Enrichment** (`kev_enrichment.py`):
- **Source**: CISA Known Exploited Vulnerabilities Catalog
- **Update Frequency**: Daily
- **Caching**: 24-hour TTL
- **Output**: KEV status, due dates, required actions
- **Impact**: CVEs in KEV → P0-IMMEDIATE priority

**EPSS Enrichment** (`epss_enrichment.py`):
- **Source**: FIRST.org Exploit Prediction Scoring System
- **Model**: Machine learning-based probability (0-100%)
- **Batch Support**: 100 CVEs per API call
- **Caching**: 24-hour TTL
- **Output**: Exploitation probability, percentile ranking
- **Impact**: High EPSS → Higher risk score

**GHSA Enrichment** (`ghsa_enrichment.py`):
- **Source**: GitHub Security Advisories
- **API**: GraphQL (requires token for higher rate limits)
- **Coverage**: Maven, npm, PyPI, RubyGems, NuGet, Rust, Go
- **Output**: Remediation guidance, patched versions, vulnerable ranges
- **Impact**: Provides actionable fix information

**VulnCheck Enrichment** (`vulncheck_enrichment.py`, optional):
- **Source**: VulnCheck API
- **API Key**: Required (free tier: 100 req/day)
- **Output**: Exploit maturity, weaponization status, attack vectors
- **Impact**: Weaponized exploits → P1-CRITICAL priority

#### Risk Scoring Algorithm

Composite risk score calculation (0-100):

```python
Risk Score = (CVSS × 0.40) + (EPSS × 0.30) + (KEV × 0.20) + (Exploit × 0.10)
```

**Component Weights**:
- **CVSS (40%)**: Base severity score from NVD/OSV
- **EPSS (30%)**: Exploitation probability from FIRST.org
- **KEV (20%)**: Active exploitation status from CISA
- **Exploit (10%)**: Public exploit availability

#### Priority Mapping

| Priority | Criteria | Risk Score | Action Timeline |
|----------|----------|------------|-----------------|
| P0-IMMEDIATE | In CISA KEV | Any | Fix immediately |
| P1-CRITICAL | High risk | ≥ 80 | Fix this week |
| P2-HIGH | Medium-high risk | ≥ 60 | Fix this sprint |
| P3-MEDIUM | Medium risk | ≥ 40 | Fix next quarter |
| P4-LOW | Low risk | < 40 | Backlog |

#### Architecture Diagram

```mermaid
graph TD
    subgraph VulnEnrich["Vulnerability Enrichment Pipeline"]
        KEV["KEV<br/>(CISA API)"]
        EPSS["EPSS<br/>(FIRST.org)"]
        GHSA["GHSA<br/>(GitHub API)"]
        VulnCheck["VulnCheck<br/>(Optional)"]
        
        KEV --> Enricher
        EPSS --> Enricher
        GHSA --> Enricher
        VulnCheck --> Enricher
        
        Enricher["Vulnerability Enricher<br/>- Risk Scoring<br/>- Priority Mapping<br/>- Data Normalization"]
        
        Enricher --> Findings["Enriched Findings<br/>- Risk Score 0-100<br/>- Priority P0-P4<br/>- KEV Context<br/>- EPSS Probability<br/>- Exploit Status<br/>- GHSA Remediation"]
    end
    
    style KEV fill:#e74c3c
    style EPSS fill:#3498db
    style GHSA fill:#2ecc71
    style VulnCheck fill:#95a5a6
    style Enricher fill:#f39c12
    style Findings fill:#9b59b6
```

#### Performance Considerations

- **Parallel Enrichment**: All sources queried concurrently
- **Batch Processing**: EPSS supports 100 CVEs per request
- **Caching**: KEV and EPSS cached for 24 hours
- **Graceful Degradation**: Continues if enrichment sources fail
- **Rate Limiting**: Respects API rate limits (GHSA: 60/hr unauthenticated, 5000/hr with token)

## Data Flow

### SBOM Generation Flow

```
1. Developer runs: bazel build //:sbom_all
2. Bazel evaluates build graph
3. Aspect attaches to each target
4. Aspect collects dependency info
5. write_sbom.py generates SPDX files
6. SPDX files written to bazel-bin/
```

### SCA Flow (with Enrichment)

```
1. Developer runs: bazel run //:sca_from_sbom
2. Script discovers all .spdx.json files
3. For each SBOM:
   a. Extract package list
   b. Query OSV database
   c. Collect vulnerabilities
4. Enrichment pipeline (if enabled):
   a. Fetch EPSS scores (batch, 100 CVEs/request)
   b. Check CISA KEV catalog
   c. Query GitHub Security Advisories
   d. Query VulnCheck API (optional)
   e. Calculate risk scores
   f. Assign priorities (P0-P4)
5. sarif_adapter.py formats enriched results
6. SARIF files with KEV/EPSS context written to bazel-bin/
7. Priority summary printed to console
8. (Optional) Upload to GitHub Code Scanning
```

## Build Graph Integration

BazBOM integrates with Bazel's build graph:

```
Target
  ├─ Aspect: sbom_aspect
  │   ├─ Collects deps
  │   └─ Propagates to dependencies
  └─ Output: target.sbom
```

Aspects automatically traverse dependencies without explicit configuration.

## Security Model

### Threat Boundaries

- **Build Environment**: Isolated Bazel sandbox
- **Network Boundary**: OSV API queries (HTTPS only)
- **Data Boundary**: SBOMs contain public dependency info only

### Controls

- **Pinned Dependencies**: All tools and rules pinned to specific versions
- **Minimal Permissions**: CI workflows use read-only tokens where possible
- **Input Validation**: All external data validated before processing
- **Audit Trail**: All operations logged; SBOMs include generation metadata

## Extension Points

### Custom Aspects

Add custom dependency collectors:

```python
# tools/supplychain/custom_aspect.bzl
def _custom_sbom_aspect_impl(target, ctx):
    # Custom dependency collection logic
    pass
```

### Custom Formatters

Support additional SBOM formats:

```python
# tools/supplychain/write_cyclonedx.py
# Generate CycloneDX instead of SPDX
```

### Custom Scanners

Integrate with other vulnerability databases:

```python
# tools/supplychain/snyk_query.py
# Query Snyk instead of OSV
```

## Performance Characteristics

- **SBOM Generation**: O(n) where n = number of dependencies
- **Aspect Traversal**: Cached by Bazel; incremental builds are fast
- **OSV Queries**: Rate-limited; ~100 packages/minute
- **SARIF Generation**: O(v) where v = number of vulnerabilities

## Dependencies

### Build-time
- `rules_jvm_external` - Maven dependency management
- `rules_python` - Python tool execution

### Runtime (Python)
- `requests` - HTTP client for OSV queries
- `jsonschema` - SPDX/SARIF validation

## Future Enhancements

See [ADR/](ADR/) for architectural decisions and future directions:

- SPDX 3.0 support
- CycloneDX format support
- Additional vulnerability databases (Snyk, Grype)
- License compliance checking
- Dependency update automation
