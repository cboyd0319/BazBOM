# BazBOM 6.3.0 Roadmap: JavaScript/TypeScript Reachability Analysis

**Release Target:** January 2026 (3 weeks sprint)
**Mission:** Add world-class reachability analysis for JavaScript and TypeScript

**Part of the Full Polyglot Parity Initiative:**
- v6.2.0 - Upgrade Intelligence + Interactive Fixing ✅
- **v6.3.0** - JavaScript/TypeScript Reachability Analysis ← YOU ARE HERE
- v6.4.0 - Python + Go Reachability Analysis
- v6.5.0 - Rust + Ruby + PHP Reachability + Complete Parity

---

## 🎯 Goal

**Add the same bytecode-level reachability analysis that JVM has, but for JavaScript and TypeScript.**

### What Reachability Means

When BazBOM finds a vulnerability in a dependency, it answers:
- ❓ "Is the vulnerable code **actually used** by your application?"
- ✅ Reachable = You call it, directly or transitively → **FIX IMMEDIATELY**
- ❌ Unreachable = Dead code, not in call graph → **LOW PRIORITY**

This reduces alert fatigue by 70-90% and lets developers focus on real risks.

---

## 📊 Current State

### JVM Reachability (World-Class)
- ✅ Bytecode analysis using ASM library
- ✅ Call graph generation from entrypoints
- ✅ Transitive reachability tracking
- ✅ Handles reflection, dynamic proxies
- ✅ Vulnerability tagging (reachable/unreachable)

### JavaScript/TypeScript (Does Not Exist)
- ❌ No AST parsing
- ❌ No call graph generation
- ❌ No reachability analysis
- ❌ All vulnerabilities treated as equally urgent

---

## 🔧 Technical Approach

### Phase 1: AST Parsing (Week 1)

**Tool Choice:** Use SWC or Babel parser (via WASM or Node.js)

**Implementation:**
```rust
// New crate: bazbom-js-reachability
pub struct JavaScriptReachabilityAnalyzer {
    parser: SwcParser,
    call_graph: CallGraph,
    entrypoints: Vec<String>,
}

impl JavaScriptReachabilityAnalyzer {
    /// Parse JavaScript/TypeScript files and build call graph
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        // 1. Find all .js, .ts, .jsx, .tsx files
        // 2. Parse into AST
        // 3. Extract function calls
        // 4. Build call graph
        // 5. Mark reachable nodes from entrypoints
    }
}
```

**Challenges:**
- Dynamic imports: `import('./module.js')`
- Require statements: `require('module')`
- Module resolution (node_modules, package.json exports)
- CommonJS vs ESM

**Solution:**
- Use enhanced-resolve (webpack's resolver) via Node.js
- Handle both CommonJS and ESM
- Track dynamic imports as "potentially reachable"

### Phase 2: Call Graph Generation (Week 1-2)

**Entrypoints:**
- Main entry (package.json "main" field)
- Exported functions (package.json "exports")
- HTTP handlers (Express, Fastify, Next.js routes)
- Event handlers (React components, Vue components)

**Graph Construction:**
```rust
pub struct CallGraph {
    nodes: HashMap<FunctionId, FunctionNode>,
    edges: Vec<(FunctionId, FunctionId)>,
}

pub struct FunctionNode {
    name: String,
    file: PathBuf,
    line: usize,
    calls: Vec<FunctionId>,
    reachable: bool,
}
```

**Algorithm:**
1. Start from entrypoints
2. DFS traversal of call graph
3. Mark all visited nodes as reachable
4. Handle dynamic calls conservatively (mark as reachable)

### Phase 3: Vulnerability Mapping (Week 2)

**Map vulnerabilities to functions:**
```rust
pub struct VulnerabilityReachability {
    cve_id: String,
    package: String,
    version: String,
    vulnerable_functions: Vec<String>,  // e.g., ["express.Router.use"]
    reachable: bool,
    call_chain: Option<Vec<String>>,    // Path from entrypoint to vuln
}
```

**Data Sources:**
- OSV API (already have this)
- GitHub Security Advisories
- Manually curated function mappings for common packages

**Example:**
```
CVE-2024-1234 in express@4.17.0
Vulnerable function: express.Router.use()

Call chain:
  app.js:main()
    → routes/api.js:setupRoutes()
      → express.Router.use()  ← VULNERABLE!

Verdict: REACHABLE → HIGH PRIORITY
```

### Phase 4: Integration (Week 3)

**Update SCA output to include reachability:**
```bash
$ bazbom scan . --npm

📦 Found 45 vulnerabilities (23 reachable, 22 unreachable)

🔴 REACHABLE VULNERABILITIES (FIX THESE!)
  CVE-2024-1234 in express@4.17.0
    Function: express.Router.use()
    Call chain: app.js → routes/api.js → express.Router.use()
    Fix: Upgrade to express@4.18.0

🟡 UNREACHABLE VULNERABILITIES (LOW PRIORITY)
  CVE-2024-5678 in lodash@4.17.0
    Function: lodash.template()
    Not called by your code ✓
```

---

## 📋 Implementation Plan

### Week 1: AST Parsing & Module Resolution
- [ ] Create bazbom-js-reachability crate
- [ ] Integrate SWC parser (Rust native!)
- [ ] Implement module resolution (node_modules, package.json)
- [ ] Parse .js, .ts, .jsx, .tsx files
- [ ] Extract function definitions and calls
- [ ] Handle CommonJS + ESM

### Week 2: Call Graph & Reachability
- [ ] Build call graph data structure
- [ ] Identify entrypoints automatically
- [ ] DFS reachability algorithm
- [ ] Handle dynamic imports/requires
- [ ] Map vulnerabilities to functions
- [ ] Generate call chains

### Week 3: Integration & Testing
- [ ] Integrate with existing SCA pipeline
- [ ] Update SARIF output with reachability
- [ ] Test with real-world projects (Express, React, Next.js)
- [ ] Documentation and examples
- [ ] Performance optimization

---

## 🎯 Success Criteria

### Technical
- ✅ Parse 1000+ line JavaScript/TypeScript projects
- ✅ Resolve npm dependencies correctly
- ✅ Generate accurate call graphs (>90% precision)
- ✅ Identify reachable/unreachable vulnerabilities
- ✅ < 10 seconds for typical projects (< 100k LOC)

### User Experience
- ✅ Automatically detect JavaScript/TypeScript projects
- ✅ Zero configuration for standard project structures
- ✅ Clear, actionable output showing reachable vulns
- ✅ Call chains to help understand why something is reachable

---

## 🚧 Known Limitations

### What We WON'T Support in v6.3.0
- ❌ Webpack/Vite dynamic imports with variables: `import(`./${name}.js`)`
- ❌ eval() and new Function() (too dynamic)
- ❌ Monkey-patching and prototype pollution
- ❌ Browser-only code (we assume Node.js runtime)
- ❌ Minified/bundled code (analyze source, not dist)

These can be addressed in later versions if needed.

---

## 📚 Technical Deep Dive

### Why SWC Instead of Babel?

**SWC (Rust-native):**
- ✅ 20x faster than Babel
- ✅ No Node.js dependency
- ✅ Native Rust integration
- ✅ Full TypeScript support
- ✅ Battle-tested (used by Next.js, Deno, Parcel)

**Babel (JavaScript):**
- ❌ Requires Node.js runtime
- ❌ Slower parsing
- ❌ FFI overhead

**Decision:** Use SWC via the `swc_ecma_parser` crate.

### Module Resolution Strategy

**Node.js resolution algorithm:**
1. Check package.json "exports" field (modern)
2. Fall back to "main" field (legacy)
3. Resolve node_modules hierarchy
4. Handle .js, .ts, .jsx, .tsx extensions
5. Support index files (index.js, index.ts)

**Implementation:**
```rust
use oxc_resolver::Resolver;  // Fast Node.js resolver in Rust

pub struct ModuleResolver {
    resolver: Resolver,
    project_root: PathBuf,
}

impl ModuleResolver {
    pub fn resolve(&self, specifier: &str, from: &Path) -> Result<PathBuf> {
        // Use oxc_resolver for Node.js-compatible resolution
        self.resolver.resolve(from, specifier)
    }
}
```

---

## 🔬 Example Analysis

### Sample Project Structure
```
my-app/
├── package.json
├── src/
│   ├── index.js         (entrypoint)
│   ├── routes/
│   │   └── api.js       (calls vulnerable express function)
│   └── utils/
│       └── helpers.js   (calls lodash, but not vulnerable function)
└── node_modules/
    ├── express@4.17.0  (has CVE-2024-1234 in Router.use())
    └── lodash@4.17.0   (has CVE-2024-5678 in template())
```

### Analysis Output
```
🔍 Analyzing JavaScript project at /path/to/my-app

📂 Discovered:
  - 3 source files
  - 45 dependencies
  - 2 vulnerabilities

🕸️  Building call graph...
  - Entrypoint: src/index.js:main()
  - 12 functions discovered
  - 8 reachable functions

🔴 REACHABLE VULNERABILITY (HIGH PRIORITY!)
  CVE-2024-1234 in express@4.17.0

  Vulnerable function: express.Router.use()

  Call chain:
    src/index.js:main() [line 10]
      → src/routes/api.js:setupRoutes() [line 5]
        → express.Router.use() [line 15]  ← VULNERABLE!

  EPSS: 78.5% (high exploitation probability)
  Fix: Upgrade to express@4.18.0

  📝 Recommendation: This vulnerability is actively used by your
      code and should be fixed immediately.

🟢 UNREACHABLE VULNERABILITY (LOW PRIORITY)
  CVE-2024-5678 in lodash@4.17.0

  Vulnerable function: lodash.template()

  Status: NOT CALLED by your application ✓

  Your code only uses: lodash.map(), lodash.filter()
  The vulnerable function template() is never imported or called.

  📝 Recommendation: This can be fixed during regular dependency
      updates. Not urgent.
```

---

## 🎯 Definition of Done

- [ ] SWC parser integrated
- [ ] Module resolution working (node_modules, package.json)
- [ ] Call graph generation for .js, .ts, .jsx, .tsx
- [ ] Reachability algorithm implemented
- [ ] Vulnerability-to-function mapping
- [ ] Integration with existing SCA pipeline
- [ ] SARIF output includes reachability info
- [ ] Documentation and examples
- [ ] 20+ passing tests
- [ ] Performance: < 10s for 100k LOC projects

---

## 📦 Deliverables

### Code
- [ ] `crates/bazbom-js-reachability/` - New crate
- [ ] Integration with `crates/bazbom-polyglot/`
- [ ] Updated `bazbom scan` command
- [ ] SARIF output with reachability

### Documentation
- [ ] `docs/polyglot/javascript-reachability.md`
- [ ] Updated README with JS/TS reachability
- [ ] Example projects in `examples/js-reachability/`

### Testing
- [ ] Unit tests for parser
- [ ] Integration tests with real projects (Express, React)
- [ ] Performance benchmarks

---

**Status:** PLANNED
**Start Date:** January 2026
**Timeline:** 3 weeks
**Next:** v6.4.0 - Python + Go Reachability

---

*Making JavaScript/TypeScript supply chain security as rigorous as JVM.*
