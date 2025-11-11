# Architecture

BazBOM is a memory-safe, Rust-first JVM supply chain security tool that produces accurate SBOMs for Maven, Gradle, and Bazel projects.

## Components Overview

```mermaid
flowchart TD
    A[bazbom CLI]
    B[Build Detection]
    C1[Maven Plugin]
    C2[Gradle Plugin]
    C3[Bazel Aspects]
    D[Graph Normalizer]
    E[Advisory Engine]
    F[Policy Engine]
    G[Exporters]

    A --> B
    B -->|pom.xml| C1
    B -->|build.gradle| C2
    B -->|BUILD/MODULE.bazel| C3
    C1 --> D
    C2 --> D
    C3 --> D
    D --> E
    E --> F
    F --> G
```

**Title:** BazBOM component flow from detection to export

### Rust Workspace

Located in `crates/`:

| Crate | Purpose | Status |
|-------|---------|--------|
| `bazbom` | CLI entry point, command orchestration | Production |
| `bazbom-core` | Shared types, inventory model | Production |
| `bazbom-graph` | Dependency graph normalization, deduplication | Production |
| `bazbom-advisories` | OSV/NVD/GHSA merge, KEV/EPSS enrichment | Production |
| `bazbom-policy` | YAML policy engine, Rego/OPA bridge | Production |
| `bazbom-formats` | SPDX, CycloneDX, SARIF, VEX exporters | Production |
| `bazbom-lsp` | Language Server Protocol for IDE integration | Beta |
| `bazbom-dashboard` | Web UI (D3.js visualizations) | Beta |
| `bazbom-tui` | Terminal UI for dependency exploration | Beta |
| `bazbom-threats` | Supply chain threat detection | Production |
| `bazbom-containers` | OCI image scanning | Beta |
| `bazbom-cache` | Advisory database caching | Production |
| `bazbom-reports` | HTML/PDF report generation | Beta |
| `bazbom-polyglot` | Multi-language ecosystem support (npm, Python, Go, Rust, Ruby, PHP) | Production |

### Build System Integration

**Maven (`plugins/bazbom-maven-plugin/`)**
- Invoked via: `mvn bazbom:graph`
- Emits: Dependency tree JSON with scopes, licenses, PURLs
- Shadow detection: Parse `maven-shade-plugin` config

**Gradle (`plugins/bazbom-gradle-plugin/`)**
- Plugin ID: `io.bazbom.gradle-plugin`
- Emits: Per-configuration dependency graphs
- Shadow detection: Parse Shadow plugin transforms

**Bazel (`tools/supplychain/aspects.bzl`)**
- Aspect: `packages_used`
- Traverses: `java_*`, `kotlin_*`, `scala_*` rules
- Data source: `maven_install.json` via `rules_jvm_external`

### Polyglot Ecosystem Support (NEW in 6.0)

**Location:** `crates/bazbom-polyglot/`

```mermaid
flowchart TD
    A[bazbom scan command] --> B{Detect Ecosystems}

    B -->|JVM| C1[Maven Plugin]
    B -->|JVM| C2[Gradle Plugin]
    B -->|JVM| C3[Bazel Aspects]
    B -->|Polyglot| D[Ecosystem Detection]

    D --> E1[npm Parser]
    D --> E2[Python Parser]
    D --> E3[Go Parser]
    D --> E4[Rust Parser]
    D --> E5[Ruby Parser]
    D --> E6[PHP Parser]

    E1 -->|package.json/lock| F[Package Extraction]
    E2 -->|requirements.txt/poetry| F
    E3 -->|go.mod| F
    E4 -->|Cargo.toml/lock| F
    E5 -->|Gemfile/lock| F
    E6 -->|composer.json/lock| F

    C1 --> G[JVM Graph Normalizer]
    C2 --> G
    C3 --> G

    F --> H[OSV Vulnerability Scanner]
    G --> I[Advisory Engine]

    H --> J[Unified SBOM Generator]
    I --> J

    J --> K[SPDX 2.3 Output]

    style D fill:#e1f5ff
    style F fill:#e1f5ff
    style H fill:#ffe1e1
    style J fill:#e1ffe1
```

**Title:** Unified JVM + Polyglot architecture in BazBOM 6.0

**Ecosystem Detection Strategy:**
1. Recursive directory walk with `walkdir`
2. `filter_entry()` to skip build artifacts (node_modules, target, .git, dist, etc.)
3. Manifest file detection (package.json, Gemfile, go.mod, etc.)
4. Lockfile preference over manifest for exact versions
5. Parallel parsing of detected ecosystems
6. Unified SBOM generation with cross-ecosystem PURLs

**Parser Architecture:**
Each parser implements:
- **Lockfile parsing** (primary): Exact versions with full dependency tree
- **Manifest fallback** (secondary): When lockfile unavailable
- **Version normalization**: Strip operators (^, ~, >=, v prefix, etc.)
- **Namespace extraction**: Registry/organization mapping
- **OSV mapping**: Ecosystem name for vulnerability queries

**Supported File Formats:**
| Ecosystem | Lockfile | Manifest | Parser Lines | Tests |
|-----------|----------|----------|--------------|-------|
| npm | package-lock.json (v6, v7) | package.json | 300 | 3 |
| Python | poetry.lock, Pipfile.lock | requirements.txt, pyproject.toml | 290 | 3 |
| Go | go.sum | go.mod | 282 | 3 |
| Rust | Cargo.lock | Cargo.toml | 240 | 3 |
| Ruby | Gemfile.lock | Gemfile | 290 | 3 |
| PHP | composer.lock | composer.json | 300 | 3 |

See [Polyglot Support Documentation](polyglot/README.md) for detailed guide.

### Reachability Analysis

**Tool:** `bazbom-reachability.jar` (OPAL-based)
**Location:** `tools/reachability/`

```mermaid
sequenceDiagram
    participant CLI as bazbom CLI
    participant JAR as Reachability Analyzer
    participant Cache as Blake3 Cache
    
    CLI->>JAR: Invoke with entrypoints JSON
    JAR->>Cache: Check cached call graphs
    Cache-->>JAR: Cache miss
    JAR->>JAR: ASM bytecode analysis
    JAR->>JAR: Build call graph
    JAR->>Cache: Store fingerprints
    JAR-->>CLI: Return reachable/unreachable tags
```

**Title:** Reachability analysis flow with caching

**Invocation:**
```bash
java -jar bazbom-reachability.jar \
  --entrypoints entrypoints.json \
  --classpath app.jar:lib1.jar:lib2.jar \
  --output call_graph.json
```

**Outputs:**
- Call graph (JSON)
- Reachable CVE tags
- Class-level fingerprints (Blake3)

**Why:** Not reused; built fresh per scan; zero network calls.

## Data Model

### Inventory → SPDX Mapping

| Inventory Field | SPDX Field | Notes |
|----------------|------------|-------|
| `name` | `packages[].name` | Maven: `artifactId` |
| `version` | `packages[].versionInfo` | Exact resolved version |
| `source` | `packages[].downloadLocation` | Maven Central URL or VCS |
| `license` | `packages[].licenseConcluded` | SPDX license ID |
| `hash` | `packages[].checksums[]` | SHA256 from lockfile |
| `purl` | `packages[].externalRefs[]` | `pkg:maven/...` |
| `scope` | `relationships[]` | `RUNTIME_DEPENDENCY_OF` vs `TEST_DEPENDENCY_OF` |

### Advisory Merge Engine

**Sources:** OSV, NVD, GHSA  
**Enrichment:** CISA KEV, EPSS  
**Priority:** P0 (CRITICAL + KEV) → P4 (LOW)

**Gotcha:** Duplicate CVEs across sources are normalized by CVE ID. GHSA-* IDs are preserved separately.

## Decision Records (Mini-ADRs)

### Why Rust?
**Decision:** Rust-first for safety and distribution simplicity.  
**Alternatives:** Keep Python (rejected: deps, CVEs, distribution complexity).  
**Status:** Complete. Zero Python in shipped binary.

### Why SPDX 2.3?
**Decision:** SPDX 2.3 as primary format; CycloneDX 1.5 optional.  
**Reasoning:** Industry standard, tooling maturity, NTIA compliance.  
**Status:** Implemented. Both formats validated in CI.

### Why Bazel Aspects?
**Decision:** Native Bazel aspects vs external scanners.  
**Reasoning:** Hermetic, reproducible, accurate dependency resolution.  
**Status:** Production. Proven on 5000+ target monorepos.

### Why Tool Cache/Checksum?
**Decision:** Blake3 fingerprinting for reachability caching.  
**Reasoning:** Avoid re-analyzing identical JARs across scans.  
**Status:** Implemented. 10x speedup on incremental scans.

## Architecture Links

- Complete architecture: [architecture/architecture.md](architecture/architecture.md)
- Graph analysis: [architecture/graph-analysis.md](architecture/graph-analysis.md)
- ADR index: [ADR/](ADR/)
- Build system details: [BAZEL.md](BAZEL.md)
- CI integration: [CI.md](CI.md)
