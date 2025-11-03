# BazBOM Current Architecture

**Last Updated:** 2025-11-03
**Status:** Transition phase - Python and Rust coexist

---

## Overview

BazBOM is in an active transition from a Python-based architecture to a Rust-first implementation. Both systems currently coexist, with the Rust CLI providing the primary user interface while delegating certain functionality to Python backends or build system plugins.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Interface                           │
├─────────────────────────────────────────────────────────────────┤
│  Rust CLI (bazbom)                                              │
│  - Command parsing & validation        ✅ Production            │
│  - Build system detection              ✅ Production            │
│  - Orchestration & workflow            ✅ Production            │
│  - Output formatting                   ✅ Production            │
└────────────────┬────────────────────────────────────────────────┘
                 │
    ┌────────────┴────────────┬──────────────────┬────────────────┐
    │                         │                   │                │
    ▼                         ▼                   ▼                ▼
┌─────────┐          ┌──────────────┐    ┌──────────────┐  ┌──────────┐
│ Rust    │          │ Build System │    │ Python       │  │ IDE      │
│ Services│          │ Plugins      │    │ Backend      │  │ Plugins  │
└─────────┘          └──────────────┘    └──────────────┘  └──────────┘
```

## Component Details

### 1. Rust CLI Layer (Primary Interface)

**Location:** `crates/bazbom/`

**Responsibilities:**
- Parse command-line arguments (Clap framework)
- Detect build system (Maven, Gradle, Bazel)
- Orchestrate scanning workflows
- Manage advisory database
- Enforce policies
- Generate output files

**Status:** ✅ Fully functional

**What It Does Well:**
```bash
bazbom scan .                    # Detects build system
bazbom db sync                   # Syncs advisory databases
bazbom policy init --template    # Initializes policies
bazbom policy check              # Validates compliance
bazbom install-hooks             # Installs git hooks
```

**What It Delegates:**
- Full dependency extraction → Build plugins or Python
- Deep SBOM enrichment → Python backend
- Bazel aspects → Python implementation

### 2. Rust Service Modules

#### 2.1 Advisory Service ✅ Production Ready

**Location:** `crates/bazbom-advisories/`

**Capabilities:**
- Downloads from OSV, NVD, GHSA, KEV, EPSS
- Caches locally in `.bazbom/cache/advisories/`
- Merges multiple sources intelligently
- Canonicalizes severity levels
- Enriches with KEV flags and EPSS scores

**Data Flow:**
```
User: bazbom db sync
         ↓
Advisory Service downloads from:
  - OSV (Open Source Vulnerabilities)
  - NVD (National Vulnerability Database)
  - GHSA (GitHub Security Advisories)
  - KEV (CISA Known Exploited Vulnerabilities)
  - EPSS (Exploit Prediction Scoring)
         ↓
Cache stored in .bazbom/cache/advisories/
         ↓
Used by scan command for vulnerability matching
```

#### 2.2 Policy Engine ✅ Production Ready

**Location:** `crates/bazbom-policy/`

**Capabilities:**
- YAML policy parsing and validation
- 5 enterprise templates (PCI-DSS, HIPAA, FedRAMP, SOC 2, Corporate)
- Multi-level inheritance (org → team → project)
- Severity thresholds
- KEV gating
- EPSS filtering
- License allowlist/denylist

**Tests:** 42 passing unit tests

#### 2.3 SBOM Formats ✅ Schema Complete

**Location:** `crates/bazbom-formats/`

**Capabilities:**
- SPDX 2.3 data structures
- CycloneDX 1.5 data structures
- SARIF 2.1.0 data structures
- JSON serialization/deserialization

**Status:** Formats implemented, content requires build plugins

#### 2.4 LSP Server ✅ Builds Successfully

**Location:** `crates/bazbom-lsp/`

**Capabilities:**
- Language Server Protocol implementation
- File watching (pom.xml, build.gradle, BUILD.bazel)
- Diagnostic publishing
- Code actions for quick fixes
- Async scanning

**Status:** Binary builds and starts, needs real-world testing

### 3. Build System Plugins

#### 3.1 Maven Plugin ⚠️ Exists, Not Auto-Integrated

**Location:** `plugins/bazbom-maven-plugin/`

**Language:** Java

**Capabilities:**
- Full dependency tree extraction
- Scope tracking (compile, runtime, test, provided)
- Effective POM analysis
- BOM imports
- Conflict resolution tracking
- Shading/relocation mapping
- PURLs, licenses, hashes

**Usage:**
```xml
<plugin>
    <groupId>io.bazbom</groupId>
    <artifactId>bazbom-maven-plugin</artifactId>
    <version>1.0.0</version>
    <executions>
        <execution>
            <goals>
                <goal>graph</goal>
            </goals>
        </execution>
    </executions>
</plugin>
```

**Output:** `target/bazbom-graph.json`

**Integration Status:** Manual - user must add to pom.xml and run

#### 3.2 Gradle Plugin ⚠️ Exists, Not Auto-Integrated

**Location:** `plugins/bazbom-gradle-plugin/`

**Language:** Kotlin

**Capabilities:**
- Similar to Maven plugin
- Gradle-specific dependency resolution
- Shadow plugin support
- Multi-configuration graphs

**Integration Status:** Manual - user must add to build.gradle

### 4. Python Backend (Being Ported)

**Location:** `tools/supplychain/`

**Components:**
- `dependency_scanner.py` - RipGrep-based fast discovery
- `graph_generator.py` - Dependency graph construction
- `osv_query.py` - OSV API integration
- `ghsa_enrichment.py` - GHSA integration
- `provenance_builder.py` - SLSA provenance
- `sbom_signing.py` - Sigstore integration
- `scan_container.py` - Container SBOM generation
- 100+ other Python files

**Status:** ⚠️ Mature and functional, being gradually ported to Rust

**When Used:**
- Bazel projects (aspects and dependency extraction)
- Full SBOM generation without plugins
- Container scanning
- SLSA provenance generation

### 5. IDE Integration

#### 5.1 VS Code Extension ⚠️ Scaffolded

**Location:** `crates/bazbom-vscode-extension/`

**Status:**
- ✅ TypeScript code compiles
- ✅ LSP client configured
- ✅ Commands defined
- ❌ Not tested with actual VS Code
- ❌ Not published to marketplace

#### 5.2 IntelliJ Plugin ⚠️ Scaffolded

**Location:** `crates/bazbom-intellij-plugin/`

**Status:**
- ✅ Kotlin code complete
- ✅ Gradle builds successfully
- ✅ Features implemented (annotators, quick fixes, tool window)
- ❌ Not tested with actual IntelliJ
- ❌ Not published to JetBrains Marketplace

## Data Flow: Full SBOM Generation

### Option 1: Maven Project (Using Plugin)

```
User: bazbom scan /path/to/maven/project
         ↓
1. Rust CLI detects pom.xml
         ↓
2. Rust CLI generates stub SBOM
         ↓
3. User must separately run: mvn bazbom:graph
         ↓
4. Maven plugin extracts full dependency tree
         ↓
5. Output: target/bazbom-graph.json
         ↓
6. User re-runs: bazbom scan . (reads graph.json)
         ↓
7. Full SBOM with all dependencies generated
```

### Option 2: Bazel Project (Using Python)

```
User: bazbom scan /path/to/bazel/project
         ↓
1. Rust CLI detects MODULE.bazel or WORKSPACE
         ↓
2. Rust CLI generates stub SBOM
         ↓
3. Python tools invoked (tools/supplychain/)
         ↓
4. Bazel aspects extract dependencies
         ↓
5. Python generates full SBOM
         ↓
6. Output written to specified directory
```

### Option 3: Direct Python Invocation

```
User: python tools/supplychain/run_scan.py
         ↓
1. Python detects build system
         ↓
2. Python extracts dependencies
         ↓
3. Python queries vulnerabilities
         ↓
4. Python generates full SBOM + SARIF
         ↓
5. Complete workflow without Rust CLI
```

## Porting Progress

### Completed (Rust Implementation)

1. ✅ CLI framework and command parsing
2. ✅ Build system detection
3. ✅ Advisory database sync and caching
4. ✅ Policy engine with templates
5. ✅ Pre-commit hooks
6. ✅ SBOM format structures
7. ✅ SARIF format structures
8. ✅ Remediation logic (needs testing)
9. ✅ LSP server foundation

### In Progress (Partial Rust Implementation)

1. ⚠️ SBOM generation (formats done, content extraction needs work)
2. ⚠️ Dependency graph (structures exist, population incomplete)
3. ⚠️ Shading detection (code exists, not tested)
4. ⚠️ Reachability analysis (unclear status)

### Still Python (To Be Ported)

1. 🔄 Full dependency extraction without plugins
2. 🔄 Bazel aspects
3. 🔄 Container scanning
4. 🔄 SLSA provenance generation
5. 🔄 Advanced graph analysis
6. 🔄 Deep enrichment workflows

## Configuration

### Rust CLI Configuration

**Location:** `bazbom.yml` or `.bazbom.yml`

**Format:** YAML

**Example:**
```yaml
policy:
  severity_threshold: HIGH
  kev_gate: true
  epss_threshold: 0.5

scan:
  reachability: true
  fast_mode: false
  
output:
  formats: [spdx, cyclonedx, sarif]
  directory: ./output
```

### Environment Variables

```bash
BAZBOM_POLICY_FILE=custom-policy.yml
BAZBOM_CACHE_DIR=~/.bazbom/cache
BAZBOM_OUTPUT_DIR=./sbom-output
```

## Testing Strategy

### Rust Tests

**Location:** `crates/*/tests/` and inline

**Coverage:** 90%+ (target)

**Test Count:** 74+ unit tests

**Run:** `cargo test --all`

### Python Tests

**Location:** `tools/supplychain/test_*.py`

**Coverage:** Varies by module

**Run:** `pytest`

### Integration Tests

**Status:** Partial - needs expansion

## Performance Characteristics

### Rust CLI

- **Startup:** <100ms
- **Build system detection:** <10ms
- **Advisory sync:** 2-5 minutes (one-time)
- **Policy check:** <1 second
- **Stub SBOM generation:** <1 second

### With Plugins

- **Maven plugin:** Depends on project size (seconds to minutes)
- **Gradle plugin:** Similar to Maven
- **Full scan:** Seconds to minutes depending on dependency count

## Security Considerations

### Rust CLI

- ✅ Memory-safe (no unsafe blocks except in dependencies)
- ✅ No network access for scanning (offline-first)
- ✅ Advisory database cached locally
- ✅ Explicit sync command (`bazbom db sync`)

### Build Plugins

- ⚠️ Run within build system (Maven/Gradle)
- ⚠️ Access to full project context
- ✅ No network access during scan
- ✅ Output to local filesystem only

### Python Backend

- ⚠️ Python runtime required
- ⚠️ Multiple dependencies
- ✅ No telemetry
- ✅ Offline mode supported

## Migration Strategy

### For End Users

**Phase 1 (Current):**
- Use Rust CLI for commands and orchestration
- Use build plugins for full SBOM generation
- Python backend available as fallback

**Phase 2 (In Progress):**
- More features ported to Rust
- Rust CLI can do more without plugins
- Python becomes optional for most workflows

**Phase 3 (Future):**
- Rust CLI fully self-contained
- Build plugins still available for deep integration
- Python completely optional

### For Contributors

**Priority 1:** Port critical path features to Rust
**Priority 2:** Maintain Python backend for complex features
**Priority 3:** Gradually deprecate Python as Rust reaches parity

See [docs/copilot/EPICS_PORTING.md](copilot/EPICS_PORTING.md) for detailed porting plan.

## Troubleshooting Common Issues

### "SBOM is empty or has no dependencies"

**Cause:** Rust CLI generated stub SBOM without plugin data

**Solution:**
- For Maven: Add and run `bazbom-maven-plugin`
- For Gradle: Add and run `bazbom-gradle-plugin`
- For Bazel: Ensure Python tools are accessible

### "Advisory cache not found"

**Cause:** Advisory database not synced

**Solution:** Run `bazbom db sync` before first scan

### "Command not found: bazbom"

**Cause:** Rust CLI not in PATH

**Solution:**
- Install via Homebrew: `brew install bazbom`
- Or add to PATH: `export PATH="$PATH:/path/to/target/debug"`

### "Python module not found"

**Cause:** Python dependencies not installed

**Solution:** `pip install -r requirements.txt` (if using Python features)

## References

- [Implementation Status](copilot/IMPLEMENTATION_STATUS.md) - Detailed capability audit
- [Porting Plan](copilot/EPICS_PORTING.md) - Python to Rust migration
- [Phase 4 Progress](copilot/PHASE_4_PROGRESS.md) - IDE integration status
- [Migration Guide](MIGRATION_GUIDE.md) - User migration guide

---

**Document Version:** 1.0
**Next Update:** After major architectural changes
