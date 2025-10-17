# GitHub Copilot Instructions for BazBOM

## Repository Overview

BazBOM is a **Bazel-native, Java-focused SBOM (Software Bill of Materials) and SCA (Software Composition Analysis)** system that:

- Generates **SPDX 2.3 SBOMs** for every build/deliverable
- Performs comprehensive Software Composition Analysis
- Produces **CI-ready, machine-readable outputs** (JSON + SARIF)
- Integrates with GitHub Code Scanning for security alerts
- Maintains **documentation as a first-class deliverable**

## Core Principles

### 1. Bazel-Native First
- Use **Bazel aspects** for automatic dependency discovery
- Zero manual dependency lists or config files to maintain
- Leverage Bazel's build graph for accurate, complete SBOMs
- All tools run hermetically via Bazel

### 2. Universal Coverage
- Produce an SBOM for **every relevant target**: libraries, binaries, deployables, container images
- Single command (`bazel build //:sbom_all`) generates all SBOMs
- Support incremental analysis for large monorepos

### 3. Minimal Invasiveness
- Assume **little/no package metadata** in existing `BUILD` files
- Solution must **infer** details (versions, licenses, PURLs) without code changes
- Works out-of-box with existing `rules_jvm_external` setups

## Technology Stack

- **Build system:** Bazel 6.0+ (tested up to 7.x)
- **Primary language:** Java/JVM via `rules_java` + `rules_jvm_external`
- **SBOM tooling:** `bazel-contrib/supply-chain` (via `http_archive`, **not BCR**)
- **Dependency source of truth:** `maven_install.json` lockfile
- **Python:** For tooling scripts (write_sbom.py, osv_query.py, sarif_adapter.py, etc.)

## Output Specifications

| Artifact Type | Format | Schema Version | Primary Use |
|--------------|--------|----------------|-------------|
| **SBOM** | SPDX (JSON) | 2.3 | Compliance, attestation, analysis |
| **SBOM** (optional) | CycloneDX (JSON) | 1.5 | Tool compatibility (flag: `--cyclonedx`) |
| **Dependency Graph** | JSON | Custom | Visualization, impact analysis |
| **SCA Findings** | JSON | Custom schema | Machine processing, auditing |
| **SARIF** | SARIF | 2.1.0 | GitHub Code Scanning integration |
| **Provenance** | SLSA Provenance | v1.0 | Build attestation, supply chain verification |
| **VEX** (optional) | CSAF VEX | 2.0 | Vulnerability Exploitability eXchange |
| **License Report** | SPDX License | 2.3 | Legal compliance analysis |

## Repository Structure

```
/
├── WORKSPACE                     # Bazel workspace, fetch bazel-contrib/supply-chain
├── BUILD.bazel                   # Root build file with //:sbom_all target
├── .bazelrc                      # Convenience aliases, remote cache config
├── .bazelversion                 # Pin Bazel version
│
├── tools/supplychain/
│   ├── BUILD.bazel              # Tooling targets (py_binary for scripts)
│   ├── defs.bzl                 # Public macros: sbom_for, sbom_all
│   ├── aspects.bzl              # Aspect implementation for dep traversal
│   ├── write_sbom.py            # Converts dep graph → SPDX 2.3 JSON
│   ├── sarif_adapter.py         # SCA findings → SARIF 2.1.0
│   ├── osv_query.py             # Query OSV for vulnerabilities
│   ├── purl_generator.py        # Maven coords → PURL conversion
│   ├── license_extractor.py     # JAR inspection for license metadata
│   ├── graph_generator.py       # Dependency graph → JSON/GraphML
│   ├── provenance_builder.py    # SLSA provenance generation
│   ├── vex_processor.py         # VEX statement application
│   ├── conflict_detector.py     # Detect version conflicts
│   ├── license_analyzer.py      # License compatibility checks
│   ├── supply_chain_risk.py     # Typosquatting, malware detection
│   ├── metrics_aggregator.py    # Dashboard JSON generation
│   ├── incremental_analyzer.py  # Git diff → affected targets
│   ├── sbom_schemas/            # Schema validation resources
│   ├── validators/              # Schema validation scripts
│   └── tests/                   # Unit tests for Python scripts
│
├── .github/
│   └── workflows/
│       ├── supplychain.yml      # Main CI: SBOM + SCA on every PR/push
│       ├── docs-lint.yml        # Markdown/docs validation
│       └── release.yml          # Release automation
│
├── docs/
│   ├── README.md                # Docs index / navigation
│   ├── SUPPLY_CHAIN.md          # Complete supply chain implementation guide
│   ├── USAGE.md                 # Daily developer commands & workflows
│   ├── ARCHITECTURE.md          # System design, diagrams, data flows
│   ├── VALIDATION.md            # SBOM/SARIF validation procedures
│   ├── TROUBLESHOOTING.md       # Common errors & solutions
│   ├── PERFORMANCE.md           # Optimization guide for large monorepos
│   ├── PROVENANCE.md            # SLSA provenance setup & signing
│   ├── VEX.md                   # VEX statement creation & management
│   ├── GRAPH_ANALYSIS.md        # Dependency graph querying & visualization
│   └── ADR/                     # Architecture Decision Records
│       ├── ADR-0001-fetch-strategy.md
│       ├── ADR-0002-sbom-format.md
│       ├── ADR-0003-aspect-scope.md
│       ├── ADR-0004-sarif-mapping.md
│       ├── ADR-0005-incremental-analysis.md
│       ├── ADR-0006-graph-storage.md
│       └── ADR-0007-slsa-level.md
│
├── examples/
│   ├── minimal_java/            # Smallest working example
│   ├── multi_module/            # Complex monorepo example
│   └── shaded_jar/              # Fat JAR / shaded dependencies
│
└── vex/statements/              # VEX statements for false positive suppression
```

## Documentation Standards (Mandatory)

Documentation quality is a **gate** for merging. Treat docs as code: versioned, reviewed, and validated in CI.

### Documentation Quality Gates
- **Linting:** `markdownlint` (enforced in CI, blocking)
- **Link validation:** All internal/external links must resolve
- **Code samples:** Must be runnable and produce expected output
- **Diagrams:** Keep Mermaid diagrams in sync with implementation

### Documentation Review Checklist
- [ ] Every new feature has corresponding documentation
- [ ] Examples are tested and up-to-date
- [ ] Architecture diagrams reflect current state
- [ ] ADRs document all major decisions
- [ ] Troubleshooting covers actual user issues
- [ ] All commands are copy-pasteable with correct flags

## Key Implementation Details

### Dependency Graph Generation
- Use Bazel aspects to traverse `java_library`, `java_binary`, `jvm_import`, and `maven_install` deps
- Collect: coordinates (group, artifact, version), PURLs, licenses, file SHA256
- Emit stable JSON per target to `bazel-out/.../<target>.deps.json`

### SBOM Generation
- `/tools/supplychain/write_sbom.py` converts `<target>.deps.json` → SPDX 2.3 JSON
- Include: Document, Packages, Files, Relationships (`CONTAINS`, `DEPENDS_ON`)
- Include license expressions (SPDX IDs), provenance (Bazel version, target label, commit)
- Optional: Flag `--cyclonedx` to emit CycloneDX JSON as secondary output

### SCA Integration
- Extract PURLs from SBOM, batch query OSV (or read offline DB)
- Output canonical `sca_findings.json`
- Map findings → SARIF (rules, results, `level`, CWEs)
- Point `artifactLocation` to package or manifest when possible

### Maven/JVM Specifics
- **Source of truth:** `maven_install.json` lockfile
- Parse lockfile for all resolved dependencies with exact versions
- Extract POM metadata for licenses, developers, SCM URLs
- Handle shaded/fat JARs by unpacking and reconstructing dependencies
- Support Kotlin, Scala, and Groovy artifacts

### Performance Optimization for Large Monorepos
- **Incremental analysis:** Only regenerate SBOMs for changed targets
- **Parallelization:** Bazel automatically parallelizes aspect analysis
- **Deduplication:** Store unique dep metadata once, reference by hash
- **Caching:** Full Bazel remote cache support for incremental builds
- **Target filtering:** Exclude test targets from production SBOMs

## Common Commands

### SBOM Generation
```bash
# Generate SBOMs for all targets
bazel build //:sbom_all

# Generate SBOM for single target
bazel build //app:myapp_sbom

# Generate with CycloneDX (in addition to SPDX)
bazel build //:sbom_all --cyclonedx

# Incremental (only changed targets)
bazel build //:sbom_all --config=supplychain-incremental
```

### Vulnerability Scanning
```bash
# Full SCA scan (OSV + NVD + GHSA)
bazel run //:sca_scan

# Scan with custom severity threshold
bazel run //:sca_scan -- --severity-threshold=high

# Offline scan (requires local database)
bazel run //:sca_scan -- --offline-mode --osv-db-path=/opt/osv-db
```

### License Analysis
```bash
# Generate license compliance report
bazel run //:license_report

# Check for license conflicts
bazel run //:license_report -- --check-conflicts

# Flag copyleft licenses
bazel run //:license_report -- --flag-copyleft
```

### Validation
```bash
# Validate all SBOMs against SPDX schema
bazel run //tools/supplychain/validators:validate_sbom -- bazel-bin/**/*.spdx.json

# Validate SARIF output
bazel run //tools/supplychain/validators:validate_sarif -- bazel-bin/sca_findings.sarif
```

## Code Style & Conventions

### Python Scripts
- Use type hints for all function signatures
- Include docstrings for all public functions
- Follow PEP 8 style guide
- Use `argparse` for command-line interfaces
- Handle errors gracefully with meaningful error messages

### Bazel Files
- Use lowercase, snake_case for rule names
- Document macros with usage examples
- Keep BUILD files minimal and readable
- Use aspects for cross-cutting concerns (SBOM, SCA)

### Documentation
- Use Mermaid for diagrams
- Keep examples copy-paste ready
- Include expected outputs for examples
- Update CHANGELOG.md following Keep a Changelog format
- Write ADRs for major architectural decisions

## CI/CD Integration

### Workflow Triggers
- **Every PR:** SBOM generation, SCA scan, SARIF upload
- **Push to main:** Full analysis, artifact upload, metrics aggregation
- **Weekly schedule:** Fresh CVE data updates

### Required Permissions
```yaml
permissions:
  contents: read
  security-events: write
  actions: read
  id-token: write  # For SLSA provenance signing
```

### Artifact Upload
- SBOMs (SPDX + CycloneDX)
- Dependency graphs (JSON + GraphML)
- SCA findings (JSON + SARIF)
- SLSA provenance (signed)
- License reports
- Metrics dashboard

## Guardrails & Best Practices

### Code Changes
- **Hermetic builds:** Only declared repositories, no network access beyond `http_archive`
- **Aspect-first:** Use aspects for dependency discovery, not shell scripts
- **Schema validation:** Validate all outputs (SPDX, SARIF, SLSA) in CI
- **Incremental builds:** Support incremental analysis for large repos

### Security
- Never commit secrets or credentials
- Sign provenance artifacts with Sigstore/cosign
- Validate all external inputs
- Use VEX statements to document false positives
- Enforce policy thresholds (max critical/high vulnerabilities)

### Performance
- Cache JAR metadata by SHA256
- Process JARs in parallel (thread pool)
- Use streaming JSON for large outputs
- Support offline mode for air-gapped environments

## Success Criteria

### Functional Requirements
- `bazel build //:sbom_all` succeeds locally and in CI
- Each target emits valid SPDX 2.3 JSON (schema-validated)
- SCA outputs `sca_findings.json` and `sca_findings.sarif`
- Dependency graph exports as both JSON and GraphML
- SLSA provenance generated for all deployable artifacts

### Performance Requirements
- Small repo (< 50 targets): **< 2 min** end-to-end with remote cache
- Medium repo (50-500 targets): **< 5 min** end-to-end with remote cache
- Large repo (500-5000 targets): **< 15 min** end-to-end with remote cache
- Incremental mode (PRs): **< 5 min** for typical changes

### Documentation Requirements
- All docs present and accurate
- CI enforces docs lint and fails on broken examples
- All code samples are runnable and produce expected output
- Mermaid diagrams render correctly and match implementation

## Working with GitHub Copilot

### Mandatory Requirements for ALL Suggestions

GitHub Copilot must meet these **non-negotiable** standards. Incomplete or low-quality suggestions will be rejected.

#### 1. Context Awareness (REQUIRED)
- **Read before suggesting:** Review ALL related files in `tools/supplychain/`, `docs/`, and test directories
- **Understand dependencies:** Check how the code interacts with Bazel aspects, lockfiles, and external tools
- **Check existing patterns:** Match the repository's existing code style, structure, and conventions
- **Review recent changes:** Look at git history to understand recent design decisions
- **NO assumptions:** If you don't have context, ask clarifying questions instead of guessing

#### 2. Error Handling (MANDATORY)
Every code suggestion MUST include:
- **Explicit error handling** for ALL failure modes (file I/O, network, parsing, validation)
- **Meaningful error messages** with actionable guidance (not "Error occurred")
- **Exit codes** that distinguish error types (0=success, 1=user error, 2=system error)
- **Validation** of ALL inputs (file paths, JSON structure, schema compliance)
- **Graceful degradation** where possible (e.g., offline mode fallback)
- **Error context:** Include file paths, line numbers, and relevant data in error messages

**Example of UNACCEPTABLE error handling:**
```python
def parse_json(file_path):
    with open(file_path) as f:
        return json.load(f)  # ❌ No error handling
```

**Example of REQUIRED error handling:**
```python
def parse_json(file_path: str) -> dict:
    """Parse JSON file with comprehensive error handling."""
    if not os.path.exists(file_path):
        raise FileNotFoundError(f"JSON file not found: {file_path}")

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON in {file_path} at line {e.lineno}: {e.msg}")
    except PermissionError:
        raise PermissionError(f"Cannot read {file_path}: permission denied")
    except Exception as e:
        raise RuntimeError(f"Failed to parse {file_path}: {str(e)}")

    return data
```

#### 3. Testing (NON-NEGOTIABLE)
ALL code changes require:
- **Unit tests** in `tools/supplychain/tests/` covering:
  - Happy path (expected inputs, expected outputs)
  - Edge cases (empty inputs, maximum size inputs, special characters)
  - Error conditions (invalid inputs, missing files, malformed data)
  - Boundary conditions (single item, thousands of items)
- **Integration tests** for multi-component features
- **Performance tests** for operations on large datasets (1000+ items)
- **Test data fixtures** in `tools/supplychain/tests/fixtures/`
- **Minimum 80% code coverage** for new code
- **All tests must pass** before suggesting the code

**Example test structure:**
```python
class TestPURLGenerator(unittest.TestCase):
    def test_maven_coordinates_to_purl_happy_path(self):
        """Test standard Maven coordinates."""
        purl = maven_to_purl("com.google.guava", "guava", "31.1-jre")
        self.assertEqual(purl, "pkg:maven/com.google.guava/guava@31.1-jre")

    def test_maven_coordinates_invalid_group_id(self):
        """Test rejection of invalid group ID."""
        with self.assertRaises(ValueError) as ctx:
            maven_to_purl("", "guava", "31.1-jre")
        self.assertIn("group_id", str(ctx.exception))

    def test_maven_coordinates_special_characters(self):
        """Test handling of special characters in coordinates."""
        # Test with dots, hyphens, underscores
        purl = maven_to_purl("org.springframework.boot", "spring-boot_2.13", "3.0.0-RC1")
        self.assertTrue(purl.startswith("pkg:maven/"))
```

#### 4. Documentation (MANDATORY FOR ALL CHANGES)
- **Update ALL affected docs** in `docs/` - no exceptions
- **NEVER create new documentation files** - update existing docs in `docs/` instead
- **ALL documentation MUST live in `docs/` directory** - no scattered markdown files
- **Update existing docs, don't proliferate:** Add sections to existing files rather than creating new ones
- **Allowed new docs:** Only ADRs in `docs/ADR/` following the ADR-NNNN-title.md naming convention
- **Code comments:** Explain WHY, not WHAT (code explains what)
- **Docstrings:** Required for all public functions, classes, and modules
- **Examples:** Provide copy-paste ready examples with expected outputs
- **Architecture diagrams:** Update Mermaid diagrams if data flow changes
- **Troubleshooting:** Add common errors to `docs/TROUBLESHOOTING.md` (don't create separate troubleshooting docs)
- **CHANGELOG:** Update with user-facing changes

**Documentation location rules:**
- ✅ Update `docs/USAGE.md` with new commands
- ✅ Update `docs/ARCHITECTURE.md` with design changes
- ✅ Add to `docs/TROUBLESHOOTING.md` for error solutions
- ✅ Create `docs/ADR/ADR-NNNN-topic.md` for architectural decisions
- ❌ DON'T create `NEW_FEATURE.md` in repo root
- ❌ DON'T create scattered docs outside `docs/`
- ❌ DON'T create a new doc for every task or feature

**Unacceptable documentation:**
```python
def process(data):
    # Process the data
    result = transform(data)  # ❌ Useless comment
    return result
```

**Required documentation:**
```python
def process_sbom_dependencies(sbom_data: dict) -> list[Dependency]:
    """Extract and normalize dependency information from SPDX SBOM.

    Handles SPDX 2.3 format with Relationships section. Resolves
    transitive dependencies via DEPENDS_ON relationships and deduplicates
    by PURL.

    Args:
        sbom_data: Parsed SPDX 2.3 JSON document with packages and relationships

    Returns:
        List of Dependency objects with normalized PURLs, licenses, and hashes

    Raises:
        ValueError: If SBOM is missing required fields (packages, relationships)
        SchemaError: If SBOM doesn't conform to SPDX 2.3 schema

    Example:
        >>> with open('app.spdx.json') as f:
        ...     sbom = json.load(f)
        >>> deps = process_sbom_dependencies(sbom)
        >>> len(deps)
        42
        >>> deps[0].purl
        'pkg:maven/com.google.guava/guava@31.1-jre'
    """
    if not sbom_data.get('packages'):
        raise ValueError("SBOM missing required 'packages' field")
    # ... implementation with clear logic
```

#### 5. Schema Validation (ALWAYS REQUIRED)
- **Validate ALL inputs** against schemas (SPDX 2.3, SARIF 2.1.0, SLSA v1.0)
- **Validate ALL outputs** before writing to disk
- **Use official schemas:** Load from `tools/supplychain/sbom_schemas/`
- **Fail fast:** Reject invalid data immediately with clear error messages
- **Include validation in tests:** Test both valid and invalid schema cases

#### 6. Performance & Scale (DESIGN REQUIREMENT)
Design ALL code to handle:
- **5000+ Bazel targets** in a single workspace
- **2000+ unique dependencies** with complex transitive graphs
- **100MB+ JSON files** (large SBOMs, dependency graphs)
- **Parallel processing:** Use thread pools or multiprocessing where appropriate
- **Memory efficiency:** Stream large files, don't load entire datasets into memory
- **Incremental processing:** Support partial updates, don't regenerate everything
- **Caching:** Cache expensive operations (JAR metadata extraction, HTTP requests)

**Performance requirements:**
- File I/O: Stream files > 10MB
- HTTP requests: Batch requests, max 100 items per batch
- JSON parsing: Use `ijson` for files > 50MB
- Deduplication: Use hash-based lookups, not linear scans

#### 7. Security Requirements (CRITICAL)
- **Input validation:** Sanitize ALL user inputs and external data
- **Path traversal prevention:** Validate all file paths, reject `..` and absolute paths outside workspace
- **Secrets detection:** Never log or print sensitive data (tokens, keys, credentials)
- **Dependency pinning:** All external tools must have SHA256 hashes
- **SLSA compliance:** Follow SLSA Level 3 requirements for provenance
- **VEX statements:** Document and track all false positives
- **Fail secure:** Default to denying access, not granting it

#### 8. Code Quality Standards (ENFORCED)
- **Type hints:** Required for ALL function signatures (Python)
- **Null safety:** Handle None/null cases explicitly
- **Immutability:** Prefer immutable data structures
- **Single responsibility:** One function = one purpose
- **DRY principle:** No copy-paste code, extract shared logic
- **Naming:** Use descriptive names, no abbreviations (except well-known: SBOM, PURL, SARIF)
- **Line length:** Max 100 characters
- **Complexity:** Max cyclomatic complexity of 10 per function

#### 9. Edge Cases & Boundary Conditions (MUST HANDLE)
Code must handle:
- **Empty inputs:** Empty files, empty arrays, empty strings
- **Maximum inputs:** Thousands of items, huge files
- **Malformed data:** Invalid JSON, missing required fields, wrong types
- **Unicode:** Non-ASCII characters, emojis, RTL text
- **Special characters:** Quotes, backslashes, newlines in strings
- **Concurrency:** Race conditions, file locking
- **Network failures:** Timeouts, connection errors, partial responses
- **Disk space:** Handle out-of-disk gracefully
- **Permissions:** Handle read-only filesystems

#### 10. Bazel Integration (STRICT)
- **Hermetic builds:** No network access except declared `http_archive`
- **Deterministic outputs:** Same inputs = same outputs (byte-for-byte)
- **Stable JSON:** Sort keys, use consistent formatting
- **Aspect best practices:** Don't traverse test dependencies unless explicitly enabled
- **Output paths:** Write to `bazel-bin/`, never to source tree
- **Action caching:** Ensure actions are cacheable (no timestamps in outputs)
- **Remote cache friendly:** Support Bazel remote execution and caching

### Quality Checklist (EVERY SUGGESTION)

Before suggesting ANY code, verify:

- [ ] I have read all related files in the repository
- [ ] Error handling covers all failure modes
- [ ] Tests cover happy path, edge cases, and error conditions
- [ ] Documentation is updated (code comments, docstrings, markdown docs)
- [ ] Schema validation is implemented for inputs and outputs
- [ ] Code handles large-scale inputs (1000+ items)
- [ ] Security best practices are followed
- [ ] Type hints are present and correct
- [ ] Code follows repository style and conventions
- [ ] Examples are runnable and produce expected output
- [ ] Performance implications are considered
- [ ] Integration with Bazel is hermetic and cacheable

### What NOT to Suggest

- **Half-implemented features:** Don't suggest partial solutions
- **TODO comments:** Complete the implementation or don't suggest it
- **Placeholder error messages:** Every error must be specific and actionable
- **Untested code:** All code must have corresponding tests
- **Breaking changes without migration:** Provide backward compatibility or migration guide
- **Magic numbers:** Use named constants
- **Silent failures:** All errors must be reported
- **Copy-paste code:** Extract shared logic into functions
- **Undocumented public APIs:** All public functions need docstrings
- **New documentation files:** Don't create new docs - update existing ones in `docs/`

## Common Anti-Patterns (DO NOT DO THESE)

### ❌ Anti-Pattern 1: Vague Error Messages
```python
# BAD
raise Exception("Error")
raise ValueError("Invalid input")
raise RuntimeError("Something went wrong")

# GOOD
raise FileNotFoundError(f"SBOM file not found at {sbom_path}. Expected SPDX 2.3 JSON file.")
raise ValueError(f"Invalid PURL format: '{purl}'. Expected 'pkg:maven/group/artifact@version'")
raise SchemaValidationError(f"SBOM failed validation at field 'packages[3].licenseConcluded': {details}")
```

### ❌ Anti-Pattern 2: Missing Input Validation
```python
# BAD
def process_deps(deps):
    for dep in deps:
        print(dep['purl'])  # Assumes 'purl' key exists

# GOOD
def process_deps(deps: list[dict]) -> None:
    if not isinstance(deps, list):
        raise TypeError(f"Expected list of dependencies, got {type(deps)}")

    for i, dep in enumerate(deps):
        if not isinstance(dep, dict):
            raise TypeError(f"Dependency {i} is not a dict: {type(dep)}")
        if 'purl' not in dep:
            raise KeyError(f"Dependency {i} missing required 'purl' field")

        purl = dep['purl']
        if not purl.startswith('pkg:'):
            raise ValueError(f"Invalid PURL format at index {i}: '{purl}'")

        print(purl)
```

### ❌ Anti-Pattern 3: No Tests
```python
# BAD: Suggesting code without tests

# GOOD: Every code suggestion includes corresponding tests
def test_process_deps_empty_list(self):
    """Test that empty dependency list is handled."""
    process_deps([])  # Should not raise

def test_process_deps_missing_purl_key(self):
    """Test error handling for missing PURL."""
    with self.assertRaises(KeyError) as ctx:
        process_deps([{'name': 'foo'}])
    self.assertIn('purl', str(ctx.exception))
```

### ❌ Anti-Pattern 4: Ignoring Scale
```python
# BAD: Loads entire 100MB SBOM into memory
with open('sbom.json') as f:
    sbom = json.load(f)
    for package in sbom['packages']:  # Could be 10,000+ packages
        process(package)

# GOOD: Streams large files
import ijson

with open('sbom.json', 'rb') as f:
    packages = ijson.items(f, 'packages.item')
    for package in packages:
        process(package)  # Processes one at a time
```

### ❌ Anti-Pattern 5: Creating Scattered Documentation
```python
# BAD: Creating new docs everywhere
# - Creates: NEW_FEATURE.md in root
# - Creates: tools/supplychain/TOOL_GUIDE.md
# - Creates: HOW_TO_USE_VEX.md in root

# GOOD: Update existing structured docs
# - Updates: docs/USAGE.md (add commands section)
# - Updates: docs/VEX.md (add usage examples)
# - Updates: docs/ARCHITECTURE.md (add design notes)
# - Creates: docs/ADR/ADR-0008-new-decision.md (only for architectural decisions)
```

### ❌ Anti-Pattern 6: No Error Context
```python
# BAD
try:
    result = parse_sbom(file_path)
except Exception:
    print("Failed to parse SBOM")
    sys.exit(1)

# GOOD
try:
    result = parse_sbom(file_path)
except json.JSONDecodeError as e:
    print(f"ERROR: Invalid JSON in {file_path}")
    print(f"  Line {e.lineno}, column {e.colno}: {e.msg}")
    print(f"  Ensure file is valid SPDX 2.3 JSON format")
    sys.exit(2)
except SchemaValidationError as e:
    print(f"ERROR: SBOM schema validation failed for {file_path}")
    print(f"  {e.details}")
    print(f"  See docs/VALIDATION.md for schema requirements")
    sys.exit(2)
except FileNotFoundError:
    print(f"ERROR: SBOM file not found: {file_path}")
    print(f"  Run 'bazel build //:sbom_all' to generate SBOMs")
    sys.exit(1)
```

### ❌ Anti-Pattern 7: Untested Edge Cases
```python
# BAD: Only tests happy path
def test_maven_to_purl(self):
    purl = maven_to_purl("com.google.guava", "guava", "31.1-jre")
    self.assertEqual(purl, "pkg:maven/com.google.guava/guava@31.1-jre")

# GOOD: Tests edge cases and error conditions
def test_maven_to_purl_happy_path(self):
    purl = maven_to_purl("com.google.guava", "guava", "31.1-jre")
    self.assertEqual(purl, "pkg:maven/com.google.guava/guava@31.1-jre")

def test_maven_to_purl_empty_group_id(self):
    with self.assertRaises(ValueError):
        maven_to_purl("", "guava", "31.1-jre")

def test_maven_to_purl_empty_artifact_id(self):
    with self.assertRaises(ValueError):
        maven_to_purl("com.google.guava", "", "31.1-jre")

def test_maven_to_purl_empty_version(self):
    with self.assertRaises(ValueError):
        maven_to_purl("com.google.guava", "guava", "")

def test_maven_to_purl_special_characters(self):
    purl = maven_to_purl("org.spring-framework.boot", "app_name", "1.0.0-SNAPSHOT")
    self.assertTrue(purl.startswith("pkg:maven/"))

def test_maven_to_purl_unicode_characters(self):
    # Ensure Unicode is properly encoded
    purl = maven_to_purl("com.example", "café", "1.0.0")
    self.assertIn("caf", purl)  # Should be URL-encoded
```

## Copilot Performance Expectations

GitHub Copilot suggestions will be evaluated on:

1. **Correctness:** Code must work as intended with NO bugs
2. **Completeness:** Includes error handling, tests, and documentation
3. **Quality:** Follows all coding standards and best practices
4. **Security:** No vulnerabilities or insecure patterns
5. **Performance:** Handles scale requirements (5000+ targets, 2000+ deps)
6. **Maintainability:** Clear, well-documented, follows existing patterns

**Grading criteria:**
- ✅ **Excellent:** Meets all requirements, comprehensive tests, clear docs
- ⚠️ **Acceptable:** Meets most requirements, minor gaps in tests or docs
- ❌ **Unacceptable:** Missing error handling, no tests, vague errors, or scattered docs

Anything below "Acceptable" will be **rejected immediately**.

## Quick Reference

### Key Files to Modify
- **WORKSPACE:** For external dependencies
- **tools/supplychain/aspects.bzl:** For dependency traversal logic
- **tools/supplychain/write_sbom.py:** For SBOM generation logic
- **tools/supplychain/osv_query.py:** For vulnerability scanning
- **tools/supplychain/sarif_adapter.py:** For SARIF output generation
- **.github/workflows/supplychain.yml:** For CI/CD pipeline

### Key Documentation Files
- **README.md:** High-level overview and quickstart
- **docs/SUPPLY_CHAIN.md:** Detailed implementation guide
- **docs/ARCHITECTURE.md:** System design and data flows
- **docs/USAGE.md:** Command reference and examples
- **docs/TROUBLESHOOTING.md:** Common issues and solutions

### Testing Changes
```bash
# Build all SBOMs
bazel build //:sbom_all

# Run unit tests
bazel test //tools/supplychain/tests/...

# Validate SBOM schema
bazel run //tools/supplychain/validators:validate_sbom -- bazel-bin/**/*.spdx.json

# Run full SCA scan
bazel run //:sca_scan

# Lint documentation
npm run lint:docs  # or equivalent markdownlint command
```

## Non-Goals

- No ad-hoc scanners outside Bazel unless executed **via Bazel** for hermeticity
- No manual dependency lists maintained outside lockfiles
- No BCR (Bazel Central Registry) usage - use `http_archive` for `bazel-contrib/supply-chain`

## Additional Resources

- **SPDX Specification:** https://spdx.github.io/spdx-spec/
- **SARIF Specification:** https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
- **SLSA Provenance:** https://slsa.dev/provenance/
- **OSV Database:** https://osv.dev/
- **Package URL (PURL):** https://github.com/package-url/purl-spec
- **Bazel Aspects:** https://bazel.build/extending/aspects
