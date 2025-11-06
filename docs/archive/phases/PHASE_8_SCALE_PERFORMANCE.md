# Phase 8: Scale & Performance

**Status:** Planned
**Priority:**  P0 - Critical Path
**Timeline:** Months 5-7 (10 weeks)
**Team Size:** 1-2 developers
**Dependencies:** Phase 0-3 (Complete), Phase 4 (helps with testing)

---

## Executive Summary

**Goal:** Handle Google/Meta-scale monorepos (50K+ targets) without requiring datacenter hardware.

**Current State:** BazBOM tested on 5K target monorepos. EndorLabs proven on 50K+ targets.

**Target State:**
- Incremental analysis (10x faster PR scans)
- Distributed analysis (split work across multiple machines)
- 50K target monorepo scans in <10 minutes (incremental)
- Memory usage <4GB for typical workloads

**Success Metrics:**
-  50K target monorepo analyzed in <10 minutes (incremental)
-  Full scan completes in <30 minutes (vs. hours)
-  Memory usage <4GB (vs. EndorLabs' 64GB requirement)
-  Zero performance regressions vs. Phase 3 baseline

**Competitive Benchmark:** Match EndorLabs' scale while using 16x less memory.

---

## Current Performance Baseline

### Measured Performance (Phase 3)

| Metric | Small (100 deps) | Medium (1K deps) | Large (10K deps) | Massive (50K targets) |
|--------|------------------|------------------|------------------|-----------------------|
| **SBOM Generation** | <5s | <15s | <60s | Untested |
| **Vulnerability Scan** | <2s | <10s | <30s | Untested |
| **Reachability Analysis** | <5s | <20s | <120s | Untested |
| **Full Scan (Total)** | ~10s | ~45s | ~3min | Unknown |
| **Memory Usage** | <100MB | <500MB | <1GB | Unknown |

### Bottlenecks Identified

1. **Serial SBOM Generation** - Processes targets one at a time
2. **Redundant Advisory Queries** - Same CVE looked up multiple times
3. **Reachability Re-computation** - No caching of call graphs
4. **File I/O** - Reading/writing SBOM files repeatedly
5. **No Incremental Mode** - Always scans entire workspace

---

## 8.1 Incremental Analysis Engine

### Problem Statement

**Scenario:** Developer changes one file in 50K target monorepo

**Today:**
- Full scan: Analyze all 50,000 targets (~8 hours)
- Wastes time: 49,999 targets unchanged

**Tomorrow:**
- Incremental scan: Analyze only affected targets (~10 minutes)
- 48x faster (10 min vs 8 hours)

### Implementation Strategy

#### 8.1.1 Git-Based Change Detection

**Approach:** Use `git diff` to find changed files, then query Bazel for affected targets

**Algorithm:**
```
1. Get changed files: `git diff --name-only HEAD~1`
2. Find affected targets: `bazel query 'rdeps(//..., set({changed_files}))'`
3. Scan only affected targets
4. Merge with cached results for unchanged targets
```

**Implementation:**

```rust
// crates/bazbom/src/incremental/mod.rs
use std::process::Command;

pub struct IncrementalAnalyzer {
    workspace_root: PathBuf,
    cache_dir: PathBuf,
}

impl IncrementalAnalyzer {
    pub fn find_affected_targets(&self, base_ref: &str) -> Result<Vec<String>> {
        // Step 1: Get changed files
        let changed_files = self.get_changed_files(base_ref)?;

        if changed_files.is_empty() {
            println!("No files changed. Using cached results.");
            return Ok(vec![]);
        }

        println!("Found {} changed files", changed_files.len());

        // Step 2: Build Bazel query
        let file_set = changed_files
            .iter()
            .map(|f| format!("//{}", f.trim_start_matches("/")))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!("rdeps(//..., set({}))", file_set);

        // Step 3: Execute Bazel query
        let output = Command::new("bazel")
            .args(&["query", &query, "--output=label"])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Bazel query failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let targets: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        println!("Found {} affected targets (out of ~50K total)", targets.len());

        Ok(targets)
    }

    fn get_changed_files(&self, base_ref: &str) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["diff", "--name-only", base_ref])
            .current_dir(&self.workspace_root)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Git diff failed"));
        }

        Ok(String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect())
    }

    pub fn scan_incremental(&self, affected_targets: Vec<String>) -> Result<ScanResult> {
        let mut results = ScanResult::new();

        // Scan affected targets (new)
        for target in affected_targets {
            let result = scan_target(&target)?;
            results.merge(result);
        }

        // Load cached results for unaffected targets
        let cached = self.load_cached_results()?;
        results.merge(cached);

        // Save updated cache
        self.save_cache(&results)?;

        Ok(results)
    }
}
```

**CLI Integration:**

```bash
# Incremental scan (PR mode)
bazbom scan --incremental --base=main

# Incremental with explicit files
bazbom scan --affected-by-files src/java/lib/foo.java

# Full scan (force)
bazbom scan --full
```

#### 8.1.2 Dependency Graph Invalidation

**Problem:** If A changes and B depends on A, must rescan B too

**Solution:** Bazel's `rdeps()` handles this automatically

**Optimization:** Cache dependency graph itself

```rust
// crates/bazbom/src/incremental/graph_cache.rs
use petgraph::Graph;

pub struct DependencyGraphCache {
    graph: Graph<String, ()>,  // Nodes = targets, edges = dependencies
    last_updated: SystemTime,
}

impl DependencyGraphCache {
    pub fn build_graph(&mut self) -> Result<()> {
        // Query full dependency graph (slow, but cache it)
        let output = Command::new("bazel")
            .args(&["query", "deps(//...)", "--output=graph"])
            .output()?;

        // Parse DOT format into petgraph
        self.graph = parse_dot(&String::from_utf8(output.stdout)?)?;
        self.last_updated = SystemTime::now();

        Ok(())
    }

    pub fn find_transitive_dependents(&self, changed_target: &str) -> Vec<String> {
        // Use graph traversal instead of repeated Bazel queries
        let node = self.graph.node_indices().find(|&n| self.graph[n] == changed_target);

        if let Some(node) = node {
            petgraph::algo::dfs::Dfs::new(&self.graph, node)
                .iter(&self.graph)
                .map(|n| self.graph[n].clone())
                .collect()
        } else {
            vec![]
        }
    }
}
```

**Cache Invalidation:** Rebuild graph daily or on `MODULE.bazel` changes

#### 8.1.3 Result Caching

**Cache Structure:**

```
.bazbom/cache/
├── sbom/
│   ├── {target_hash}.spdx.json        # Per-target SBOMs
│   ├── {target_hash}.cyclonedx.json
│   └── workspace.spdx.json             # Merged workspace SBOM
├── findings/
│   ├── {target_hash}.sarif            # Per-target findings
│   └── workspace.sarif                 # Merged findings
├── reachability/
│   └── {classpath_hash}.json          # Reachability results
└── metadata.json                       # Cache index
```

**Cache Key:** BLAKE3 hash of:
- Target label
- Dependency versions (from resolved graph)
- BazBOM version

**Invalidation Triggers:**
- File changes affecting target
- Dependency version changes
- BazBOM CLI upgrade

**Implementation:**

```rust
// crates/bazbom/src/incremental/cache.rs
use blake3::Hasher;

pub struct ResultCache {
    cache_dir: PathBuf,
}

impl ResultCache {
    pub fn get_cache_key(&self, target: &str, deps: &[Dependency]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(target.as_bytes());
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());  // BazBOM version

        for dep in deps {
            hasher.update(dep.purl.as_bytes());
        }

        hasher.finalize().to_hex().to_string()
    }

    pub fn load_cached_sbom(&self, cache_key: &str) -> Option<Sbom> {
        let path = self.cache_dir.join("sbom").join(format!("{}.spdx.json", cache_key));
        if path.exists() {
            serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
        } else {
            None
        }
    }

    pub fn save_cached_sbom(&self, cache_key: &str, sbom: &Sbom) -> Result<()> {
        let path = self.cache_dir.join("sbom").join(format!("{}.spdx.json", cache_key));
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, serde_json::to_string_pretty(sbom)?)?;
        Ok(())
    }
}
```

---

## 8.2 Parallel Processing

### Current: Serial Execution

**Problem:** Targets processed one at a time

```rust
for target in targets {
    scan_target(target);  // Slow!
}
```

### Solution: Parallel with Rayon

```rust
use rayon::prelude::*;

targets.par_iter()
    .map(|target| scan_target(target))
    .collect::<Vec<_>>();
```

**Speedup:** 4-8x on modern CPUs (4-16 cores)

**Implementation:**

```rust
// crates/bazbom/src/scan_orchestrator.rs
use rayon::prelude::*;

pub struct ScanOrchestrator;

impl ScanOrchestrator {
    pub fn scan_parallel(&self, targets: Vec<String>) -> Result<Vec<ScanResult>> {
        let results: Vec<_> = targets
            .par_iter()  // Parallel iterator
            .map(|target| {
                println!("Scanning target: {}", target);
                self.scan_single_target(target)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(results)
    }
}
```

**Thread Pool Tuning:**

```rust
// Limit parallelism to avoid OOM
rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get().min(16))  // Cap at 16 threads
    .build_global()?;
```

---

## 8.3 Memory Optimization

### Current: Load Entire SBOM into Memory

**Problem:** 50K targets × 10KB SBOM = 500MB just for SBOMs

**Solution:** Stream processing

#### 8.3.1 Streaming SBOM Generation

```rust
// Instead of:
let sboms: Vec<Sbom> = targets.iter().map(|t| generate_sbom(t)).collect();  // OOM!

// Use streaming:
use std::io::BufWriter;

let file = File::create("workspace.spdx.json")?;
let mut writer = BufWriter::new(file);

for target in targets {
    let sbom = generate_sbom(target)?;
    serde_json::to_writer(&mut writer, &sbom)?;
    writer.write_all(b"\n")?;  // Newline-delimited JSON
}
```

#### 8.3.2 Advisory Cache in SQLite

**Problem:** Loading 1M+ CVEs into HashMap = high memory

**Solution:** SQLite with indexed queries

```rust
// crates/bazbom-advisories/src/db.rs
use rusqlite::{Connection, params};

pub struct AdvisoryDb {
    conn: Connection,
}

impl AdvisoryDb {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Create schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vulnerabilities (
                id TEXT PRIMARY KEY,
                purl TEXT NOT NULL,
                severity TEXT,
                cvss_score REAL,
                cisa_kev BOOLEAN,
                epss REAL,
                fixed_version TEXT,
                description TEXT
            )",
            [],
        )?;

        // Index for fast lookups
        conn.execute("CREATE INDEX IF NOT EXISTS idx_purl ON vulnerabilities(purl)", [])?;

        Ok(Self { conn })
    }

    pub fn query_vulns(&self, purl: &str) -> Result<Vec<Vulnerability>> {
        let mut stmt = self.conn.prepare("SELECT * FROM vulnerabilities WHERE purl = ?1")?;
        let vulns = stmt.query_map(params![purl], |row| {
            Ok(Vulnerability {
                id: row.get(0)?,
                purl: row.get(1)?,
                severity: row.get(2)?,
                cvss_score: row.get(3)?,
                cisa_kev: row.get(4)?,
                epss: row.get(5)?,
                fixed_version: row.get(6)?,
                description: row.get(7)?,
            })
        })?;

        vulns.collect()
    }
}
```

**Memory Savings:** HashMap (500MB) → SQLite (<50MB resident)

---

## 8.4 Distributed Analysis (Advanced)

### Use Case: CI/CD with Kubernetes

**Scenario:** Enterprise has Kubernetes cluster, wants to distribute BazBOM scans

**Architecture:**

```mermaid
flowchart TB
    Coordinator["Coordinator<br/>(Rust)"]
    Redis["Redis Queue"]
    Worker1["Worker Pod 1"]
    Worker2["Worker Pod 2"]
    Worker3["Worker Pod 3"]
    Coordinator -->|Distribute work| Worker1
    Coordinator -->|Distribute work| Worker2
    Coordinator -->|Distribute work| Worker3
    Worker1 -->|Publish results| Redis
    Worker2 -->|Publish results| Redis
    Worker3 -->|Publish results| Redis
    Coordinator <--|Collect results| Redis
```

**Coordinator:**

```rust
// crates/bazbom/src/distributed/coordinator.rs
use redis::Commands;

pub struct DistributedCoordinator {
    redis: redis::Client,
}

impl DistributedCoordinator {
    pub fn distribute_work(&self, targets: Vec<String>) -> Result<()> {
        let mut conn = self.redis.get_connection()?;

        for target in targets {
            let task = Task {
                target,
                workspace_root: self.workspace_root.clone(),
            };

            conn.rpush("bazbom:work_queue", serde_json::to_string(&task)?)?;
        }

        Ok(())
    }

    pub fn collect_results(&self) -> Result<Vec<ScanResult>> {
        let mut conn = self.redis.get_connection()?;
        let mut results = Vec::new();

        loop {
            let result: Option<String> = conn.lpop("bazbom:results", None)?;
            match result {
                Some(json) => results.push(serde_json::from_str(&json)?),
                None => break,  // No more results
            }
        }

        Ok(results)
    }
}
```

**Worker:**

```rust
// crates/bazbom/src/distributed/worker.rs
pub struct DistributedWorker {
    redis: redis::Client,
}

impl DistributedWorker {
    pub fn run(&self) -> Result<()> {
        let mut conn = self.redis.get_connection()?;

        loop {
            // Fetch task from queue (blocking)
            let task: Option<String> = conn.blpop("bazbom:work_queue", 30)?;

            match task {
                Some(json) => {
                    let task: Task = serde_json::from_str(&json)?;
                    let result = scan_target(&task.target)?;

                    // Push result back
                    conn.rpush("bazbom:results", serde_json::to_string(&result)?)?;
                }
                None => {
                    println!("No work available. Exiting.");
                    break;
                }
            }
        }

        Ok(())
    }
}
```

**Kubernetes Deployment:**

```yaml
# k8s/bazbom-worker.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bazbom-worker
spec:
  replicas: 10  # Scale out to 10 workers
  template:
    spec:
      containers:
      - name: bazbom-worker
        image: bazbom:latest
        command: ["bazbom", "worker", "--redis=redis:6379"]
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
```

**CLI:**

```bash
# Coordinator mode
bazbom scan --distributed --workers=10 --redis=redis:6379

# Worker mode
bazbom worker --redis=redis:6379
```

---

## Performance Targets & Benchmarks

### Target Performance (Phase 8 Complete)

| Scenario | Current | Phase 8 Target | Improvement |
|----------|---------|----------------|-------------|
| **Small repo (100 deps)** | 10s | 5s | 2x faster |
| **Medium repo (1K deps)** | 45s | 20s | 2.25x faster |
| **Large repo (10K deps)** | 3min | 60s | 3x faster |
| **Monorepo (50K targets, full)** | Unknown | <30min | Baseline |
| **Monorepo (50K targets, incremental)** | N/A | <10min | 6x faster |

### Memory Targets

| Scenario | Current | Phase 8 Target | Improvement |
|----------|---------|----------------|-------------|
| **Small repo** | <100MB | <50MB | 2x reduction |
| **Large repo** | <1GB | <4GB | Competitive with EndorLabs |
| **Monorepo (full)** | Unknown | <8GB | vs. EndorLabs' 64GB |

---

## Success Criteria

- [ ] 50K target monorepo full scan completes in <30 minutes
- [ ] Incremental scan (1% of targets changed) completes in <10 minutes
- [ ] Memory usage <4GB for 10K dependency project
- [ ] Parallel processing achieves 4x+ speedup on 8-core machine
- [ ] Cache hit rate >80% in typical development workflow
- [ ] SQLite advisory DB queries <1ms per lookup
- [ ] Zero crashes or OOM errors in 24-hour stress test

---

## Resource Requirements

**Team:** 1-2 developers for 10 weeks
**Skills:** Rust performance tuning, distributed systems (optional), Bazel query optimization
**Budget:** $20K-40K (contractors)

---

**Last Updated:** 2025-10-30
**Next:** Phases 6, 7, 9, 10, 11 (supporting features)
