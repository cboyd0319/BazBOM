---
name: reachability-validator
description: Validates reachability analysis results for accuracy, checks for common issues (missing entrypoints, dynamic code false negatives), and provides recommendations. Activates when user asks about reachability correctness, call graph validation, or accuracy concerns.
---

# Reachability Validator Skill

Automatically validates reachability analysis results and provides accuracy assessments.

## When to Use

Activate this skill when you hear:
- "Is this reachability analysis correct?"
- "Validate call graph"
- "Check reachability accuracy"
- "Why is this marked unreachable?"
- "Reachability false positive/negative"
- "Verify entrypoint detection"

## Validation Checks

### 1. Entrypoint Detection
```bash
# List detected entrypoints
RUST_LOG=bazbom_reachability::entrypoints=debug bazbom scan -r . 2>&1 | grep "entrypoint"

# Validation criteria:
# ✅ Main/entry functions detected
# ✅ Framework routes detected (Express, Flask, Django, etc.)
# ✅ Test functions included (or excluded based on config)
# ✅ Expected entrypoint count reasonable
```

### 2. Call Graph Completeness
```bash
# Export call graph
bazbom scan -r --export-graph callgraph.dot .

# Validate:
# ✅ Graph contains expected functions
# ✅ No isolated subgraphs (except intentional)
# ✅ Framework-specific patterns present
# ✅ Cross-module calls tracked
```

### 3. Reachability Results
```bash
# Compare with/without reachability
bazbom scan . -o /tmp/without
bazbom scan -r . -o /tmp/with

VULN_WITHOUT=$(jq '.vulnerabilities | length' /tmp/without/sca_findings.json)
VULN_WITH=$(jq '.vulnerabilities | length' /tmp/with/sca_findings.json)
REDUCTION=$(echo "scale=1; 100 - ($VULN_WITH * 100 / $VULN_WITHOUT)" | bc)

echo "Reduction: ${REDUCTION}%"

# Expected ranges by language:
# - Rust: 70-90% reduction (highly static)
# - Go: 60-85% reduction
# - JavaScript/TypeScript: 60-80% reduction
# - Python: 55-75% reduction
# - Ruby: 50-70% reduction
# - PHP: 45-65% reduction
```

### 4. Dynamic Code Handling
```bash
# Check for conservative markings
RUST_LOG=bazbom_reachability::dynamic=debug bazbom scan -r . 2>&1 | grep "eval\|reflection\|dynamic"

# Should see warnings for:
# - eval() calls
# - Reflection usage
# - Dynamic imports
# - Metaprogramming patterns
```

## Validation Report Format

```
Reachability Analysis Validation Report
========================================

Language: JavaScript (Express.js)
Project: api-server
Scan Date: 2024-11-18

Entrypoint Detection:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ PASS - Found 47 entrypoints
   ├─ Express routes: 32
   ├─ Middleware: 8
   ├─ Jest tests: 7 (included)
   └─ Expected range: 30-60 ✓

Call Graph Analysis:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ PASS - Call graph complete
   ├─ Total functions: 1,234
   ├─ Reachable: 487 (39%)
   ├─ Unreachable: 747 (61%)
   └─ Isolated subgraphs: 0 ✓

Reachability Results:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ PASS - Noise reduction within expected range
   ├─ Vulnerabilities without reachability: 237
   ├─ Vulnerabilities with reachability: 56
   ├─ Reduction: 76% ✓ (expected 60-80%)
   └─ P0/P1 reachable: 12 (requires immediate attention)

Dynamic Code Handling:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⚠️  WARNING - Dynamic patterns detected
   ├─ eval() calls: 3 instances (marked all reachable)
   ├─ require() with variables: 5 instances (best-effort)
   ├─ Dynamic property access: 12 instances
   └─ Impact: May cause false positives (conservative)

Framework Integration:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ PASS - Express.js patterns detected
   ├─ app.get/post routes: ✓
   ├─ Router.use middleware: ✓
   ├─ Error handlers: ✓
   └─ Async handlers: ✓

Overall Assessment:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ VALIDATED - Reachability analysis is accurate

Confidence: HIGH
  - Entrypoint detection: 95%
  - Call graph completeness: 90%
  - Result accuracy: 85%
  - False negative risk: LOW
  - False positive risk: MEDIUM (due to eval() usage)

Recommendations:
  1. Review eval() usage at api/utils.js:45 (security risk)
  2. Consider excluding test files if not needed
  3. Export call graph for manual review of edge cases
```

## Common Issues Detected

### False Positives (Marked Reachable but Not)

**Indicators:**
- Reduction < 50% (should be 60-90% for most languages)
- Many vulnerabilities in clearly dead code
- Test code being treated as production

**Diagnosis:**
```bash
# Check if tests included
jq '.reachability.entrypoints[] | select(.framework == "jest")' /tmp/results/callgraph.json

# Exclude tests and rescan
bazbom scan -r --exclude-paths "**/test/**,**/*.test.js" .

# Check dynamic code conservatism
RUST_LOG=bazbom_reachability::dynamic=debug bazbom scan -r . 2>&1 | grep -A 5 "conservative"
```

**Solution:**
- Exclude test directories
- Review dynamic code patterns
- Adjust analysis depth

### False Negatives (Marked Unreachable but Is)

**Indicators:**
- Known reachable code marked as unreachable
- Framework routes not detected
- Missing expected entrypoints

**Diagnosis:**
```bash
# Check entrypoint count
RUST_LOG=bazbom_reachability::entrypoints=info bazbom scan -r .

# Verify framework detection
RUST_LOG=bazbom_js_reachability::frameworks=debug bazbom scan -r .  # For JS

# Manual entrypoint specification
bazbom scan -r --entrypoints src/index.js:main,src/routes.js:* .
```

**Solution:**
- Add missing framework patterns
- Manually specify entrypoints
- Check for new framework version

### Unrealistic Reduction

**Indicators:**
- Reduction > 95% (too aggressive)
- Critical code marked unreachable
- Most dependencies marked unreachable

**Diagnosis:**
```bash
# List unreachable packages
jq '.packages[] | select(.reachable == false) | .name' /tmp/results/sbom.spdx.json | head -20

# Check if expected packages are reachable
jq '.packages[] | select(.name == "express") | .reachable' /tmp/results/sbom.spdx.json

# Verify entrypoints exist
jq '.reachability.entrypoints | length' /tmp/results/callgraph.json
```

**Solution:**
- Verify entrypoints detected correctly
- Check for graph construction errors
- Enable deeper analysis

## Language-Specific Validation

### JavaScript/TypeScript
**Expected reduction:** 60-80%
**Key checks:**
- ✅ Express/Fastify/Koa routes detected
- ✅ Next.js API routes detected
- ✅ React components considered if SSR
- ⚠️ eval(), new Function() marked conservative

### Python
**Expected reduction:** 55-75%
**Key checks:**
- ✅ Flask/Django routes detected
- ✅ FastAPI endpoints detected
- ✅ Click commands detected
- ⚠️ exec(), eval(), getattr() marked conservative

### Go
**Expected reduction:** 60-85%
**Key checks:**
- ✅ func main() detected
- ✅ HTTP handlers detected
- ✅ Test functions handled correctly
- ⚠️ reflect package usage marked conservative

### Rust
**Expected reduction:** 70-90%
**Key checks:**
- ✅ fn main() detected
- ✅ #[test] functions handled
- ✅ Trait implementations tracked
- ✅ No dynamic code concerns

### Ruby
**Expected reduction:** 50-70%
**Key checks:**
- ✅ Rails controllers/routes detected
- ✅ Sinatra routes detected
- ✅ RSpec tests handled
- ⚠️ method_missing, send() marked conservative

### PHP
**Expected reduction:** 45-65%
**Key checks:**
- ✅ Laravel routes detected
- ✅ Symfony routes detected
- ✅ WordPress hooks detected
- ⚠️ Variable functions, eval() marked conservative

## Quick Validation Commands

```bash
# Basic validation
bazbom scan -r . -o /tmp/results
jq '.reachability | {
  entrypoints: (.entrypoints | length),
  reachable_packages: (.packages[] | select(.reachable == true) | .name) | length,
  reduction_pct: (.reduction_percentage)
}' /tmp/results/sbom.spdx.json

# Validate specific vulnerability
jq '.vulnerabilities[] | select(.id == "CVE-2024-1234") | {
  id,
  reachable,
  call_path
}' /tmp/results/sca_findings.json

# Export and visualize
bazbom scan -r --export-graph /tmp/callgraph.dot .
dot -Tpng /tmp/callgraph.dot -o /tmp/callgraph.png
open /tmp/callgraph.png  # Visual inspection
```

## Success Criteria

Reachability analysis passes validation when:
- ✅ Entrypoint count reasonable for project size
- ✅ Reduction within expected language range (45-90%)
- ✅ Call graph contains expected functions
- ✅ Framework patterns detected correctly
- ✅ Dynamic code handled conservatively
- ✅ No critical false negatives (manual spot check)
- ✅ False positives acceptable (<15% over-reporting)

Remember: **Conservative analysis is preferred** - better to over-report (false positive) than miss a real vulnerability (false negative).
