# Phase 9: Container & JVM Ecosystem Expansion

**Status:** Planned
**Priority:** 🟡 P1 - Strategic Depth
**Timeline:** Months 6-9 (12 weeks)
**Team Size:** 2-3 developers
**Dependencies:** Phase 0-3 complete, Phase 4 recommended (IDE integration reusable)

---

## Executive Summary

**Goal:** Deepen JVM ecosystem coverage with container support and additional build systems.

**Current Scope:** BazBOM supports Maven, Gradle, Bazel for Java/Kotlin/Scala. This phase expands to:
1. **Container Support** - Detect and scan JVM artifacts in Docker/OCI images
2. **Additional JVM Build Systems** - Ant, Buildr support
3. **Enhanced JVM Language Support** - Groovy, Clojure improvements
4. **Kotlin Multiplatform** - JVM targets in multiplatform projects
5. **Additional Scala Tooling** - sbt (Scala Build Tool)

> **⚠️ SCOPE CLARIFICATION:** BazBOM is **JVM-ONLY**. Multi-language support (Node.js, Python, Go, Rust) is **OUT OF SCOPE**. This ensures world-class depth for JVM ecosystems rather than shallow breadth across many languages.

**Success Metrics:**
- ✅ Comprehensive JVM build system coverage (Ant, Maven, Gradle, Bazel, Buildr)
- ✅ All JVM languages supported (Java, Kotlin, Scala, Groovy, Clojure)
- ✅ Container scanning detects JVM artifacts with 99%+ accuracy
- ✅ Support largest JVM monorepos (5000+ targets)

**Strategic Rationale:** Become the definitive tool for **all** JVM projects, regardless of build system or language variant.

---

## Architecture: JVM Build System Integration

### Modular Build System Support

**Problem:** Each JVM build system has unique dependency resolution

**Solution:** Modular parsers for each JVM build system

**Structure:**
```
crates/
├── bazbom/                   # Core CLI
├── bazbom-core/              # Shared utilities
├── bazbom-formats/           # SBOM/SARIF outputs
├── bazbom-advisories/        # Advisory engine
├── bazbom-build-systems/     # Build system integrations
│   ├── mod.rs               # BuildSystemDetector trait
│   ├── ant/                 # Ant (build.xml) support
│   ├── maven/               # Maven (pom.xml) existing
│   ├── gradle/              # Gradle (build.gradle) existing
│   ├── bazel/               # Bazel (BUILD.bazel) existing
│   └── buildr/              # Buildr (buildfile, Rakefile)
├── bazbom-containers/        # Container scanning (JVM artifacts)
└── bazbom-languages/         # JVM language enhancements
    ├── groovy/              # Groovy-specific features
    └── clojure/             # Clojure-specific features
```

### BuildSystemDetector Trait

**Interface:**
```rust
// crates/bazbom-build-systems/src/lib.rs
pub trait BuildSystemDetector: Send + Sync {
    /// Name of the build system (e.g., "ant", "maven", "gradle")
    fn name(&self) -> &str;

    /// Detect if project uses this build system
    fn detect(&self, project_root: &Path) -> Result<bool>;

    /// Extract dependency graph from build files
    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph>;

    /// Generate SBOM for this build system
    fn generate_sbom(&self, graph: &DependencyGraph) -> Result<Sbom>;

    /// Scan for vulnerabilities (uses shared advisory engine)
    fn scan_vulnerabilities(&self, graph: &DependencyGraph) -> Result<Vec<Vulnerability>>;

    /// Reachability analysis (JVM-specific via ASM)
    fn analyze_reachability(&self, project_root: &Path) -> Result<Option<ReachabilityResult>> {
        // Default: Use bazbom-reachability.jar for all JVM projects
        self.run_jvm_reachability(project_root)
    }
}
```

**Build System Registry:**
```rust
// crates/bazbom-build-systems/src/registry.rs
pub struct BuildSystemRegistry {
    detectors: Vec<Box<dyn BuildSystemDetector>>,
}

impl BuildSystemRegistry {
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(AntDetector::new()),
                Box::new(MavenDetector::new()),
                Box::new(GradleDetector::new()),
                Box::new(BazelDetector::new()),
                Box::new(BuildrDetector::new()),
            ],
        }
    }

    pub fn detect_all(&self, project_root: &Path) -> Result<Vec<&dyn BuildSystemDetector>> {
        self.detectors
            .iter()
            .filter(|detector| detector.detect(project_root).unwrap_or(false))
            .map(|boxed| boxed.as_ref())
            .collect()
    }
}
```

---

## 9.1 Ant Build System Support

### Overview

**Goal:** Add support for Apache Ant, one of the original Java build tools.

**Target Users:** Legacy Java projects, enterprise systems with Ant builds

### Detection

**Files:** `build.xml` (required), `ivy.xml` (optional for dependency management)

### Implementation Strategy

**Phase 1:** Ant with Ivy dependency management
- Parse `ivy.xml` for dependencies
- Resolve via Ivy's dependency resolution
- Generate dependency graph

**Phase 2:** Ant without Ivy
- Parse `build.xml` for `<path>` elements
- Detect manual JAR dependencies in `lib/` directories
- Limited transitive dependency support

### Dependency Extraction

```rust
// crates/bazbom-build-systems/src/ant/mod.rs
pub struct AntDetector;

impl BuildSystemDetector for AntDetector {
    fn name(&self) -> &str { "ant" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("build.xml").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Check for Ivy first (preferred)
        if project_root.join("ivy.xml").exists() {
            self.extract_ivy_dependencies(project_root)
        } else {
            // Fallback: scan build.xml and lib/ directory
            self.extract_manual_dependencies(project_root)
        }
    }

    fn extract_ivy_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Parse ivy.xml using XML parser
        // Resolve dependencies via Ivy resolver
        // Build dependency graph
        todo!("Implement Ivy XML parsing")
    }

    fn extract_manual_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Scan lib/ directory for JARs
        // Extract metadata from JAR manifests
        // Limited graph (direct dependencies only)
        todo!("Implement manual JAR scanning")
    }
}
```

### PURL Format

**Maven coordinates:** `pkg:maven/org.apache.commons/commons-lang3@3.12.0`

**Ivy format:** Same as Maven (Ivy uses Maven repos)

### Challenges

- **No standard lock file:** Ant doesn't have equivalent of `pom.xml.lock`
- **Manual dependency management:** Many Ant projects use checked-in JARs
- **Limited metadata:** JARs may lack proper manifests

### Advisory Sources

- Reuse existing Maven advisory database (same artifacts)
- OSV.dev (supports Maven coordinates)

---

## 9.2 Buildr Build System Support

### Overview

**Goal:** Support Buildr, a Ruby-based JVM build tool

**Target Users:** JRuby projects, legacy systems

### Detection

**Files:** `buildfile`, `Rakefile` (with Buildr DSL)

### Implementation Strategy

Buildr uses Ruby DSL but targets JVM:
- Parse buildfile for Maven-style dependencies
- Resolve via Maven repositories
- Support Buildr's artifact() syntax

### Dependency Extraction

```rust
// crates/bazbom-build-systems/src/buildr/mod.rs
pub struct BuildrDetector;

impl BuildSystemDetector for BuildrDetector {
    fn name(&self) -> &str { "buildr" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        let buildfile = project_root.join("buildfile");
        if buildfile.exists() {
            // Check if it contains Buildr syntax
            let content = fs::read_to_string(&buildfile)?;
            Ok(content.contains("Buildr::") || content.contains("artifact("))
        } else {
            Ok(false)
        }
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Parse buildfile for artifact() declarations
        // Format: artifact('group:name:jar:version')
        // Resolve via Maven Central
        todo!("Implement Buildr DSL parsing")
    }
}
```

### PURL Format

**Maven coordinates:** `pkg:maven/group/artifact@version`

### Challenges

- **Ruby DSL parsing:** Need Ruby parser or regex-based extraction
- **Dynamic dependencies:** Buildr allows programmatic dependency definition
- **Legacy tool:** Limited modern usage

---

## 9.3 Groovy Language Support

### Overview

**Goal:** Enhanced support for Groovy-based JVM projects

**Current State:** Basic support via Maven/Gradle

**Enhancements:**
- Grape (Groovy dependency manager) support
- @Grab annotation parsing for inline dependencies
- GroovyDoc analysis

### Detection

**Files:** `go.mod`, `go.sum`

### Dependency Extraction

**go.mod Parsing:**
```rust
// crates/bazbom-ecosystems/src/go/mod.rs
pub struct GoPlugin;

impl EcosystemPlugin for GoPlugin {
    fn name(&self) -> &str { "go" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("go.mod").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        let go_mod = fs::read_to_string(project_root.join("go.mod"))?;
        let mut graph = DependencyGraph::new();

        let mut in_require_block = false;

        for line in go_mod.lines() {
            let line = line.trim();

            if line == "require (" {
                in_require_block = true;
                continue;
            }

            if line == ")" {
                in_require_block = false;
                continue;
            }

            if in_require_block || line.starts_with("require ") {
                // Parse: require github.com/gin-gonic/gin v1.9.0
                let parts: Vec<&str> = line.split_whitespace().collect();

                if parts.len() >= 2 {
                    let name = parts[0].trim_start_matches("require ");
                    let version = parts[1].trim_start_matches('v');

                    graph.add_component(Component {
                        name: name.to_string(),
                        version: version.to_string(),
                        purl: format!("pkg:golang/{}@{}", name, version),
                        ecosystem: Ecosystem::Go,
                    });
                }
            }
        }

        Ok(graph)
    }
}
```

### PURL Format

**Go:** `pkg:golang/github.com/gin-gonic/gin@v1.9.0`

### Advisory Sources

- **Go Vulnerability Database** (https://vuln.go.dev/)
- **OSV.dev** (supports Go)
- **GitHub Advisory Database** (many Go packages)

---

## 9.4 Rust/Cargo Support

### Detection

**Files:** `Cargo.toml`, `Cargo.lock`

### Dependency Extraction

**Cargo.lock Parsing:**
```rust
// crates/bazbom-ecosystems/src/rust_ecosystem/mod.rs
use cargo_lock::Lockfile;

pub struct RustPlugin;

impl EcosystemPlugin for RustPlugin {
    fn name(&self) -> &str { "rust" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("Cargo.toml").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        let lockfile = Lockfile::load(project_root.join("Cargo.lock"))?;
        let mut graph = DependencyGraph::new();

        for package in &lockfile.packages {
            graph.add_component(Component {
                name: package.name.to_string(),
                version: package.version.to_string(),
                purl: format!("pkg:cargo/{}@{}", package.name, package.version),
                ecosystem: Ecosystem::Rust,
            });
        }

        Ok(graph)
    }
}
```

**Dependency:** `cargo-lock = "9.0"` (official Rust lockfile parser)

### PURL Format

**Cargo:** `pkg:cargo/serde@1.0.188`

### Advisory Sources

- **RustSec Advisory Database** (https://rustsec.org/)
- **OSV.dev** (supports Rust)
- **GitHub Advisory Database**

---

## 9.5 Enhanced Container Support

### Current State

**Phase 0-3:** Basic Syft fallback for container SBOMs

**Gap:** No reachability analysis for container dependencies

### Target State

**Features:**
- Multi-layer SBOM (OS packages + application dependencies)
- Dockerfile analysis (base image vulnerabilities)
- Runtime SBOM (what's actually loaded)

### Implementation

**Dockerfile Parsing:**
```rust
// crates/bazbom-ecosystems/src/container/mod.rs
pub struct ContainerPlugin;

impl ContainerPlugin {
    pub fn analyze_dockerfile(&self, dockerfile_path: &Path) -> Result<ContainerAnalysis> {
        let content = fs::read_to_string(dockerfile_path)?;
        let mut analysis = ContainerAnalysis::default();

        for line in content.lines() {
            let line = line.trim();

            // Extract base image
            if line.starts_with("FROM ") {
                let base_image = line.strip_prefix("FROM ").unwrap().split_whitespace().next().unwrap();
                analysis.base_image = Some(base_image.to_string());
            }

            // Detect package installations
            if line.contains("apt-get install") || line.contains("yum install") || line.contains("apk add") {
                analysis.os_package_installs.push(line.to_string());
            }

            // Detect vulnerabilities in RUN commands
            if line.contains("curl") && !line.contains("https://") {
                analysis.warnings.push(Warning {
                    type_: "INSECURE_DOWNLOAD",
                    message: "Downloading over HTTP instead of HTTPS".to_string(),
                    line: line.to_string(),
                });
            }
        }

        Ok(analysis)
    }

    pub fn scan_container_image(&self, image_name: &str) -> Result<Sbom> {
        // Use Syft for comprehensive container scanning
        let output = Command::new("syft")
            .args(&[image_name, "-o", "spdx-json"])
            .output()?;

        let sbom: Sbom = serde_json::from_slice(&output.stdout)?;
        Ok(sbom)
    }
}
```

**Integration:**
```bash
# Scan Dockerfile
bazbom scan --dockerfile Dockerfile

# Scan container image
bazbom scan --container myapp:latest

# Combined: source code + container
bazbom scan . --with-container
```

---

## 9.6 Polyglot Project Support

### Use Case

**Example:** Monorepo with:
- Java backend (`pom.xml`)
- Node.js frontend (`package.json`)
- Python ML service (`requirements.txt`)

### Implementation

**Multi-Ecosystem Scan:**
```rust
// crates/bazbom/src/scan_orchestrator.rs
impl ScanOrchestrator {
    pub fn scan_polyglot(&self, project_root: &Path) -> Result<PolyglotScanResult> {
        let registry = EcosystemRegistry::new();
        let detected = registry.detect_ecosystems(project_root)?;

        println!("Detected ecosystems:");
        for ecosystem in &detected {
            println!("  - {}", ecosystem.name());
        }

        let mut results = Vec::new();

        for ecosystem in detected {
            println!("\nScanning {} dependencies...", ecosystem.name());

            let graph = ecosystem.extract_dependencies(project_root)?;
            let sbom = ecosystem.generate_sbom(&graph)?;
            let vulns = ecosystem.scan_vulnerabilities(&graph)?;

            results.push(EcosystemResult {
                ecosystem: ecosystem.name().to_string(),
                sbom,
                vulnerabilities: vulns,
            });
        }

        Ok(PolyglotScanResult { results })
    }
}
```

**Merged Output:**
```json
{
  "ecosystems": [
    {
      "name": "jvm",
      "dependencies": 247,
      "vulnerabilities": 3
    },
    {
      "name": "node",
      "dependencies": 512,
      "vulnerabilities": 12
    },
    {
      "name": "python",
      "dependencies": 68,
      "vulnerabilities": 1
    }
  ],
  "total_vulnerabilities": 16,
  "critical": 2,
  "high": 7,
  "medium": 5,
  "low": 2
}
```

---

## Success Criteria

### Phase 9 Completion Checklist

- [ ] Node.js/npm support with package-lock.json parsing
- [ ] Python/pip support with poetry.lock and requirements.txt
- [ ] Go modules support with go.mod/go.sum parsing
- [ ] Rust/Cargo support with Cargo.lock parsing
- [ ] Enhanced container support with Dockerfile analysis
- [ ] Polyglot project detection and scanning
- [ ] Advisory engine works for all ecosystems (OSV integration)
- [ ] SBOM generation for each ecosystem (SPDX + CycloneDX)
- [ ] Universal CLI: `bazbom scan` works for any language
- [ ] Documentation for each ecosystem

### Competitive Benchmark

**After Phase 9:**

| Feature | Checkmarx | Snyk | BazBOM |
|---------|-----------|------|--------|
| **Java/JVM** | ✅ | ✅ | ✅ |
| **Node.js** | ✅ | ✅ | ✅ |
| **Python** | ✅ | ✅ | ✅ |
| **Go** | ✅ | ✅ | ✅ |
| **Rust** | ✅ | ✅ | ✅ |
| **Containers** | ✅ | ✅ | ✅ |
| **Polyglot Projects** | ✅ | ✅ | ✅ |
| **Total Languages** | 75+ | 10+ | 6 |
| **Cost** | $200+/dev/year | $99+/dev/year | **FREE** |

**Gap:** Still fewer languages than Checkmarx, but covers 90% of use cases.

---

## Resource Requirements

**Team:** 2-3 developers for 12 weeks
- 1x Node.js/Python specialist (6 weeks)
- 1x Go/Rust specialist (6 weeks)
- 1x Container specialist (part-time, 4 weeks)

**Skills:** Multi-language expertise, lockfile parsing, advisory integration

**Budget:** $48K-72K (contractors)

---

**Last Updated:** 2025-10-30
**Next:** Phase 10 (AI Intelligence)
