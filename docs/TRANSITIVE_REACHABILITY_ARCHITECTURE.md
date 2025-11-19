# Transitive Reachability Analysis - Architecture Design

**Status:** 🚧 IN PROGRESS (1/8 ecosystems complete)
**Last Updated:** 2025-11-18
**Goal:** True reachability analysis across package boundaries for ALL ecosystems

## Implementation Status

| Ecosystem | Status | Files | Validation |
|-----------|--------|-------|------------|
| **Rust/Cargo** | ✅ COMPLETE | `RUST_TRANSITIVE_REACHABILITY_COMPLETE.md` | Tested on 400+ dep monorepo |
| **Go/Go Modules** | ⏳ PENDING | - | - |
| **JavaScript/npm** | ⏳ PENDING | - | - |
| **Python/pip** | ⏳ PENDING | - | - |
| **Java/Maven/Gradle** | ⏳ PENDING | - | - |
| **Ruby/Bundler** | ⏳ PENDING | - | - |
| **PHP/Composer** | ⏳ PENDING | - | - |
| **Bazel** | ⏳ PENDING | - | - |

**See:** `docs/TRANSITIVE_REACHABILITY_ROADMAP.md` for detailed implementation plan.

## Core Requirements

1. ✅ Analyze application source code
2. ✅ Analyze ALL dependency source code (direct + transitive)
3. ✅ Build unified call graph spanning all packages
4. ✅ Trace reachability from app entrypoints through all dependencies
5. ✅ Map vulnerabilities to reachable/unreachable status

**Note:** All requirements proven working for Rust/Cargo. Architecture is validated and ready to replicate across other ecosystems.

## Unified Architecture

### Phase 1: Dependency Resolution

**For each ecosystem, resolve:**
- List of all installed dependencies (direct + transitive)
- Location of dependency source code
- Version information

**Ecosystem-Specific Approaches:**

| Ecosystem | Dependency Location | Resolution Method |
|-----------|-------------------|-------------------|
| **Rust** | `~/.cargo/registry/src/`, `target/` | Parse `Cargo.lock`, extract sources |
| **Python** | `site-packages/`, `venv/` | Parse installed packages, find .py files |
| **Go** | `$GOPATH/pkg/mod/` | Use `go list -m all`, locate sources |
| **JavaScript** | `node_modules/` | Parse `package-lock.json`, traverse tree |
| **Ruby** | `gems/` | Use `bundle list`, locate gem sources |
| **Java/Maven** | `~/.m2/repository/` | Parse `pom.xml`, download sources or decompile JARs |
| **Gradle** | Same as Maven | Parse `build.gradle`, resolve from Maven Central |
| **PHP** | `vendor/` | Parse `composer.lock`, locate sources |

### Phase 2: Multi-Package Call Graph Construction

**For each package (app + all deps):**

1. **Parse Source Files**
   - Use ecosystem-specific parser (syn, ast, acorn, etc.)
   - Extract all function definitions
   - Extract all function calls

2. **Build Package-Local Call Graph**
   - Node = Function (with full qualified name)
   - Edge = Function A calls Function B
   - Store in directed graph (petgraph)

3. **Identify Exported APIs**
   - Public functions (Rust `pub`, Python module-level)
   - Exported symbols (JavaScript `export`, Go capitalized)
   - Package entry points

**Data Structure:**
```rust
struct UnifiedCallGraph {
    // Package name -> Local call graph
    packages: HashMap<String, PackageCallGraph>,

    // Cross-package edges: (pkg_a::fn_x) -> (pkg_b::fn_y)
    cross_package_calls: Vec<(FunctionId, FunctionId)>,

    // Vulnerability locations: CVE -> List<FunctionId>
    vulnerability_map: HashMap<String, Vec<FunctionId>>,
}

struct FunctionId {
    package: String,
    module: Option<String>,
    function: String,
}
```

### Phase 3: Cross-Package Call Linking

**Link calls across package boundaries:**

1. **Identify Import Statements**
   ```rust
   use chrono::Utc;  // Links to chrono package
   import requests   # Links to requests package
   ```

2. **Resolve Function References**
   ```rust
   Utc::now()  // Resolves to chrono::Utc::now()
   requests.get()  # Resolves to requests.api.get()
   ```

3. **Create Cross-Package Edges**
   ```rust
   app::main() -> chrono::Utc::now()
   chrono::Utc::now() -> chrono::offset::Local::now()
   chrono::offset::Local::now() -> time::localtime_r()  // VULNERABLE!
   ```

### Phase 4: Transitive Reachability Traversal

**DFS/BFS from application entrypoints:**

```rust
fn compute_reachability(graph: &UnifiedCallGraph, entrypoints: &[FunctionId]) -> HashSet<FunctionId> {
    let mut reachable = HashSet::new();
    let mut stack = entrypoints.to_vec();

    while let Some(func) = stack.pop() {
        if !reachable.insert(func.clone()) {
            continue;  // Already visited
        }

        // Add all functions this function calls (same package)
        if let Some(pkg_graph) = graph.packages.get(&func.package) {
            for callee in pkg_graph.get_callees(&func) {
                stack.push(callee.clone());
            }
        }

        // Add all functions this function calls (cross-package)
        for (caller, callee) in &graph.cross_package_calls {
            if caller == &func {
                stack.push(callee.clone());
            }
        }
    }

    reachable
}
```

### Phase 5: Vulnerability Mapping

**Map CVEs to functions:**

```rust
fn map_vulnerabilities_to_reachability(
    vulns: &[Vulnerability],
    reachable_funcs: &HashSet<FunctionId>,
    vuln_map: &HashMap<String, Vec<FunctionId>>,
) -> HashMap<String, bool> {
    let mut results = HashMap::new();

    for vuln in vulns {
        // Get functions affected by this CVE
        let affected_funcs = vuln_map.get(&vuln.cve_id).unwrap_or(&vec![]);

        // Check if ANY affected function is reachable
        let is_reachable = affected_funcs.iter().any(|f| reachable_funcs.contains(f));

        results.insert(format!("{}@{}", vuln.package, vuln.version), is_reachable);
    }

    results
}
```

## Implementation Plan by Ecosystem

### 1. Rust (Highest Priority) - ✅ COMPLETE!

**Status:** PRODUCTION READY (2025-11-18)
**Actual Time:** ~8 hours
**Documentation:** `docs/RUST_TRANSITIVE_REACHABILITY_COMPLETE.md`

**Implementation:**
1. ✅ Parse `Cargo.lock` to get all dependencies
2. ✅ Locate source in `vendor/` or `~/.cargo/registry/src/`
3. ✅ Parse all .rs files in app + all deps using syn
4. ✅ Build unified call graphs (6,000+ functions)
5. ✅ Link `use` statements to external crates
6. ✅ Traverse and mark reachable

**Validation:**
- ✅ Minimal test: chrono → time transitive chain (2,841 functions)
- ✅ Real-world test: dyn-art/monorepo (397 deps, 6,372 functions)
- ✅ Performance: 30 seconds for 400+ dependencies

**Key Learnings:**
- Vendor directory support is essential (modern Cargo)
- Prevent recursive analysis with Cargo.lock detection
- Conservative over-approximation for uncertain cases
- Architecture proven scalable to large monorepos

### 2. Python (High Priority)

**Challenges:**
- Dynamic typing makes call resolution hard
- Can't statically determine all calls
- Need to handle imports carefully

**Steps:**
1. Find installed packages in site-packages/venv
2. Parse all .py files using ast module
3. Build call graphs with conservative over-approximation
4. Handle dynamic imports (`__import__`, `importlib`)
5. Mark functions as reachable if ANY path exists

**Estimated Time:** 12-16 hours

### 3. JavaScript/TypeScript (High Priority)

**Challenges:**
- Massive dependency trees (100s-1000s of packages)
- Dynamic requires
- CommonJS vs ES modules

**Steps:**
1. Parse package-lock.json for full dependency tree
2. Locate all packages in node_modules/
3. Parse using acorn/babel
4. Handle both require() and import
5. Build unified graph
6. Use caching to handle large trees

**Estimated Time:** 16-20 hours

### 4. Go (Medium Priority)

**Advantages:**
- Static typing helps
- `go list` provides dependency info
- Sources in GOPATH

**Steps:**
1. Run `go list -m all` to get dependencies
2. Locate sources in `$GOPATH/pkg/mod/`
3. Parse using go/parser
4. Build call graphs
5. Link imports
6. Traverse

**Estimated Time:** 10-12 hours

### 5. Ruby (Medium Priority)

**Challenges:**
- Dynamic typing
- Metaprogramming
- Multiple require styles

**Steps:**
1. Use `bundle list` to get all gems
2. Locate gem sources
3. Parse using parser gem
4. Conservative approximation for dynamic calls
5. Build and traverse graph

**Estimated Time:** 12-16 hours

### 6. Java/Maven/Gradle (High Priority - Enterprise)

**Challenges:**
- JARs need decompilation
- Reflection is common
- Large bytecode analysis

**Approaches:**
- **Option A:** Use source JARs when available
- **Option B:** Decompile with Procyon/FernFlower
- **Option C:** Analyze bytecode with ASM

**Steps:**
1. Parse pom.xml/build.gradle for all dependencies
2. Download sources or decompile JARs
3. Parse Java source or analyze bytecode
4. Build call graphs
5. Handle reflection conservatively
6. Traverse

**Estimated Time:** 20-24 hours (complex)

### 7. PHP (Lower Priority)

**Challenges:**
- Dynamic typing
- Include/require can be dynamic
- Composer

**Steps:**
1. Parse composer.lock
2. Locate sources in vendor/
3. Parse using nikic/php-parser
4. Conservative approximation
5. Build and traverse graph

**Estimated Time:** 12-16 hours

## Performance Optimizations

### 1. Caching
- Cache parsed call graphs by package@version
- Reuse across multiple scans
- Store in ~/.bazbom/reachability_cache/

### 2. Parallel Processing
- Parse packages in parallel
- Use rayon for Rust implementation
- Limit parallelism to avoid OOM

### 3. Pruning
- Only analyze packages with known vulnerabilities
- Skip test dependencies unless explicitly enabled
- Limit graph depth (configurable)

### 4. Incremental Analysis
- Detect which packages changed
- Reuse unchanged package graphs
- Only recompute cross-package links

## Accuracy vs Performance Trade-offs

### Conservative (Default)
- Over-approximate reachability
- Mark as reachable if ANY doubt
- Faster, fewer false negatives
- May have false positives

### Precise (Opt-in)
- Deep analysis including reflection/dynamic calls
- Slower but more accurate
- Use for critical applications

### Fast (Large repos)
- Only analyze direct dependencies
- Skip deep transitive analysis
- Fallback for 1000+ package repos

## Testing Strategy

### Unit Tests
- Test each ecosystem's dependency resolver
- Test call graph builder
- Test cross-package linker
- Test reachability algorithm

### Integration Tests
- Real apps with known transitive vulnerabilities
- Verify correct reachable/unreachable marking
- Measure performance on various repo sizes

### Validation Apps Needed

| Ecosystem | App with Transitive Vuln | Dependency Chain | Status |
|-----------|--------------------------|------------------|--------|
| **Rust** | App → chrono → time (vuln) | ✅ Created & Validated | ✅ COMPLETE |
| **Python** | App → requests → urllib3 (vuln) | Find or create | ⏳ Pending |
| **JavaScript** | App → axios → follow-redirects (vuln) | Find or create | ⏳ Pending |
| **Go** | App → pkg A → pkg B (vuln) | Create test app | ⏳ Pending |
| **Ruby** | App → gem A → gem B (vuln) | Create test app | ⏳ Pending |
| **Java** | App → lib A → lib B (vuln) | Create test app | ⏳ Pending |
| **PHP** | App → pkg A → pkg B (vuln) | Create test app | ⏳ Pending |

## Implementation Timeline

### Week 1: Foundation ✅ COMPLETE!
- [x] Design complete
- [x] Unified data structures
- [x] Dependency resolution framework
- [x] Rust implementation
- [x] Validation on real-world monorepo

### Week 2: Core Ecosystems (IN PROGRESS)
- [ ] Go implementation (NEXT UP)
- [ ] JavaScript implementation
- [ ] Python implementation

### Week 3: Enterprise & Final
- [ ] Java implementation
- [ ] Ruby implementation
- [ ] PHP implementation
- [ ] Bazel implementation
- [ ] Final validation

## Success Criteria

### Per-Ecosystem (Rust: ✅ COMPLETE)

1. ✅ Can analyze transitive dependencies (Rust: 397 deps analyzed)
2. ✅ Correctly identifies reachable vulnerabilities through 2+ layers (Rust: chrono → time proven)
3. ✅ Validates with real-world apps containing transitive vulns (Rust: dyn-art/monorepo)
4. ✅ Performance acceptable < 30s for typical app (Rust: 30s for 397 deps)
5. ✅ Documented accuracy metrics (Rust: see complete doc)

### Overall (1/8 complete)

- [x] Rust/Cargo
- [ ] Go/Go Modules (NEXT)
- [ ] JavaScript/npm
- [ ] Python/pip
- [ ] Java/Maven/Gradle
- [ ] Ruby/Bundler
- [ ] PHP/Composer
- [ ] Bazel

## Next Steps

1. ✅ ~~Implement Rust transitive reachability~~ **COMPLETE!**
2. ✅ ~~Create test app with chrono → time transitive vulnerability~~ **COMPLETE!**
3. ✅ ~~Validate it works end-to-end~~ **COMPLETE!**
4. ✅ ~~Test on real-world monorepo~~ **COMPLETE (dyn-art/monorepo)!**
5. **→ Implement Go/Go Modules (NEXT UP)**
6. Expand to JavaScript, Python
7. Continue through remaining ecosystems
8. Final cross-ecosystem validation

---

**Status:** Rust PRODUCTION READY - Architecture validated, ready for replication
**Last Updated:** 2025-11-18
**See:** `docs/TRANSITIVE_REACHABILITY_ROADMAP.md` for detailed plan
