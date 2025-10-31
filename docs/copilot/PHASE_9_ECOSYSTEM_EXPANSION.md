# Phase 9: Ecosystem Expansion

**Status:** Planned
**Priority:** 🟡 P1 - Strategic Growth
**Timeline:** Months 6-9 (12 weeks)
**Team Size:** 2-3 developers (1 per ecosystem)
**Dependencies:** Phase 0-3 complete, Phase 4 recommended (IDE integration reusable)

---

## Executive Summary

**Goal:** Expand beyond JVM-only to become a multi-language SCA tool.

**Current Limitation:** BazBOM only supports Java/Kotlin/Scala. Competitors support 10-75+ languages.

**Target Ecosystems (Priority Order):**
1. **Node.js/npm** - Largest package ecosystem (2M+ packages)
2. **Python/pip** - Data science, ML, DevOps tooling
3. **Go modules** - Cloud-native, Kubernetes ecosystem
4. **Rust/Cargo** - Growing systems programming adoption
5. **Containers** - Enhanced support beyond current Syft fallback

**Success Metrics:**
- ✅ Support top 5 ecosystems with same features as JVM
- ✅ Universal `bazbom scan` command works for any language
- ✅ Polyglot projects (e.g., Java backend + Node.js frontend) supported
- ✅ 50K+ weekly scans across all ecosystems

**Strategic Rationale:** Expand market size from $900M (JVM only) to $2.25B (all languages).

---

## Architecture: Plugin System

### Why Plugins?

**Problem:** Hardcoding support for each language makes CLI bloated

**Solution:** Plugin architecture with language-specific modules

**Structure:**
```
crates/
├── bazbom/                   # Core CLI
├── bazbom-core/              # Shared utilities
├── bazbom-formats/           # SBOM/SARIF (language-agnostic)
├── bazbom-advisories/        # Advisory engine (reusable)
├── bazbom-ecosystems/        # NEW: Plugin framework
│   ├── mod.rs               # EcosystemPlugin trait
│   ├── jvm/                 # Java/Kotlin/Scala (existing)
│   ├── node/                # Node.js/npm
│   ├── python/              # Python/pip
│   ├── go/                  # Go modules
│   └── rust/                # Rust/Cargo
```

### EcosystemPlugin Trait

**Interface:**
```rust
// crates/bazbom-ecosystems/src/lib.rs
pub trait EcosystemPlugin: Send + Sync {
    /// Name of the ecosystem (e.g., "node", "python")
    fn name(&self) -> &str;

    /// Detect if project uses this ecosystem
    fn detect(&self, project_root: &Path) -> Result<bool>;

    /// Extract dependency graph from lockfiles/manifests
    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph>;

    /// Generate SBOM for this ecosystem
    fn generate_sbom(&self, graph: &DependencyGraph) -> Result<Sbom>;

    /// Scan for vulnerabilities (uses shared advisory engine)
    fn scan_vulnerabilities(&self, graph: &DependencyGraph) -> Result<Vec<Vulnerability>>;

    /// Optional: Reachability analysis (if supported)
    fn analyze_reachability(&self, project_root: &Path) -> Result<Option<ReachabilityResult>> {
        Ok(None)  // Default: not supported
    }
}
```

**Plugin Registry:**
```rust
// crates/bazbom-ecosystems/src/registry.rs
pub struct EcosystemRegistry {
    plugins: Vec<Box<dyn EcosystemPlugin>>,
}

impl EcosystemRegistry {
    pub fn new() -> Self {
        Self {
            plugins: vec![
                Box::new(JvmPlugin::new()),
                Box::new(NodePlugin::new()),
                Box::new(PythonPlugin::new()),
                Box::new(GoPlugin::new()),
                Box::new(RustPlugin::new()),
            ],
        }
    }

    pub fn detect_ecosystems(&self, project_root: &Path) -> Result<Vec<&dyn EcosystemPlugin>> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.detect(project_root).unwrap_or(false))
            .map(|boxed| boxed.as_ref())
            .collect()
    }
}
```

---

## 9.1 Node.js/npm Support

### Detection

**Files:** `package.json`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`

### Dependency Extraction

**Lockfile Parsing:**
```rust
// crates/bazbom-ecosystems/src/node/mod.rs
use serde_json::Value;

pub struct NodePlugin;

impl EcosystemPlugin for NodePlugin {
    fn name(&self) -> &str { "node" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("package.json").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        let package_lock = project_root.join("package-lock.json");

        if package_lock.exists() {
            self.parse_package_lock(&package_lock)
        } else {
            // Fallback: parse package.json (no version lock)
            self.parse_package_json(&project_root.join("package.json"))
        }
    }

    fn parse_package_lock(&self, path: &Path) -> Result<DependencyGraph> {
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        let mut graph = DependencyGraph::new();

        // package-lock.json v2/v3 format
        if let Some(packages) = json["packages"].as_object() {
            for (name, pkg) in packages {
                if name.is_empty() {
                    continue;  // Skip root
                }

                let name = name.trim_start_matches("node_modules/");
                let version = pkg["version"].as_str().unwrap_or("unknown");

                graph.add_component(Component {
                    name: name.to_string(),
                    version: version.to_string(),
                    purl: format!("pkg:npm/{}@{}", name, version),
                    ecosystem: Ecosystem::Node,
                });
            }
        }

        Ok(graph)
    }
}
```

### PURL Format

**npm:** `pkg:npm/express@4.18.2`
**Scoped:** `pkg:npm/%40babel/core@7.22.0` (URL-encoded `@`)

### Advisory Sources

- **npm Advisory Database** (https://github.com/npm/security-advisories)
- **OSV.dev** (supports npm)
- **Snyk Vulnerability Database** (public API)

### Reachability Analysis

**Challenge:** JavaScript is dynamically typed, no bytecode

**Approach:** Static analysis with tree-sitter (parse JS/TS AST)

**Deferred:** Phase 10 (lower priority than JVM reachability)

---

## 9.2 Python/pip Support

### Detection

**Files:** `requirements.txt`, `Pipfile`, `Pipfile.lock`, `poetry.lock`, `pyproject.toml`

### Dependency Extraction

**Lockfile Parsing:**
```rust
// crates/bazbom-ecosystems/src/python/mod.rs
pub struct PythonPlugin;

impl EcosystemPlugin for PythonPlugin {
    fn name(&self) -> &str { "python" }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("requirements.txt").exists() ||
           project_root.join("Pipfile").exists() ||
           project_root.join("pyproject.toml").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Priority order: poetry.lock > Pipfile.lock > requirements.txt
        if project_root.join("poetry.lock").exists() {
            self.parse_poetry_lock(&project_root.join("poetry.lock"))
        } else if project_root.join("Pipfile.lock").exists() {
            self.parse_pipfile_lock(&project_root.join("Pipfile.lock"))
        } else {
            self.parse_requirements_txt(&project_root.join("requirements.txt"))
        }
    }

    fn parse_requirements_txt(&self, path: &Path) -> Result<DependencyGraph> {
        let content = fs::read_to_string(path)?;
        let mut graph = DependencyGraph::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse package==version format
            if let Some((name, version)) = line.split_once("==") {
                graph.add_component(Component {
                    name: name.to_string(),
                    version: version.to_string(),
                    purl: format!("pkg:pypi/{}@{}", name, version),
                    ecosystem: Ecosystem::Python,
                });
            }
        }

        Ok(graph)
    }
}
```

### PURL Format

**PyPI:** `pkg:pypi/requests@2.28.1`
**Namespace:** `pkg:pypi/azure/storage@12.0.0`

### Advisory Sources

- **PyPI Advisory Database** (https://github.com/pypa/advisory-database)
- **OSV.dev** (supports Python)
- **Safety DB** (requires API key for full access)

---

## 9.3 Go Modules Support

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
