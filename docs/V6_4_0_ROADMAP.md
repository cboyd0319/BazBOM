# BazBOM 6.4.0 Roadmap: Python + Go Reachability Analysis

**Release Target:** February 2026 (3 weeks sprint)
**Mission:** Add world-class reachability analysis for Python and Go

**Part of the Full Polyglot Parity Initiative:**
- v6.2.0 - Upgrade Intelligence + Interactive Fixing ✅
- v6.3.0 - JavaScript/TypeScript Reachability Analysis ✅
- **v6.4.0** - Python + Go Reachability Analysis ← YOU ARE HERE
- v6.5.0 - Rust + Ruby + PHP Reachability + Complete Parity

---

## 🎯 Goal

**Add reachability analysis for Python and Go with the same rigor as JVM and JavaScript.**

These languages present unique challenges:
- **Python**: Extremely dynamic (monkey-patching, metaprogramming, eval)
- **Go**: Static but with interfaces, reflection, and goroutines

---

## 🐍 Part 1: Python Reachability Analysis

### Technical Approach

**Parser: Python AST (built-in)**

Python has excellent AST support in Rust via `RustPython`:
```rust
use rustpython_parser::ast;

pub struct PythonReachabilityAnalyzer {
    ast_cache: HashMap<PathBuf, ast::Module>,
    call_graph: CallGraph,
    entrypoints: Vec<String>,
}
```

### Challenges

1. **Dynamic Nature**
   - `getattr(obj, "method_name")()` - runtime method resolution
   - `exec()` and `eval()` - arbitrary code execution
   - Metaclasses and `__getattribute__`
   - Monkey-patching

2. **Module System**
   - Relative imports: `from . import module`
   - Absolute imports: `from package.module import func`
   - Star imports: `from module import *`
   - Conditional imports (inside `if` statements)

3. **Type Ambiguity**
   - Duck typing - no static type information
   - Union types - function could return multiple types
   - Need type inference or hints

### Solution Strategy

**Conservative Analysis:**
- Assume dynamic calls are reachable (better safe than sorry)
- Use type hints when available (PEP 484)
- Track common patterns (Django views, Flask routes)
- Warn user about highly dynamic code

**Implementation Plan:**

#### Week 1: AST Parsing & Import Resolution
```rust
impl PythonReachabilityAnalyzer {
    /// Parse Python files and resolve imports
    pub fn parse_project(&mut self, root: &Path) -> Result<()> {
        // 1. Find all .py files
        // 2. Parse into AST
        // 3. Extract imports
        // 4. Resolve import paths
        // 5. Build module graph
    }

    /// Resolve Python imports (supports relative and absolute)
    fn resolve_import(&self, import: &str, from: &Path) -> Result<PathBuf> {
        // Handle:
        // - from package.module import func
        // - from . import sibling
        // - from .. import parent
        // - import package.module as alias
    }
}
```

#### Week 2: Call Graph Generation

**Entrypoints:**
- `if __name__ == "__main__":`
- Flask routes: `@app.route("/path")`
- Django views: functions in urls.py
- FastAPI routes: `@app.get("/path")`
- Click CLI commands: `@click.command()`
- Pytest tests: `def test_*()`

**Call Extraction:**
```rust
fn extract_calls(node: &ast::Stmt) -> Vec<FunctionCall> {
    match node {
        ast::Stmt::Expr(expr) => {
            match &expr.value {
                ast::Expr::Call(call) => {
                    // Direct call: func()
                    extract_function_name(call.func)
                }
                _ => vec![]
            }
        }
        ast::Stmt::Assign(assign) => {
            // Method call: obj.method()
            // Attribute access: module.func
        }
        _ => vec![]
    }
}
```

**Conservative Rules:**
- `getattr()` calls → mark all class methods as potentially reachable
- `exec()` / `eval()` → mark entire module as reachable
- Decorators → analyze decorator functions
- Generators / async → track async call chains

#### Week 3: Vulnerability Mapping & Integration

**Map CVEs to Python functions:**
```python
# Example: Django vulnerability
CVE-2024-1234 in django@3.2.0
Vulnerable function: django.db.models.query.QuerySet.raw()

Call chain:
  app.py:main()
    → views/api.py:user_detail()
      → models.py:User.objects.raw()  ← VULNERABLE!
```

**Data Sources:**
- Python Security Advisories (PyPI)
- OSV database
- Manual function mappings for popular packages

### Python Reachability Output Example

```bash
$ bazbom scan . --python

🐍 Python Reachability Analysis

📦 Found 23 vulnerabilities (12 reachable, 11 unreachable)

🔴 REACHABLE VULNERABILITIES
  CVE-2024-1234 in django@3.2.0

  Vulnerable function: django.db.models.query.QuerySet.raw()

  Call chain:
    app.py:main() [line 45]
      → views/api.py:user_detail() [line 102]
        → models.py:User.objects.raw()  ← VULNERABLE!

  Risk: SQL Injection
  Fix: Upgrade to django@3.2.18

⚠️  DYNAMIC CODE DETECTED
  File: utils/helpers.py:42
  Code: exec(user_input)

  Warning: Dynamic code execution makes static analysis
  unreliable. All dependencies are conservatively marked
  as reachable.

  Recommendation: Refactor to avoid exec()/eval()
```

---

## 🐹 Part 2: Go Reachability Analysis

### Technical Approach

**Parser: Go AST (built-in in Go, or tree-sitter)**

Go's static nature makes this more tractable than Python:
```rust
use tree_sitter::{Parser, Language};

extern "C" { fn tree_sitter_go() -> Language; }

pub struct GoReachabilityAnalyzer {
    parser: Parser,
    call_graph: CallGraph,
    package_map: HashMap<String, PathBuf>,
}
```

**Alternatively, use Go itself:**
- Call `go build -work -x` to get compiler output
- Parse AST using `go/parser` and `go/ast` (via cgo or subprocess)
- Leverage Go's excellent tooling

### Challenges

1. **Interfaces**
   - Interface satisfaction is implicit
   - Method calls through interfaces hard to resolve statically

2. **Goroutines**
   - `go func()` - concurrent execution
   - Channel communication

3. **Reflection**
   - `reflect.Value.Call()`
   - `reflect.Value.MethodByName()`

4. **Build Tags**
   - Platform-specific code: `// +build linux`
   - Need to respect build context

### Solution Strategy

**Static Analysis + Go Tooling:**
- Use `go list -json` to understand package structure
- Parse AST with `go/parser`
- Use `go/types` for type information
- Handle interfaces conservatively

**Implementation Plan:**

#### Week 1: AST Parsing & Package Resolution

```rust
impl GoReachabilityAnalyzer {
    /// Parse Go project using go/parser
    pub fn parse_project(&mut self, root: &Path) -> Result<()> {
        // 1. Run: go list -json ./...
        // 2. Parse each package
        // 3. Extract imports
        // 4. Build package dependency graph
    }

    /// Use go/types for type information
    fn analyze_types(&mut self, pkg: &Package) -> Result<TypeInfo> {
        // Leverage Go's type system
        // Handle interfaces, structs, methods
    }
}
```

#### Week 2: Call Graph with Interface Resolution

**Entrypoints:**
- `func main()` in main package
- HTTP handlers: `http.HandleFunc`, `gin.GET`, etc.
- gRPC services: generated service implementations
- CLI commands: `cobra.Command.Run`

**Interface Handling:**
```rust
// When we see:
//   var i MyInterface = &MyStruct{}
//   i.Method()

// Resolution:
// 1. Find all types that implement MyInterface
// 2. Mark all their Method() implementations as potentially reachable
// 3. Conservative: assume any implementation could be called
```

**Goroutine Tracking:**
```rust
// Track goroutine launches
// go someFunc() → mark someFunc as reachable
// Channel sends/receives → track data flow
```

#### Week 3: Vulnerability Mapping & Integration

**Go vulnerability database:**
- Use govulncheck database (official Go vulnerability DB)
- Map vulnerabilities to specific functions/methods
- Generate call chains

### Go Reachability Output Example

```bash
$ bazbom scan . --go

🐹 Go Reachability Analysis

📦 Found 15 vulnerabilities (8 reachable, 7 unreachable)

🔴 REACHABLE VULNERABILITY
  GO-2024-1234 in github.com/gin-gonic/gin@v1.7.0

  Vulnerable function: gin.Context.BindJSON()

  Call chain:
    main.go:main() [line 20]
      → routes/api.go:HandleUser() [line 45]
        → gin.Context.BindJSON()  ← VULNERABLE!

  Risk: JSON Injection
  Fix: Upgrade to gin@v1.9.0

⚠️  REFLECTION DETECTED
  File: internal/registry/registry.go:102
  Code: reflect.Value.MethodByName("Process").Call(...)

  Warning: Reflection makes exact reachability uncertain.
  Conservatively marking reflected package as reachable.
```

---

## 📋 Combined Timeline (3 Weeks)

### Week 1: Parsing & Import/Package Resolution
- Days 1-3: Python AST parsing + import resolution
- Days 4-5: Go AST parsing + package resolution

### Week 2: Call Graph Generation
- Days 1-2: Python call graph with dynamic analysis
- Days 3-4: Go call graph with interface resolution
- Day 5: Handle goroutines and reflection

### Week 3: Integration & Testing
- Days 1-2: Vulnerability mapping for both languages
- Days 3-4: Integration with SCA pipeline
- Day 5: Testing, documentation, polish

---

## 🎯 Success Criteria

### Python
- ✅ Parse Python 3.8+ projects
- ✅ Resolve absolute and relative imports
- ✅ Identify common framework entrypoints (Django, Flask, FastAPI)
- ✅ Handle decorators and class methods
- ✅ Warn about dynamic code (exec/eval/getattr)
- ✅ Accuracy: >80% (Python's dynamism limits precision)

### Go
- ✅ Parse Go 1.18+ projects (with generics)
- ✅ Resolve package imports and vendoring
- ✅ Handle interfaces conservatively
- ✅ Track goroutines
- ✅ Integrate with govulncheck database
- ✅ Accuracy: >95% (Go is more static)

### Performance
- ✅ Python: < 15 seconds for 100k LOC
- ✅ Go: < 10 seconds for 100k LOC

---

## 🚧 Known Limitations

### Python
- ❌ Dynamic `getattr()` / `setattr()` with variables
- ❌ `exec()` and `eval()` (conservatively mark as reachable)
- ❌ Metaclass magic and descriptor protocol
- ❌ C extensions (can't analyze native code)

### Go
- ❌ CGo calls (can't analyze C code)
- ❌ Reflection with string method names
- ❌ Unsafe pointer manipulation
- ❌ Assembly code

---

## 📦 Deliverables

### Code
- [ ] `crates/bazbom-python-reachability/` - New crate
- [ ] `crates/bazbom-go-reachability/` - New crate
- [ ] Integration with polyglot pipeline
- [ ] Updated CLI and SARIF output

### Documentation
- [ ] `docs/polyglot/python-reachability.md`
- [ ] `docs/polyglot/go-reachability.md`
- [ ] Example projects for Django, Flask, Gin, Echo

### Testing
- [ ] 30+ unit tests per language
- [ ] Integration tests with real projects
- [ ] Performance benchmarks

---

## 🔬 Real-World Test Cases

### Python Projects
- Django web app with PostgreSQL
- Flask API with SQLAlchemy
- FastAPI microservice
- Click CLI tool
- Jupyter notebook analysis

### Go Projects
- Gin web server
- gRPC service
- Cobra CLI application
- Kubernetes operator
- Docker/containerd (if time permits)

---

**Status:** PLANNED
**Start Date:** February 2026
**Timeline:** 3 weeks
**Next:** v6.5.0 - Rust + Ruby + PHP + Complete Parity

---

*Bringing Python and Go to the same world-class standard as JVM and JavaScript.*
