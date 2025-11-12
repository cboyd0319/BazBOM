# BazBOM 6.2.0 Roadmap: Polyglot Upgrade Intelligence

**Release Target:** December 2025 (2 weeks sprint)
**Mission:** Bring Upgrade Intelligence and Interactive Fixing to all polyglot ecosystems

**Part of the Full Polyglot Parity Initiative:**
- **v6.2.0** - Upgrade Intelligence + Interactive Fixing ← YOU ARE HERE
- **v6.3.0** - JavaScript/TypeScript Reachability Analysis
- **v6.4.0** - Python + Go Reachability Analysis
- **v6.5.0** - Rust + Ruby + PHP Reachability + Complete Parity

---

## 🎯 The Gap

Currently, BazBOM has two tiers of support:

### Tier 1: JVM (World-Class)
- ✅ SBOM generation
- ✅ Vulnerability scanning
- ✅ **Upgrade Intelligence** - recursive transitive analysis with breaking changes
- ✅ **Interactive Fixing** - `bazbom fix` with guided remediation
- ✅ Reachability analysis
- ✅ Shading detection

### Tier 2: Polyglot (Functional but Basic)
- ✅ SBOM generation
- ✅ Vulnerability scanning (OSV API)
- ❌ **Upgrade Intelligence** - NOT IMPLEMENTED
- ❌ **Interactive Fixing** - NOT IMPLEMENTED
- ❌ Reachability analysis - NOT PLANNED (language-specific, complex)

**The Problem:** The two killer features that make BazBOM different (Upgrade Intelligence and Interactive Fixing) only work for Maven. This creates a jarring experience for polyglot monorepos.

---

## 🚀 Version 6.2.0 Goals

**Make Upgrade Intelligence and Interactive Fixing work for ALL supported ecosystems.**

### Primary Deliverables

1. **Upgrade Intelligence for Polyglot**
   - npm packages: `bazbom fix express --explain`
   - PyPI packages: `bazbom fix django --explain`
   - Go modules: `bazbom fix github.com/gin-gonic/gin --explain`
   - Rust crates: `bazbom fix serde --explain`
   - Ruby gems: `bazbom fix rails --explain`
   - PHP packages: `bazbom fix symfony/symfony --explain`

2. **Interactive Fix for Polyglot**
   - Detect ecosystem from package identifier
   - Apply fixes to appropriate manifest files
   - Test automation for npm (package.json), PyPI (requirements.txt), etc.

3. **Automated Dependency Updates**
   - Update package.json (npm)
   - Update requirements.txt, pyproject.toml (Python)
   - Update go.mod (Go)
   - Update Cargo.toml (Rust)
   - Update Gemfile (Ruby)
   - Update composer.json (PHP)

---

## 📋 Technical Implementation Plan

### Phase 1: Upgrade Intelligence Extension (Week 1)

#### Task 1.1: Extend deps.dev Integration
**File:** `crates/bazbom-depsdev/src/client.rs`

Currently, the deps.dev client only queries Maven. Extend it to support all ecosystems:

```rust
pub async fn get_dependencies(
    &self,
    system: System, // Already supports Npm, PyPI, Go, Cargo, RubyGems
    package: &str,
    version: &str,
) -> Result<DependencyGraph>
```

**Implementation:**
- ✅ Already supports multiple systems in the enum
- ✅ API calls work for all ecosystems
- ✅ No code changes needed - just use it!

#### Task 1.2: Make UpgradeAnalyzer Multi-Ecosystem
**File:** `crates/bazbom-upgrade-analyzer/src/analyzer.rs`

Current problem (line 51):
```rust
// HARDCODED TO MAVEN!
let direct_analysis = self
    .analyze_single_package(System::Maven, package, from_version, to_version)
    .await?;
```

**Solution:** Add ecosystem detection

```rust
pub async fn analyze_upgrade(
    &mut self,
    package: &str,
    from_version: &str,
    to_version: &str,
) -> Result<UpgradeAnalysis> {
    // Detect ecosystem from package format
    let system = detect_ecosystem_from_package(package);

    let direct_analysis = self
        .analyze_single_package(system, package, from_version, to_version)
        .await?;
    // ... rest of the code stays the same
}
```

**Ecosystem Detection Logic:**
```rust
fn detect_ecosystem_from_package(package: &str) -> System {
    if package.contains(':') {
        // Maven GAV format: org.springframework:spring-core
        System::Maven
    } else if package.starts_with('@') || package.contains('/') && !package.starts_with("github.com") {
        // npm scoped: @types/node or express
        System::Npm
    } else if package.starts_with("github.com/") || package.contains("golang.org/") {
        // Go module path
        System::Go
    } else if package.contains('-') && package.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric()) {
        // PyPI: django, scikit-learn
        System::PyPI
    } else {
        // Rust crates: serde, tokio
        System::Cargo
    }
}
```

**Time Estimate:** 4 hours

#### Task 1.3: GitHub Release Notes Parser (Multi-Ecosystem)
**File:** `crates/bazbom-upgrade-analyzer/src/github.rs`

Good news: This already works generically! It:
- Fetches GitHub releases
- Parses markdown for breaking changes
- Works for any repo regardless of language

**Time Estimate:** 0 hours (already done!)

---

### Phase 2: Interactive Fix Extension (Week 1-2)

#### Task 2.1: Extend Fix Command to Detect Ecosystem
**File:** `crates/bazbom/src/commands/fix.rs`

Add ecosystem detection to the fix command:

```rust
pub async fn handle_fix(
    package: Option<String>,
    // ... other params
) -> Result<()> {
    if explain {
        if let Some(pkg) = package {
            // Detect ecosystem and pass to upgrade intelligence
            let system = detect_ecosystem_from_package(&pkg);
            return upgrade_intelligence::explain_upgrade(&pkg, system).await;
        }
    }
    // ...
}
```

**Time Estimate:** 2 hours

#### Task 2.2: Dependency File Updaters
**New Files:**
- `crates/bazbom/src/remediation/updaters/npm.rs`
- `crates/bazbom/src/remediation/updaters/python.rs`
- `crates/bazbom/src/remediation/updaters/go.rs`
- `crates/bazbom/src/remediation/updaters/rust.rs`
- `crates/bazbom/src/remediation/updaters/ruby.rs`
- `crates/bazbom/src/remediation/updaters/php.rs`

Each updater implements:
```rust
pub trait DependencyUpdater {
    /// Update dependency version in manifest file
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()>;

    /// Run package manager install command
    fn install(&self, project_root: &Path) -> Result<()>;

    /// Get lockfile path
    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf>;
}
```

**npm Implementation Example:**
```rust
pub struct NpmUpdater;

impl DependencyUpdater for NpmUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut pkg_json: serde_json::Value = serde_json::from_str(&content)?;

        // Update dependencies
        if let Some(deps) = pkg_json["dependencies"].as_object_mut() {
            if deps.contains_key(package) {
                deps[package] = json!(new_version);
            }
        }

        // Update devDependencies
        if let Some(dev_deps) = pkg_json["devDependencies"].as_object_mut() {
            if dev_deps.contains_key(package) {
                dev_deps[package] = json!(new_version);
            }
        }

        fs::write(file_path, serde_json::to_string_pretty(&pkg_json)?)?;
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        std::process::Command::new("npm")
            .arg("install")
            .current_dir(project_root)
            .status()?;
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        Some(project_root.join("package-lock.json"))
    }
}
```

**Time Estimate:** 12 hours (2 hours per ecosystem)

#### Task 2.3: Test Runners for Each Ecosystem
**File:** `crates/bazbom/src/test_runner.rs`

Add test runners for each package manager:

```rust
pub fn run_tests_for_ecosystem(system: System, project_root: &Path) -> Result<TestResult> {
    match system {
        System::Maven => run_maven_tests(project_root),
        System::Npm => run_npm_tests(project_root),
        System::PyPI => run_python_tests(project_root),
        System::Go => run_go_tests(project_root),
        System::Cargo => run_rust_tests(project_root),
        System::RubyGems => run_ruby_tests(project_root),
    }
}

fn run_npm_tests(root: &Path) -> Result<TestResult> {
    let output = Command::new("npm")
        .arg("test")
        .current_dir(root)
        .output()?;

    Ok(TestResult {
        passed: output.status.success(),
        output: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}
```

**Time Estimate:** 6 hours

---

### Phase 3: Integration & Polish (Week 2)

#### Task 3.1: Update Interactive Fix UI
**File:** `crates/bazbom/src/interactive_fix.rs`

Extend the TUI to show ecosystem-specific information:
- Package manager being used
- Manifest file being updated
- Test command being run

**Time Estimate:** 4 hours

#### Task 3.2: End-to-End Testing
Create test projects for each ecosystem and verify:
```bash
# npm
cd examples/npm-app
bazbom fix express --explain
bazbom fix express --interactive

# Python
cd examples/python-app
bazbom fix django --explain
bazbom fix django --interactive

# Go
cd examples/go-app
bazbom fix github.com/gin-gonic/gin --explain
bazbom fix github.com/gin-gonic/gin --interactive

# Rust
cd examples/rust-app
bazbom fix serde --explain
bazbom fix serde --interactive
```

**Time Estimate:** 6 hours

#### Task 3.3: Documentation Updates
- Update README.md to show polyglot upgrade intelligence examples
- Add docs/polyglot/upgrade-intelligence.md
- Update docs/polyglot/README.md

**Time Estimate:** 2 hours

---

## 📊 Success Criteria

### Technical Requirements
- ✅ `bazbom fix <package> --explain` works for all 6 ecosystems
- ✅ `bazbom fix --interactive` handles npm, PyPI, Go, Rust packages
- ✅ Automated version updates work for all manifest formats
- ✅ Test runners execute for all package managers
- ✅ Breaking change detection from GitHub releases
- ✅ 100+ new unit tests covering polyglot fix functionality

### User Experience Requirements
- ✅ Same beautiful UX for polyglot as Maven
- ✅ Error messages are clear and actionable
- ✅ Auto-detection of ecosystem "just works"
- ✅ Documentation has examples for all 6 ecosystems

---

## 📅 Development Timeline

### Week 1 (Dec 2-6)
- **Day 1-2:** Upgrade Intelligence extension
  - Ecosystem detection logic
  - Update analyzer to use detected system
  - Test with npm, PyPI, Go, Rust

- **Day 3-5:** Dependency updaters
  - Implement npm updater
  - Implement Python updater
  - Implement Go updater

### Week 2 (Dec 9-13)
- **Day 1-2:** More dependency updaters
  - Implement Rust updater
  - Implement Ruby updater (optional)
  - Implement PHP updater (optional)

- **Day 3:** Test runners
  - Test execution for all ecosystems
  - Rollback on failure

- **Day 4:** Integration testing
  - Create example projects
  - End-to-end testing

- **Day 5:** Documentation & release
  - Update README
  - Write polyglot upgrade guide
  - Release v6.2.0

---

## 🔧 Out of Scope (Future Work)

### NOT in v6.2.0:
- ❌ Reachability analysis for polyglot (too complex, language-specific)
- ❌ Shading detection for polyglot (not applicable to most ecosystems)
- ❌ Build system integration (npm scripts, etc.)
- ❌ Monorepo-specific features (lerna, yarn workspaces, etc.)

These can be considered for v6.3.0 or later if there's demand.

---

## 💡 Key Design Decisions

### 1. Ecosystem Detection
**Decision:** Auto-detect from package name format
**Rationale:** Most intuitive UX - users shouldn't have to specify `--ecosystem npm`
**Trade-off:** Some ambiguous cases (e.g., `express` could be PyPI or npm), but context usually makes it clear

### 2. Dependency File Updates
**Decision:** Use JSON/TOML parsing, not regex
**Rationale:** Preserve formatting, comments, avoid breaking manifests
**Trade-off:** More code, but much safer

### 3. Test Execution
**Decision:** Run package manager's default test command
**Rationale:** Respects project conventions, doesn't force a specific test framework
**Trade-off:** Requires test script to be configured in project

---

## 🚦 Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| deps.dev API rate limits | MEDIUM | Cache aggressively, implement backoff |
| Lockfile format changes | LOW | Parse JSON/YAML defensively, handle errors gracefully |
| Test commands vary by project | MEDIUM | Allow custom test commands in config, good error messages |
| Breaking change detection accuracy | MEDIUM | Parse GitHub releases conservatively, show warnings |

---

## 📦 Deliverables

### Code
- [ ] Ecosystem detection module
- [ ] Multi-ecosystem UpgradeAnalyzer
- [ ] 6 dependency updater implementations
- [ ] 6 test runner implementations
- [ ] Extended interactive fix UI
- [ ] 100+ unit tests

### Documentation
- [ ] Updated README with polyglot upgrade examples
- [ ] docs/polyglot/upgrade-intelligence.md
- [ ] docs/polyglot/interactive-fix.md
- [ ] Migration guide for users

### Testing
- [ ] 6 example projects (one per ecosystem)
- [ ] E2E test suite
- [ ] CI integration tests

---

## 🎉 Release Announcement (Draft)

```markdown
# BazBOM 6.2.0: Full Polyglot Parity 🌍

**The upgrade intelligence you love for Maven now works for npm, Python, Go, Rust, Ruby, and PHP!**

## What's New

### 🚀 Upgrade Intelligence for All Ecosystems
```bash
# npm packages
bazbom fix express --explain

# Python packages
bazbom fix django --explain

# Go modules
bazbom fix github.com/gin-gonic/gin --explain

# Rust crates
bazbom fix serde --explain
```

### 🎯 Interactive Fix for All Languages
```bash
bazbom fix --interactive
# Now handles npm, PyPI, Go, Rust, Ruby, PHP!
```

### ✨ Smart Ecosystem Detection
No more `--ecosystem` flags. BazBOM auto-detects from package name.

---

**Status:** PLANNING → IMPLEMENTATION STARTING NOW
**Last Updated:** 2025-11-11
**Release Date:** December 2025 (2 weeks)

---

*Making world-class upgrade intelligence available to ALL developers, regardless of language.*
```
