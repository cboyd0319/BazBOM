# Quick Start: BazBOM on Your Monorepo

## 🚀 Get Scanning in 5 Minutes

Based on your diagnostics, here's the fastest path to your first scan.

---

## Step 1: First Test Scan (2-3 minutes)

```bash
cd /path/to/your/monorepo

# Start small - test with 10 packages
RUST_LOG=debug /path/to/BazBOM/target/release/bazbom full --limit 10
```

**What this does:**
- Scans only 10 polyglot packages (npm/Python/Go)
- Loads all 2,067 Maven dependencies from `maven_install.json`
- Creates SBOM in `./sbom/`
- Runs vulnerability analysis with EPSS + KEV
- Takes ~2-3 minutes

**Expected output:**
```
💪 Running FULL scan with ALL features enabled...

   ℹ️  Limiting scan to 10 packages/targets

[bazbom] detected JVM build system: Bazel
[bazbom] scanning for polyglot ecosystems...
📦 Detected polyglot ecosystems:
  📦 Node.js/npm - X packages, Y vulnerabilities
  🐍 Python - X packages, Y vulnerabilities
  🐹 Go - X packages, Y vulnerabilities
[bazbom] wrote JVM SPDX SBOM to "./sbom/sbom.spdx.json"
[bazbom] wrote polyglot SBOM to "./sbom/polyglot-sbom.json"
[bazbom] parsing SBOM from "./sbom/sbom.spdx.json"
[bazbom] extracted XXXX components from SBOM
[bazbom] syncing advisory database...
[bazbom] loaded XXXXX EPSS scores and XXXX KEV entries
```

---

## Step 2: Check Results

```bash
# Verify SBOM was created
ls -lh sbom/

# Count Maven dependencies loaded
cat sbom/sbom.spdx.json | jq '.packages | length'
# Should show ~2067 packages

# Check polyglot ecosystems detected
cat sbom/polyglot-sbom.json | jq '.ecosystems[].ecosystem'
# Should show: npm, Python, Go

# Review vulnerability findings
cat findings/sca.sarif | jq '.runs[0].results | length'
# Shows number of vulnerabilities found
```

---

## Step 3: Scale Up (10-15 minutes)

```bash
# Increase to 50 packages
RUST_LOG=info bazbom full --limit 50 --benchmark
```

**New flags:**
- `--benchmark` - Shows timing breakdown
- `RUST_LOG=info` - Less verbose than debug

**Expected output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Performance Benchmark Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Phase                     Time          Percentage
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SBOM Generation          1m 12s         12.0%
Polyglot Scanning        7m 34s         75.0%
Vulnerability Scan       1m 18s         13.0%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total Scan Time         10m 04s        100.0%
```

---

## Step 4: Full Scan (1-3 hours first time)

```bash
# Remove limit, enable all features
bazbom full --incremental --benchmark

# Or with caching for faster subsequent runs
bazbom full --benchmark
```

**First run:** 1-3 hours (scanning 230 npm + 80 Python + 10 Go projects + Bazel deps)

**Subsequent runs:** 15-45 minutes (with `.bazbom/cache/`)

---

## 🎯 Target-Specific Scans (Fast!)

If you only want to scan specific parts:

```bash
# Scan specific service only (your largest component)
bazbom scan --target //service-a/...

# Scan SDK package
bazbom scan --target //sdk/...

# Scan infrastructure
bazbom scan --target //infrastructure/...

# Scan changed files only (for PRs)
bazbom full --diff --base main
```

---

## 📊 Understanding Your Results

### Maven Dependencies (~2,067 packages)

From `_bazel/java/maven_install.json`:
- AWS SDK (100+ packages)
- Database drivers (Redshift, MariaDB, etc.)
- Common libraries (Jackson, Guava, etc.)
- Testing frameworks

**Check what was loaded:**
```bash
cat sbom/sbom.spdx.json | jq '.packages[].name' | head -20
```

### npm Projects (230 packages)

Tools and internal apps:
- Internal development tools
- Web applications
- Connector SDK frontend
- Internal dashboards
- Algolia/Zendesk integrations

**Check what was scanned:**
```bash
cat sbom/polyglot-sbom.json | jq '.ecosystems[] | select(.ecosystem == "Node.js/npm")'
```

### Python Projects (80 packages)

Backend tools and automation:
- Connector generators
- Database tools
- Internal automation
- Alert systems

**Check what was scanned:**
```bash
cat sbom/polyglot-sbom.json | jq '.ecosystems[] | select(.ecosystem == "Python")'
```

### Go Modules (10 packages)

Infrastructure components:
- Data lake servers
- Infrastructure operator
- SRE tools
- K8s health checks

**Check what was scanned:**
```bash
cat sbom/polyglot-sbom.json | jq '.ecosystems[] | select(.ecosystem == "Go")'
```

---

## ⚡ Performance Tips

### Use Caching

After first scan, cache is created in `.bazbom/cache/`:

```bash
# Check cache size
du -sh .bazbom/cache/

# Clear if needed
rm -rf .bazbom/cache/*
```

**Cache contains:**
- EPSS scores (24h lifetime)
- KEV catalog (24h lifetime)
- Scan results (keyed by git commit)
- Polyglot scan data

### Use Incremental Mode

For PRs and development:

```bash
# Only scan files changed since main
bazbom full --incremental --base main

# Or use diff mode
bazbom full --diff
```

**Speedup:** 70-95% faster for typical PRs

### Parallelize

Use all CPU cores:

```bash
# Auto-detect
export RAYON_NUM_THREADS=$(sysctl -n hw.ncpu)  # macOS
# or
export RAYON_NUM_THREADS=$(nproc)  # Linux

bazbom full
```

---

## 🐛 Troubleshooting

### Problem: Scan takes too long

**Solution 1:** Use limit
```bash
bazbom full --limit 20
```

**Solution 2:** Scan specific targets
```bash
bazbom scan --target //your/package/...
```

**Solution 3:** Use incremental
```bash
bazbom full --incremental
```

### Problem: 0 dependencies detected

**Status:** ✅ FIXED in latest build

**Verify fix:**
```bash
cat sbom/sbom.spdx.json | jq '.packages | length'
# Should show 2000+ packages
```

If still 0, check:
```bash
# Verify maven_install.json exists
ls -lh _bazel/java/maven_install.json

# Check Bazel version
bazel version
```

### Problem: Database sync fails

**Solutions:**
```bash
# Check network
curl -I https://api.first.org/data/v1/epss

# Retry with clean cache
rm -rf .bazbom/advisories
RUST_LOG=debug bazbom full --limit 5
```

### Problem: Out of memory

**Solutions:**
```bash
# Reduce limit
bazbom full --limit 10

# Clear cache
rm -rf .bazbom/cache/*

# Scan in batches (different targets)
bazbom scan --target //service-a/...
bazbom scan --target //sdk/...
```

---

## 📁 Output Structure

After a scan, you'll have:

```
.
├── sbom/
│   ├── sbom.spdx.json          # Main SBOM (Maven deps)
│   ├── polyglot-sbom.json      # npm, Python, Go
│   └── cyclonedx.json          # Alternative format
├── findings/
│   ├── sca.sarif               # Vulnerability findings
│   └── semgrep.sarif           # SAST findings (if enabled)
├── .bazbom/
│   ├── cache/                  # Cached data
│   ├── advisories/             # EPSS, KEV databases
│   └── benchmarks/             # Performance data
└── bazbom-diagnostics/         # From diagnostics script
```

---

## 🎓 Next Steps

### Immediate
1. ✅ Run first test: `bazbom full --limit 10`
2. ✅ Verify results: Check `sbom/` and `findings/`
3. ✅ Review benchmark: See timing breakdown

### This Week
1. Run full scan: `bazbom full --benchmark`
2. Establish baseline: Document current vuln count
3. Triage findings: Focus on Critical/High

### Next 2 Weeks
1. **CI/CD Integration:** Add to GitHub Actions
2. **Incremental scans:** Test on PRs
3. **Monitor performance:** Track scan times

---

## 💡 Pro Tips

### 1. Create bazbom.toml

Save this in your repo root:

```toml
[scan]
incremental = true
benchmark = true

[bazel]
universe = "//..."

[polyglot]
npm_enabled = true
python_enabled = true
go_enabled = true
```

Then just run:
```bash
bazbom full  # Uses config automatically
```

### 2. Alias for convenience

Add to your `.zshrc` or `.bashrc`:

```bash
alias bscan='RUST_LOG=info bazbom full --benchmark'
alias btest='RUST_LOG=debug bazbom full --limit 10'
alias bdiff='bazbom full --diff --base main'
```

Then:
```bash
btest     # Quick test
bscan     # Full scan
bdiff     # PR scan
```

### 3. Watch progress

```bash
# In one terminal
bazbom full --benchmark

# In another terminal
tail -f .bazbom/logs/latest.log  # If logging to file
```

---

## ✅ Quick Checklist

Before your first scan:

- [ ] BazBOM built: `/path/to/BazBOM/target/release/bazbom`
- [ ] In monorepo root: `cd /path/to/your/monorepo`
- [ ] Bazel works: `bazel version` shows 8.3.1
- [ ] `.bazbom/` in `.gitignore` (already done ✓)

For testing:

- [ ] Run test scan: `bazbom full --limit 10`
- [ ] Check SBOM: `cat sbom/sbom.spdx.json | jq '.packages | length'`
- [ ] Check findings: `cat findings/sca.sarif | jq '.runs[0].results | length'`
- [ ] Review debug logs for errors

For production:

- [ ] Run full scan: `bazbom full --benchmark`
- [ ] Document scan time
- [ ] Review all findings
- [ ] Create bazbom.toml
- [ ] Set up CI/CD

---

## 🎉 You're Ready!

Your command to start:

```bash
cd /path/to/your/monorepo
RUST_LOG=debug /path/to/BazBOM/target/release/bazbom full --limit 10 --benchmark
```

**Expected time:** 2-3 minutes
**Expected output:** SBOM + vulnerability findings
**Next step:** Review results, then scale to `--limit 50`

---

**Questions?** See `MONOREPO_TUNING_GUIDE.md` for comprehensive details.

**Issues?** Check debug logs: `RUST_LOG=debug` shows everything.

**Success?** Share your benchmark results! We'd love to see BazBOM running on an 8.3GB monorepo!
