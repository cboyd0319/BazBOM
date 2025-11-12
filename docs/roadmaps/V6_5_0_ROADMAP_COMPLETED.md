# BazBOM 6.5.0 Roadmap: Complete Polyglot Parity

**Release Target:** March 2026 (4 weeks sprint)
**Mission:** Complete reachability analysis for Rust, Ruby, PHP + achieve FULL feature parity across ALL ecosystems

**Part of the Full Polyglot Parity Initiative:**
- v6.2.0 - Upgrade Intelligence + Interactive Fixing ✅
- v6.3.0 - JavaScript/TypeScript Reachability Analysis ✅
- v6.4.0 - Python + Go Reachability Analysis ✅
- **v6.5.0** - Rust + Ruby + PHP Reachability + Complete Parity ← YOU ARE HERE 🎯

---

## 🎯 Mission: Complete Feature Parity

**After v6.5.0, BazBOM will have IDENTICAL capabilities for ALL supported languages:**

| Feature | JVM | JS/TS | Python | Go | **Rust** | **Ruby** | **PHP** |
|---------|-----|-------|--------|----|----|------|-----|
| SBOM Generation | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vulnerability Scanning | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Upgrade Intelligence | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Interactive Fixing | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Reachability Analysis** | ✅ | ✅ | ✅ | ✅ | 🚧 | 🚧 | 🚧 |
| **Developer Experience** | ✅ | ✅ | ✅ | ✅ | 🚧 | 🚧 | 🚧 |

---

## 🦀 Part 1: Rust Reachability Analysis

### Why Rust is Special

Rust is **the easiest** language for reachability analysis:
- ✅ Fully static (no reflection, no eval)
- ✅ Excellent compiler integration (MIR, LLVM IR)
- ✅ Explicit trait implementations
- ✅ No runtime metaprogramming

**We can achieve near-perfect accuracy (>98%).**

### Technical Approach

**Option 1: Use MIR (Mid-level Intermediate Representation)**
```rust
// Access Rust compiler internals
use rustc_middle::mir;
use rustc_interface::interface;

pub struct RustReachabilityAnalyzer {
    mir_bodies: HashMap<DefId, mir::Body>,
    call_graph: CallGraph,
}

impl RustReachabilityAnalyzer {
    /// Analyze using rustc's MIR
    pub fn analyze_with_mir(&mut self, crate_root: &Path) -> Result<ReachabilityReport> {
        // 1. Invoke rustc to get MIR
        // 2. Extract call graph from MIR
        // 3. Track trait method dispatch
        // 4. Map to source locations
    }
}
```

**Option 2: Static AST Analysis**
```rust
use syn::{parse_file, Item, Expr};

// Parse Rust source files
let ast = parse_file(&source)?;

// Extract function calls
for item in ast.items {
    match item {
        Item::Fn(func) => analyze_function_body(&func),
        Item::Impl(impl_block) => analyze_trait_impl(&impl_block),
        _ => {}
    }
}
```

**Decision:** Start with **Option 2 (AST)** for speed, add **Option 1 (MIR)** for precision in v6.6.0.

### Implementation (Week 1)

**Entrypoints:**
- `fn main()` in binary crates
- `#[test]` functions
- Web frameworks: `#[actix_web::main]`, `#[tokio::main]`
- Benchmarks: `#[bench]`

**Call Graph:**
```rust
impl RustReachabilityAnalyzer {
    fn extract_calls(&self, func: &syn::ItemFn) -> Vec<FunctionCall> {
        let mut calls = vec![];

        for stmt in &func.block.stmts {
            // Direct calls: some_func()
            // Method calls: obj.method()
            // Trait methods: trait_obj.trait_method()
            // Macro invocations: println!(), vec![]
        }

        calls
    }

    fn resolve_trait_dispatch(&self, call: &Expr) -> Vec<DefId> {
        // For trait methods, find all implementations
        // Example: Iterator::map() → Vec::map, Option::map, etc.
    }
}
```

**Macro Handling:**
- `println!()`, `eprintln!()` → standard library, always reachable
- `vec!()`, `format!()` → expand macros
- Custom macros → conservative analysis

### Output Example

```bash
$ bazbom scan . --rust

🦀 Rust Reachability Analysis

📦 Found 8 vulnerabilities (3 reachable, 5 unreachable)

🔴 REACHABLE VULNERABILITY
  RUSTSEC-2024-1234 in tokio@1.25.0

  Vulnerable function: tokio::runtime::Runtime::block_on()

  Call chain:
    src/main.rs:main() [line 10]
      → src/server.rs:start_server() [line 45]
        → tokio::runtime::Runtime::block_on()  ← VULNERABLE!

  Risk: Panic in async context
  Fix: Upgrade to tokio@1.29.0

🟢 UNREACHABLE VULNERABILITIES
  RUSTSEC-2024-5678 in regex@1.7.0

  Vulnerable function: regex::Regex::is_match_at()

  Status: NOT CALLED by your application ✓

  Your code uses: Regex::is_match(), Regex::captures()
  The vulnerable internal function is_match_at() is not exposed.
```

---

## 💎 Part 2: Ruby Reachability Analysis

### Why Ruby is Hard

Ruby is **one of the hardest** languages for static analysis:
- ❌ Extreme dynamism (`define_method`, `method_missing`)
- ❌ Monkey-patching (can redefine any method at runtime)
- ❌ `eval`, `instance_eval`, `class_eval`
- ❌ Meta-programming (DSLs like Rails, RSpec)

**Conservative approach: >70% accuracy is realistic.**

### Technical Approach

**Parser: Ruby AST (RubyParser gem or tree-sitter)**

```rust
use tree_sitter::{Parser, Language};

extern "C" { fn tree_sitter_ruby() -> Language; }

pub struct RubyReachabilityAnalyzer {
    parser: Parser,
    call_graph: CallGraph,
    dynamic_code_warnings: Vec<String>,
}
```

### Implementation (Week 2)

**Entrypoints:**
- Rails controllers: `def index`, `def show`, etc.
- Rake tasks: `task :name do ... end`
- RSpec tests: `describe` / `it` blocks
- Sinatra routes: `get '/path' do ... end`

**Challenges:**

1. **Dynamic Method Definition**
```ruby
# How do we analyze this statically?
define_method("user_#{action}") do
  # ...
end
```
**Solution:** Conservatively mark all methods as potentially reachable if `define_method` is used.

2. **Monkey-Patching**
```ruby
class String
  def some_new_method
    # patching built-in class!
  end
end
```
**Solution:** Track monkey-patches and warn user. Mark patched methods as reachable.

3. **`method_missing`**
```ruby
def method_missing(name, *args)
  # Catch-all for any method call
end
```
**Solution:** If `method_missing` exists, conservatively assume ALL methods could be called.

**Conservative Rules:**
- Any use of `eval`, `instance_eval`, `class_eval` → entire gem marked reachable
- `method_missing` present → all methods in that class potentially reachable
- `define_method` → track if possible, otherwise mark all as reachable
- Rails magic (`has_many`, `belongs_to`) → parse Rails DSL specifically

### Output Example

```bash
$ bazbom scan . --ruby

💎 Ruby Reachability Analysis

📦 Found 18 vulnerabilities (14 reachable, 4 unreachable)

🔴 REACHABLE VULNERABILITY
  CVE-2024-1234 in rails@6.1.0

  Vulnerable method: ActiveRecord::Base.where()

  Call chain:
    app/controllers/users_controller.rb:index [line 10]
      → app/models/user.rb:User.search() [line 45]
        → ActiveRecord::Base.where()  ← VULNERABLE!

  Risk: SQL Injection
  Fix: Upgrade to rails@6.1.7

⚠️  DYNAMIC CODE DETECTED
  File: lib/dynamic_loader.rb:23
  Code: eval(config_string)

  Warning: Dynamic code prevents accurate analysis.
  All dependencies conservatively marked as reachable.

  Recommendation: Refactor to avoid eval()

⚠️  MONKEY-PATCHING DETECTED
  File: config/initializers/string_extensions.rb:5
  Code: class String; def custom_method; ...; end; end

  Warning: Patching core classes affects reachability.
```

---

## 🐘 Part 3: PHP Reachability Analysis

### Why PHP is Moderate Difficulty

PHP is **moderately dynamic**:
- ❌ Variable functions: `$func()`
- ❌ `eval()` and `include` with variables
- ✅ But most modern PHP (8.0+) is fairly static
- ✅ Symfony/Laravel follow patterns that are analyzable

**Target accuracy: >85%**

### Technical Approach

**Parser: PHP Parser (PHP-Parser via PHP or tree-sitter)**

```rust
pub struct PhpReachabilityAnalyzer {
    parser: PhpParser,
    call_graph: CallGraph,
    autoload_map: HashMap<String, PathBuf>,  // PSR-4 autoloading
}
```

### Implementation (Week 3)

**Entrypoints:**
- Symfony controllers: `#[Route]` attributes
- Laravel routes: defined in `routes/web.php`
- WordPress hooks: `add_action`, `add_filter`
- Composer scripts

**PSR-4 Autoloading:**
```rust
impl PhpReachabilityAnalyzer {
    /// Resolve class names using composer.json autoload
    fn resolve_class(&self, class_name: &str) -> Result<PathBuf> {
        // Parse composer.json "autoload" section
        // Map namespace to directory
        // Example: App\Http\Controllers\UserController
        //   → app/Http/Controllers/UserController.php
    }
}
```

**Challenges:**

1. **Variable Functions**
```php
$function_name = 'some_func';
$function_name();  // Dynamic call
```
**Solution:** Track string assignments when possible, otherwise mark as dynamic.

2. **Dynamic Includes**
```php
include $_GET['file'];  // VERY BAD!
```
**Solution:** Warn loudly, mark as security issue, assume all code reachable.

3. **Magic Methods**
```php
public function __call($name, $args) {
  // Catch undefined methods
}
```
**Solution:** Similar to Ruby's `method_missing`.

### Output Example

```bash
$ bazbom scan . --php

🐘 PHP Reachability Analysis

📦 Found 12 vulnerabilities (7 reachable, 5 unreachable)

🔴 REACHABLE VULNERABILITY
  CVE-2024-1234 in symfony/http-kernel@5.4.0

  Vulnerable method: Symfony\Component\HttpKernel\HttpKernel::handle()

  Call chain:
    public/index.php [line 25]
      → src/Kernel.php:handle() [line 18]
        → Symfony\Component\HttpKernel\HttpKernel::handle()  ← VULNERABLE!

  Risk: Request smuggling
  Fix: Upgrade to symfony/http-kernel@5.4.20

⚠️  VARIABLE FUNCTION CALL
  File: src/DynamicLoader.php:42
  Code: $handler_name();

  Warning: Variable function calls prevent accurate analysis.
  Marking potential call targets as reachable.
```

---

## 🎯 Part 4: Complete Parity Features (Week 4)

### Unified Developer Experience

**Goal:** IDENTICAL UX regardless of language.

#### 1. Unified CLI
```bash
# Same command works for ALL languages
bazbom scan .

# Auto-detects:
# - Java/Kotlin/Scala → JVM analysis
# - JavaScript/TypeScript → JS analysis
# - Python → Python analysis
# - Go → Go analysis
# - Rust → Rust analysis
# - Ruby → Ruby analysis
# - PHP → PHP analysis

# Polyglot monorepo? Scans ALL ecosystems!
```

#### 2. Unified Output Format
```bash
📊 BazBOM Analysis Results

Ecosystems detected: JVM, JavaScript, Python

🔴 CRITICAL VULNERABILITIES (REACHABLE)
  3 in JVM dependencies
  2 in npm packages
  1 in Python packages

🟡 HIGH VULNERABILITIES (UNREACHABLE)
  12 in JVM dependencies
  8 in npm packages
  5 in Python packages

📈 Reachability reduced alerts by 78%
    Without reachability: 31 vulnerabilities to fix
    With reachability:     6 vulnerabilities to fix

💡 Focus on the 6 reachable vulnerabilities first!
```

#### 3. Cross-Language Call Chains

**Future enhancement:** Track calls across language boundaries!

```bash
# Example: JVM → JavaScript via GraalVM
Java:com.example.Main.main()
  → GraalVM:eval("javascript_code")
    → JavaScript:processData()
      → npm:vulnerable-package  ← VULNERABLE!

# Example: Python → C extension
Python:main.py:process()
  → C Extension:numpy.array.dot()
    → Vulnerable C code  ← VULNERABLE!
```

#### 4. Unified SARIF Output

**All languages produce identical SARIF format:**
```json
{
  "runs": [{
    "tool": {
      "driver": {
        "name": "BazBOM",
        "version": "6.5.0",
        "informationUri": "https://bazbom.dev"
      }
    },
    "results": [{
      "ruleId": "CVE-2024-1234",
      "level": "error",
      "message": {
        "text": "Reachable vulnerability in express@4.17.0"
      },
      "locations": [{
        "physicalLocation": {
          "artifactLocation": {
            "uri": "src/routes/api.js"
          },
          "region": {
            "startLine": 42
          }
        }
      }],
      "properties": {
        "reachable": true,
        "callChain": ["app.js:main", "routes/api.js:setupRoutes", "express.Router.use"],
        "epss": 0.85,
        "cisaKev": true
      }
    }]
  }]
}
```

---

## 📋 Complete Timeline (4 Weeks)

### Week 1: Rust Reachability
- Days 1-2: AST parsing with syn crate
- Days 3-4: Call graph with trait resolution
- Day 5: Testing and integration

### Week 2: Ruby Reachability
- Days 1-2: Ruby AST parsing
- Days 3-4: Rails/RSpec-specific analysis
- Day 5: Handle dynamic code conservatively

### Week 3: PHP Reachability
- Days 1-2: PHP AST parsing + PSR-4 resolution
- Days 3-4: Symfony/Laravel patterns
- Day 5: Integration and testing

### Week 4: Unified Experience
- Days 1-2: Unified CLI and output
- Day 3: Cross-language integration
- Days 4-5: Documentation, polish, celebration 🎉

---

## 🎯 Success Criteria

### Technical
- ✅ Rust reachability: >98% accuracy
- ✅ Ruby reachability: >70% accuracy (dynamic nature limits this)
- ✅ PHP reachability: >85% accuracy
- ✅ Unified CLI works for polyglot monorepos
- ✅ SARIF output consistent across all languages
- ✅ Performance: < 30 seconds for 500k LOC polyglot project

### Feature Parity
- ✅ **ALL** 7 ecosystems have:
  - SBOM generation
  - Vulnerability scanning
  - Upgrade intelligence
  - Interactive fixing
  - Reachability analysis
- ✅ **IDENTICAL** developer experience
- ✅ **IDENTICAL** output formats

---

## 🎉 Launch Announcement (Draft)

```markdown
# BazBOM 6.5.0: Complete Polyglot Parity Achieved 🎯

**We did it. TRUE polyglot parity.**

BazBOM now provides **world-class** supply chain security for:
- ☕ JVM (Java/Kotlin/Scala)
- 🟨 JavaScript/TypeScript
- 🐍 Python
- 🐹 Go
- 🦀 Rust
- 💎 Ruby
- 🐘 PHP

## What "Complete Parity" Means

Every single ecosystem gets:
- ✅ SBOM generation
- ✅ Vulnerability scanning with EPSS + CISA KEV
- ✅ Upgrade intelligence (what breaks BEFORE you upgrade)
- ✅ Interactive fixing with test-before-commit
- ✅ **Reachability analysis** (is the vuln actually used?)

## Reachability Changes Everything

Before:
```
❌ 237 vulnerabilities found
   Developer: "I don't have time to fix all these..."
```

After:
```
✅ 237 vulnerabilities found (28 reachable, 209 unreachable)
   Focus on these 28 FIRST. The rest can wait.

   Reachability reduced your workload by 88% 🎉
```

## Polyglot Monorepos Just Work

```bash
$ cd my-giant-monorepo
$ bazbom scan .

📊 Analyzing polyglot monorepo...
   - 45 Java services
   - 23 TypeScript microservices
   - 12 Python data pipelines
   - 8 Go CLIs
   - 3 Rust crates

📦 Results:
   JVM: 45 vulns (12 reachable)
   JavaScript: 123 vulns (18 reachable)
   Python: 67 vulns (9 reachable)
   Go: 8 vulns (2 reachable)
   Rust: 3 vulns (0 reachable)

🎯 Total: 41 reachable vulnerabilities to fix
   (vs 246 without reachability = 83% reduction!)
```

## The Vision

**Make supply chain security something developers WANT, not dread.**

With complete polyglot parity, BazBOM is the ONLY tool that gives you:
- The same rigorous analysis regardless of language
- Reachability to cut through the noise
- Upgrade intelligence so you know what breaks
- A UX that doesn't make you want to quit programming

---

**Upgrade today:** `cargo install bazbom`
**Read the docs:** https://docs.bazbom.dev
```

---

## 📦 Deliverables

### Code
- [ ] `crates/bazbom-rust-reachability/`
- [ ] `crates/bazbom-ruby-reachability/`
- [ ] `crates/bazbom-php-reachability/`
- [ ] Unified CLI with auto-detection
- [ ] Cross-language SARIF output

### Documentation
- [ ] `docs/polyglot/rust-reachability.md`
- [ ] `docs/polyglot/ruby-reachability.md`
- [ ] `docs/polyglot/php-reachability.md`
- [ ] `docs/polyglot/COMPLETE_PARITY.md` - The big announcement!
- [ ] Updated README showcasing all languages

### Testing
- [ ] 50+ tests per new language
- [ ] Polyglot monorepo integration tests
- [ ] Performance benchmarks
- [ ] Real-world project testing

---

**Status:** PLANNED
**Start Date:** March 2026
**Timeline:** 4 weeks
**Next:** v7.0 - GitHub Marketplace with complete polyglot support!

---

*The finish line. Complete feature parity for ALL developers, regardless of language choice.*

**🎯 MISSION ACCOMPLISHED 🎯**
