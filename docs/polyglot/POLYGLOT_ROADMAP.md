# 🌍 BazBOM Polyglot Support - Implementation Roadmap

## Vision: JVM-First, Polyglot-Complete

**Philosophy:** World-class depth for JVM + comprehensive support for all major ecosystems

---

## ✅ Current Status

### Implemented Features

#### 1. **Ecosystem Detection** ✅
Auto-detect all ecosystems in a directory tree:
- **Node.js/npm** 📦 - `package.json`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`
- **Python** 🐍 - `requirements.txt`, `pyproject.toml`, `Pipfile`, `poetry.lock`
- **Go** 🐹 - `go.mod`, `go.sum`
- **Rust** 🦀 - `Cargo.toml`, `Cargo.lock`
- **Ruby** 💎 - `Gemfile`, `Gemfile.lock`
- **PHP** 🐘 - `composer.json`, `composer.lock`

**Smart Detection:**
- Skips `node_modules`, `.git`, `target`, `dist`, `build`, `__pycache__`, `.venv`
- Deduplicates ecosystems in the same directory
- Prefers lockfiles over manifest files for accuracy

#### 2. **Node.js/npm Parser** ✅ (Phase 1 Complete)
- Full `package-lock.json` v7+ support
- Legacy `package-lock.json` v6 support with recursive dependencies
- Scoped package support (`@types/node`, `@angular/core`, etc.)
- Fallback to `package.json` when no lockfile present
- yarn.lock and pnpm-lock.yaml stubs (TODO: full implementation)

---

## 🚧 In Progress

### Phase 2: Python Ecosystem
**Status:** Stub created, implementation pending

**Features to implement:**
- `requirements.txt` parser (simple format: `package==version`)
- `poetry.lock` parser (TOML format)
- `Pipfile.lock` parser (JSON format)
- `setup.py` analyzer (extract `install_requires`)
- `pyproject.toml` parser (PEP 518)

**Implementation Strategy:**
```rust
// Parse requirements.txt line by line
// Format: package==version or package>=version
async fn parse_requirements_txt(path: &Path) -> Result<Vec<Package>>

// Parse poetry.lock (TOML)
async fn parse_poetry_lock(path: &Path) -> Result<Vec<Package>>
```

### Phase 3: Go Modules
**Status:** Stub created, implementation pending

**Features to implement:**
- `go.mod` parser (module dependencies)
- `go.sum` parser (checksums and exact versions)
- Indirect dependencies detection
- Replace directives handling

**Implementation Strategy:**
```rust
// Parse go.mod (simple line-based format)
// require github.com/user/repo v1.2.3
async fn parse_go_mod(path: &Path) -> Result<Vec<Package>>
```

### Phase 4: Rust Cargo
**Status:** Stub created, using `cargo-lock` crate

**Features to implement:**
- Use existing `cargo-lock` crate for parsing
- Extract all dependencies with versions
- Dependency tree analysis

**Implementation:**
```rust
use cargo_lock::Lockfile;

async fn parse_cargo_lock(path: &Path) -> Result<Vec<Package>> {
    let lockfile = Lockfile::load(path)?;
    // Convert to Package format
}
```

### Phase 5: Ruby & PHP
**Status:** Stubs created, lower priority

---

## 🎯 Phase 6: Vulnerability Scanning (Critical)

### Integration with OSV (Open Source Vulnerabilities)
**API:** https://osv.dev/

**Features:**
- Query OSV API for each package
- Map CVEs to packages across all ecosystems
- Cache results for performance

**Implementation:**
```rust
async fn query_osv(ecosystem: &str, package: &str, version: &str) -> Result<Vec<Vulnerability>> {
    // POST to https://api.osv.dev/v1/query
    // {
    //   "package": {"ecosystem": "npm", "name": "express"},
    //   "version": "4.17.0"
    // }
}
```

**OSV Ecosystem Names:**
- npm → `npm`
- Python → `PyPI`
- Go → `Go`
- Rust → `crates.io`
- Ruby → `RubyGems`
- PHP → `Packagist`

### Integration with GitHub Advisory Database
**API:** https://github.com/advisories

**Features:**
- Query GitHub Security Advisories
- Cross-reference with OSV data
- Add GitHub-specific metadata (GHSA-xxxx IDs)

---

## 🎨 Phase 7: Unified SBOM Generation

### Features
1. **Multi-Ecosystem SPDX**
   - Single SPDX document with packages from all ecosystems
   - Proper PURL (Package URL) format for each ecosystem
   - Namespace support (e.g., `@types` for npm, `github.com/user` for Go)

2. **CycloneDX Support**
   - Generate CycloneDX 1.5 format
   - Component types for each ecosystem
   - Dependency relationships

3. **Dependency Graph**
   - Unified dependency tree across all languages
   - Transitive dependency resolution
   - Circular dependency detection

---

## 📊 Phase 8: Intelligence Features for All Languages

Apply existing BazBOM intelligence to ALL ecosystems:

### 1. Quick Wins Analysis ⚡
- Identify easy patches across all languages
- Time estimates per ecosystem
- Non-breaking change detection

### 2. Prioritized Action Plan 📋
- P0-P4 classification across all ecosystems
- EPSS + KEV integration for all CVEs
- Language-agnostic priority algorithm

### 3. Breaking Change Detection 🔧
- Semantic versioning analysis per ecosystem
- npm: Check major version bumps
- Python: Check for breaking changes in changelogs
- Go: v2+ module path changes
- Rust: Major version bumps

### 4. Copy-Paste Remediation 📋
**Per-Ecosystem Formats:**

**npm:**
```json
"dependencies": {
  "express": "^4.19.0"
}
```

**Python:**
```
express==2.0.0
```

**Go:**
```
go get github.com/user/repo@v1.2.3
```

**Rust:**
```toml
[dependencies]
serde = "1.0"
```

---

## 🧪 Phase 9: Testing & Validation

### Test Scenarios
1. **Monorepo with Multiple Ecosystems**
   ```
   /monorepo
   ├── frontend/ (Node.js)
   ├── backend/ (Python)
   ├── services/
   │   ├── auth-service/ (Go)
   │   └── data-service/ (Rust)
   └── scripts/ (Python)
   ```

2. **Polyglot Container**
   - Build container with multiple languages
   - Run both `bazbom scan` and `bazbom container-scan`
   - Verify unified results

3. **Large Monorepo (5000+ files)**
   - Performance benchmarks
   - Memory usage
   - Scan time per ecosystem

---

## 📈 Implementation Priority

| Phase | Feature | Priority | Effort | Status |
|-------|---------|----------|--------|--------|
| 1 | Ecosystem Detection | P0 | Small | ✅ Done |
| 2 | npm Parser | P0 | Medium | ✅ Done |
| 3 | OSV Integration | P0 | Medium | 🚧 Next |
| 4 | Python Parser | P1 | Medium | 📝 Planned |
| 5 | Go Parser | P1 | Small | 📝 Planned |
| 6 | Rust Parser | P1 | Small | 📝 Planned |
| 7 | Unified SBOM | P0 | Large | 📝 Planned |
| 8 | Intelligence Features | P1 | Large | 📝 Planned |
| 9 | Ruby/PHP Parsers | P2 | Medium | 📝 Planned |
| 10 | Testing | P0 | Large | 📝 Planned |

---

## 🎯 Success Criteria

### Must Have (MVP)
- ✅ Auto-detect 6 ecosystems (npm, Python, Go, Rust, Ruby, PHP)
- ✅ Parse npm lockfiles accurately
- 🚧 Query OSV for vulnerabilities across all ecosystems
- 🚧 Generate unified SPDX SBOM
- 🚧 Apply priority classification (P0-P4) to all languages

### Should Have
- Parse Python requirements/poetry lockfiles
- Parse Go modules
- Parse Rust Cargo.lock
- Breaking change detection per ecosystem
- Copy-paste remediation for all languages

### Nice to Have
- yarn.lock and pnpm-lock.yaml full support
- Ruby/PHP full support
- GitHub Advisory integration
- Dependency graph visualization across languages

---

## 🚀 Usage Example (Future)

```bash
# Scan entire monorepo (all ecosystems)
bazbom scan /path/to/monorepo

# Output:
# 📦 Detected 4 ecosystems:
#   📦 Node.js/npm (3 projects)
#   🐍 Python (2 projects)
#   🐹 Go (5 services)
#   🦀 Rust (1 library)
#
# 📊 Total packages: 1,247
#   npm: 856 packages
#   Python: 234 packages
#   Go: 89 packages
#   Rust: 68 packages
#
# 🔍 Total vulnerabilities: 42
#   P0 (urgent): 3
#   P1 (high): 12
#   P2 (medium): 18
#   P3 (low): 9
#
# ⚡ Quick Wins: 15 patches (45 minutes)
#   npm: 8 patches
#   Python: 5 patches
#   Go: 2 patches

# Filter by ecosystem
bazbom scan . --ecosystem npm
bazbom scan . --ecosystem python

# Show only P0 across all languages
bazbom scan . --show p0

# Generate unified SBOM
bazbom scan . --format spdx --output sbom.json
```

---

## 📚 Architecture

```
bazbom-polyglot/
├── detection.rs          # Ecosystem detection (✅ Done)
├── ecosystems.rs         # Common types (✅ Done)
├── parsers/
│   ├── npm.rs           # Node.js parser (✅ Done)
│   ├── python.rs        # Python parser (🚧 Stub)
│   ├── go.rs            # Go parser (🚧 Stub)
│   ├── rust.rs          # Rust parser (🚧 Stub)
│   ├── ruby.rs          # Ruby parser (🚧 Stub)
│   └── php.rs           # PHP parser (🚧 Stub)
├── vulnerabilities.rs    # OSV/GitHub Advisory (📝 TODO)
└── sbom.rs              # Unified SBOM generation (📝 TODO)
```

---

## 🎉 Impact

**Before:**
- JVM only (Java/Kotlin/Scala)
- Polyglot monorepos required multiple tools

**After:**
- **Single tool for entire monorepo**
- Unified vulnerability view across all languages
- Consistent prioritization (P0-P4) across ecosystems
- One SBOM for everything
- Same intelligence features for all languages

**Result:** BazBOM becomes the **comprehensive** security tool for polyglot monorepos while maintaining its JVM-first depth and quality.

---

**Status:** Phase 1 & 2 Complete | Phase 3-10 In Progress

**Next Steps:**
1. Implement OSV vulnerability scanning
2. Complete Python parser
3. Generate unified SBOM
4. Apply intelligence features to all languages
