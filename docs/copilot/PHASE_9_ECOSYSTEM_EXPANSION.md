# Phase 9: Container & JVM Ecosystem Expansion

**Status:** Planned
**Priority:**  P1 - Strategic Depth
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

> ** SCOPE CLARIFICATION:** BazBOM is **JVM-ONLY**. Multi-language support (Node.js, Python, Go, Rust) is **OUT OF SCOPE**. This ensures world-class depth for JVM ecosystems rather than shallow breadth across many languages.

**Success Metrics:**
-  Comprehensive JVM build system coverage (Ant, Maven, Gradle, Bazel, Buildr)
-  All JVM languages supported (Java, Kotlin, Scala, Groovy, Clojure)
-  Container scanning detects JVM artifacts with 99%+ accuracy
-  Support largest JVM monorepos (5000+ targets)

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

**Files:** `.groovy` files with @Grab annotations, `grape.xml`

### Implementation

```rust
// crates/bazbom-languages/src/groovy/mod.rs
pub struct GroovyAnalyzer;

impl GroovyAnalyzer {
    pub fn extract_grab_dependencies(&self, project_root: &Path) -> Result<Vec<Dependency>> {
        // Scan .groovy files for @Grab annotations
        // Format: @Grab(group='org.springframework', module='spring-core', version='5.3.0')
        // Convert to Maven coordinates
        
        let groovy_files = find_groovy_files(project_root)?;
        let mut dependencies = Vec::new();
        
        for file in groovy_files {
            let content = fs::read_to_string(&file)?;
            // Parse @Grab annotations using regex or parser
            dependencies.extend(parse_grab_annotations(&content)?);
        }
        
        Ok(dependencies)
    }
}
```

### Features

- Parse @Grab, @GrabResolver, @GrabExclude annotations
- Support Grape dependency management
- Groovy script dependency detection
- Integration with Maven Central

### PURL Format

**Maven coordinates:** `pkg:maven/org.springframework/spring-core@5.3.0`

### Advisory Sources

- Reuse existing Maven advisory database
- OSV.dev (supports Maven coordinates)

---

## 9.4 sbt (Scala Build Tool) Support

### Overview

**Goal:** Add support for sbt, the standard Scala build tool

**Target Users:** Scala projects using sbt (most common Scala build tool)

### Detection

**Files:** `build.sbt`, `project/build.properties`

### Implementation

```rust
// crates/bazbom-build-systems/src/sbt/mod.rs
pub struct SbtDetector;

impl BuildSystemDetector for SbtDetector {
    fn name(&self) -> &str { "sbt" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("build.sbt").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Parse build.sbt for library dependencies
        // Format: libraryDependencies += "org.typelevel" %% "cats-core" % "2.9.0"
        // Run sbt dependencyTree and parse output
        todo!("Implement sbt dependency extraction")
    }
}
```

### Features

- Parse build.sbt for dependencies
- Support %% (Scala version suffix) resolution
- Extract from sbt dependencyTree output
- Multi-project sbt builds

### PURL Format

**Maven coordinates with Scala version:** `pkg:maven/org.typelevel/cats-core_2.13@2.9.0`

### Advisory Sources

- Maven advisory database (sbt uses Maven repos)
- OSV.dev

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

## 9.6 Multi-Module JVM Project Support

### Use Case

**Example:** JVM monorepo with:
- Java backend (`pom.xml` or `build.gradle`)
- Kotlin services (`build.gradle.kts`)
- Scala data processing (`build.sbt`)

### Implementation

**Multi-Build-System Scan:**
```rust
// crates/bazbom/src/scan_orchestrator.rs
impl ScanOrchestrator {
    pub fn scan_multi_module(&self, project_root: &Path) -> Result<MultiModuleScanResult> {
        let registry = BuildSystemRegistry::new();
        let detected = registry.detect_all(project_root)?;

        println!("Detected JVM build systems:");
        for build_system in &detected {
            println!("  - {}", build_system.name());
        }

        let mut results = Vec::new();

        for build_system in detected {
            println!("\nScanning {} dependencies...", build_system.name());

            let graph = build_system.extract_dependencies(project_root)?;
            let sbom = build_system.generate_sbom(&graph)?;
            let vulns = build_system.scan_vulnerabilities(&graph)?;

            results.push(ModuleResult {
                build_system: build_system.name().to_string(),
                sbom,
                vulnerabilities: vulns,
            });
        }

        Ok(MultiModuleScanResult { results })
    }
}
```

**Merged Output:**
```json
{
  "build_systems": [
    {
      "name": "maven",
      "modules": ["backend-api", "common-lib"],
      "dependencies": 247,
      "vulnerabilities": 3
    },
    {
      "name": "gradle",
      "modules": ["kotlin-service"],
      "dependencies": 189,
      "vulnerabilities": 2
    },
    {
      "name": "bazel",
      "modules": ["//java/processing/..."],
      "dependencies": 156,
      "vulnerabilities": 1
    }
  ],
  "total_dependencies": 592,
  "total_vulnerabilities": 6,
  "critical": 1,
  "high": 2,
  "medium": 2,
  "low": 1
}
```

---

## Success Criteria

### Phase 9 Completion Checklist

- [ ] Ant build system support with Ivy dependency management
- [ ] Buildr build system support with Maven coordinate resolution
- [ ] Groovy language enhancements (@Grab annotation parsing)
- [ ] Clojure language enhancements (tools.deps, deps.edn parsing)
- [ ] sbt (Scala Build Tool) support
- [ ] Kotlin Multiplatform support (JVM targets)
- [ ] Enhanced container support for JVM artifacts (JAR/WAR detection in images)
- [ ] Multi-module JVM project detection and scanning
- [ ] Advisory engine works for all JVM artifacts (Maven Central, OSV integration)
- [ ] SBOM generation for all JVM build systems (SPDX + CycloneDX)
- [ ] Universal CLI: `bazbom scan` works for all JVM build systems
- [ ] Documentation for each build system and language variant

### Competitive Benchmark

**After Phase 9:**

| Feature | Checkmarx | Snyk | BazBOM |
|---------|-----------|------|--------|
| **Maven** |  |  |  |
| **Gradle** |  |  |  |
| **Bazel (JVM)** |  |  |  |
| **Ant + Ivy** |  |  |  |
| **Buildr** |  |  |  |
| **sbt (Scala)** |  |  |  |
| **Groovy (@Grab)** |  |  |  |
| **Clojure (tools.deps)** |  |  |  |
| **Containers (JVM artifacts)** |  |  |  |
| **JVM Language Coverage** | Partial | Good | **Complete** |
| **JVM Build Systems** | 3-4 | 3-4 | **8+** |
| **Cost** | $200+/dev/year | $99+/dev/year | **FREE** |

**Advantage:** Best-in-class JVM coverage. Depth over breadth strategy.

---

## Resource Requirements

**Team:** 2-3 developers for 12 weeks
- 1x JVM build systems specialist (Ant, Buildr, sbt) (6 weeks)
- 1x JVM languages specialist (Groovy, Clojure, Kotlin Multiplatform) (6 weeks)
- 1x Container specialist (JVM artifact detection) (part-time, 4 weeks)

**Skills:** JVM ecosystem expertise, build tool internals, XML/EDN/TOML parsing, Maven repository integration

**Budget:** $48K-72K (contractors)

---

**Last Updated:** 2025-10-30
**Next:** Phase 10 (AI Intelligence)
