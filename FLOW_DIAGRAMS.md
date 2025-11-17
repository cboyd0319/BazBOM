# BazBOM Flow Diagrams

## 1. Command Routing Flow

```
main.rs (main() function, line 123)
  │
  ├─ Parse CLI args using Clap
  │   └─ cli::Cli::parse() (line 124)
  │
  ├─ Get command or use default Scan (lines 125-157)
  │
  └─ Match on Commands enum (lines 159-613)
     ├─ Commands::Scan { ... }
     │   └─ handle_scan(...) in commands/scan.rs
     │
     ├─ Commands::Check { path }
     │   ├─ Auto-detect main module
     │   └─ Call handle_scan(path, fast=true, no_upload=true)
     │
     ├─ Commands::Ci { path, out_dir }
     │   └─ Call handle_scan(path, fast=true, format=sarif, json=true)
     │
     ├─ Commands::Pr { path, base, baseline }
     │   └─ Call handle_scan(path, incremental=true, diff=true)
     │
     ├─ Commands::Full { path, out_dir }
     │   └─ Call handle_scan(path, reachability=true, cyclonedx=true, 
     │                                 benchmark=true, ml_risk=true)
     │
     ├─ Commands::Quick { path }
     │   └─ Call handle_scan(path, fast=true, auto_detect_target=true)
     │
     ├─ Commands::ContainerScan { ... }
     │   └─ handle_container_scan(opts) in commands/container_scan.rs
     │
     ├─ Commands::Policy { action }
     │   └─ handle_policy(action) in commands/policy.rs
     │
     ├─ Commands::Fix { ... }
     │   └─ handle_fix(...) in commands/fix.rs
     │
     ├─ Commands::License { action }
     │   └─ handle_license(action) in commands/license.rs
     │
     ├─ Commands::Jira { action }
     │   └─ handle_jira(cmd) in commands/jira.rs
     │
     ├─ Commands::GitHub { action }
     │   └─ handle_github(cmd) in commands/github.rs
     │
     └─ [... other commands ...]
```

## 2. Scan Handler Flow (handle_scan function)

```
handle_scan() in commands/scan.rs
  │
  ├─ Step 1: Smart Defaults (lines 43-80)
  │   ├─ SmartDefaults::detect()
  │   └─ Auto-enable features based on environment
  │       ├─ JSON for CI
  │       ├─ Reachability for small repos
  │       ├─ Incremental for PRs
  │       └─ Diff if baseline found
  │
  ├─ Step 2: Profile Loading (lines 82-88)
  │   └─ Load bazbom.toml profile if specified
  │
  ├─ Step 3: Diff Mode Check (lines 90-99)
  │   └─ If diff && baseline → compare_with_baseline()
  │
  ├─ Step 4: JSON Mode Setup (lines 101-105)
  │   └─ Set BAZBOM_JSON_MODE environment variable
  │
  ├─ Step 5: Check Orchestrator Flags (lines 107-138)
  │   │
  │   └─ If any of: cyclonedx, semgrep, codeql, autofix, containers
  │       │
  │       ├─ Create ScanOrchestrator
  │       └─ Run orchestrator.run() ← ORCHESTRATED PATH
  │           (see below)
  │
  └─ Else: Legacy Scan Path
      └─ handle_legacy_scan(path, reachability, format, ...)
```

## 3. ScanOrchestrator::run() Flow

```
ScanOrchestrator::run() in scan_orchestrator.rs (line 78)
  │
  ├─ Initialize PerformanceMonitor (if benchmark=true)
  │
  ├─ Step 0: Check Incremental (lines 96-107)
  │   ├─ IncrementalAnalyzer::check_incremental_scan()
  │   └─ If no changes detected → return (early exit)
  │
  ├─ Step 0.5: Check Cache (lines 109-127)
  │   ├─ Check BAZBOM_DISABLE_CACHE env var
  │   ├─ try_use_cache()
  │   └─ If cache hit → return (fast path) ⚡
  │
  ├─ Build phases list based on enabled features (lines 130-141)
  │   ├─ SBOM Generation (always)
  │   ├─ SCA Analysis (always)
  │   ├─ Semgrep SAST (if with_semgrep)
  │   ├─ CodeQL Analysis (if with_codeql)
  │   └─ Threat Intel (if threat_detection)
  │
  ├─ Initialize ScanProgress for multi-phase display (line 141)
  │
  ├─ PHASE 1: SBOM Generation (lines 144-155)
  │   ├─ progress.start_phase()
  │   ├─ perf_monitor.start_phase("sbom_generation")
  │   ├─ generate_sbom()
  │   │   └─ Detects build system (Maven, Gradle, Bazel, etc.)
  │   │   └─ Parses dependency files
  │   │   └─ Generates SPDX/CycloneDX
  │   ├─ progress.complete_phase()
  │   └─ perf_monitor.end_phase()
  │
  ├─ PHASE 2: SCA Analysis (lines 160-181)
  │   ├─ progress.start_phase("Fetching vulnerability data...")
  │   ├─ ScaAnalyzer::new()
  │   ├─ sca.run(&context)
  │   │   ├─ ensure_advisory_database()
  │   │   │   ├─ Check manifest age
  │   │   │   └─ If > 24hrs old: db_sync()
  │   │   ├─ load_sbom_components()
  │   │   ├─ Query vulnerability database
  │   │   └─ Enrich with EPSS, KEV
  │   └─ reports.push(sca_report)
  │
  ├─ PHASE 3: Semgrep (optional, lines 184-199)
  │   ├─ SemgrepAnalyzer::run(&context)
  │   ├─ Generate SARIF findings
  │   └─ reports.push(semgrep_report)
  │
  ├─ PHASE 4: CodeQL (optional)
  │   ├─ CodeqlAnalyzer::run(&context)
  │   └─ reports.push(codeql_report)
  │
  ├─ PHASE 5: Threat Detection (optional)
  │   ├─ ThreatAnalyzer::run(&context)
  │   └─ reports.push(threat_report)
  │
  ├─ Merge all reports
  │   └─ merge_sarif_reports(reports)
  │
  ├─ Write outputs
  │   ├─ Write SBOM (SPDX format)
  │   ├─ Write SARIF (findings)
  │   └─ If cyclonedx: write CycloneDX SBOM
  │
  ├─ Optional: Autofix
  │   └─ OpenRewriteRunner (if autofix mode enabled)
  │
  ├─ Optional: Threat Detection
  │   └─ ThreatAnalyzer (if enabled)
  │
  ├─ Optional: Publishing
  │   └─ GitHubPublisher (if not no_upload)
  │
  ├─ Report Performance Metrics
  │   └─ perf_monitor.metrics() if benchmarking
  │
  └─ Return Ok(())
```

## 4. SBOM Generation Pipeline (generate_sbom)

```
ScanOrchestrator::generate_sbom()
  │
  ├─ Detect build system
  │   ├─ Check for pom.xml → Maven
  │   ├─ Check for build.gradle → Gradle
  │   ├─ Check for build.bazel → Bazel
  │   ├─ Check for package.json → npm/Node
  │   ├─ Check for go.mod → Go
  │   ├─ Check for requirements.txt, pyproject.toml → Python
  │   ├─ Check for Cargo.toml → Rust
  │   └─ Check for mix.exs → Elixir
  │
  ├─ Run appropriate parser based on build system
  │   ├─ Maven: parse pom.xml hierarchy
  │   ├─ Gradle: parse build.gradle + dependency graph
  │   ├─ Bazel: extract from maven_install.json
  │   ├─ npm: parse package.json + package-lock.json
  │   ├─ Python: parse setup.py, pyproject.toml, requirements.txt
  │   └─ etc.
  │
  ├─ Build dependency tree with versions
  │
  ├─ Generate PURL (Package URL) for each component
  │
  ├─ Serialize to SPDX format
  │   └─ Write to context.sbom_dir/spdx.json
  │
  └─ (Optional) Serialize to CycloneDX format
      └─ Write to context.sbom_dir/cyclonedx.json
```

## 5. Vulnerability Analysis Pipeline (SCA Analyzer)

```
ScaAnalyzer::run(&context)
  │
  ├─ ensure_advisory_database()
  │   ├─ Check .bazbom/advisories/manifest.json
  │   ├─ If missing or > 24 hours old:
  │   │   └─ db_sync(&cache_dir, false)
  │   │       ├─ Download OSV, NVD, GitHub advisories
  │   │       └─ Update manifest.json
  │   └─ Return cache_dir path
  │
  ├─ load_sbom_components()
  │   ├─ Read context.sbom_dir/spdx.json
  │   ├─ Extract packages array
  │   ├─ For each package:
  │   │   ├─ Get name, version
  │   │   ├─ Extract PURL from externalRefs
  │   │   ├─ Detect ecosystem (maven, npm, python, etc.)
  │   │   └─ Build Component struct
  │   └─ Return Vec<Component>
  │
  ├─ For each component:
  │   │
  │   ├─ Query advisory database
  │   │   ├─ Load advisories for ecosystem
  │   │   └─ Find matching package + version
  │   │
  │   ├─ For each matching vulnerability:
  │   │   ├─ Check if version in vulnerable range
  │   │   │   └─ Use is_version_affected(version, range)
  │   │   ├─ Load EPSS score (if available)
  │   │   ├─ Check if in KEV catalog (exploited)
  │   │   ├─ Determine severity (Critical, High, etc.)
  │   │   └─ Create Finding struct
  │   │
  │   └─ Add Finding to results
  │
  ├─ Build SARIF report
  │   ├─ Create Rule objects for each vulnerability
  │   ├─ Create Result objects for each finding
  │   └─ Set locations, messages, severities
  │
  └─ Return SarifReport
```

## 6. Profile Application Flow

```
Commands::Scan { profile, ... } → handle_scan(profile, ...)
  │
  └─ apply_profile(profile_name, workspace_path)
      │
      ├─ Load bazbom.toml from workspace
      ├─ Find profile by name
      ├─ Apply profile settings:
      │   ├─ Override reachability
      │   ├─ Override format
      │   ├─ Override cyclonedx
      │   ├─ Override with_semgrep
      │   ├─ Override with_codeql
      │   ├─ Override autofix
      │   ├─ Override containers
      │   └─ Override other scan options
      │
      └─ Return merged configuration
          (CLI args take precedence over profile)
```

## 7. Cache Checking Flow

```
ScanOrchestrator::try_use_cache()
  │
  ├─ Build ScanParameters
  │   ├─ Hash of workspace path
  │   ├─ Hashes of all dependency files
  │   ├─ Scan options (reachability, format, etc.)
  │   └─ Tool versions
  │
  ├─ Calculate cache key from parameters
  │
  ├─ Look for .bazbom/cache/{key}/
  │   ├─ Check spdx.json exists
  │   ├─ Check sca_findings.sarif exists
  │   └─ Check manifest.json for metadata
  │
  ├─ Validate cache
  │   ├─ Check scan parameters match
  │   ├─ Check timestamps
  │   └─ Check tool versions
  │
  ├─ If valid:
  │   ├─ Load cached SBOM
  │   ├─ Load cached findings
  │   └─ Return Ok(true) - cache hit
  │
  └─ Else:
      └─ Return Ok(false) - cache miss
```

## 8. Key File I/O Locations

```
Workspace Root
  ├─ bazbom.toml (profiles, configuration)
  │
  ├─ pom.xml (Maven)
  ├─ build.gradle/.kts (Gradle)
  ├─ build.bazel (Bazel)
  ├─ go.mod (Go)
  ├─ Cargo.toml (Rust)
  ├─ package.json (npm/Node)
  ├─ pyproject.toml (Python)
  │
  └─ .bazbom/
      ├─ advisories/
      │   ├─ manifest.json (advisory DB metadata)
      │   ├─ osv.json (vulnerabilities)
      │   ├─ nvd.json (NVD data)
      │   └─ github.json (GitHub advisories)
      │
      ├─ cache/
      │   └─ {cache_key}/
      │       ├─ spdx.json (SBOM)
      │       ├─ cyclonedx.json (CycloneDX SBOM)
      │       ├─ sca_findings.sarif (findings)
      │       └─ manifest.json (cache metadata)
      │
      ├─ sbom/
      │   ├─ spdx.json (current SBOM)
      │   └─ cyclonedx.json (optional)
      │
      ├─ findings/
      │   ├─ sca_findings.sarif (SCA findings)
      │   ├─ semgrep_findings.sarif (if enabled)
      │   ├─ codeql_findings.sarif (if enabled)
      │   └─ all_findings.sarif (merged)
      │
      └─ incremental/
          ├─ baseline.json (previous scan)
          └─ changed_files.txt (tracked changes)

Output Directory
  ├─ sbom.spdx.json
  ├─ sbom.cyclonedx.json (if cyclonedx=true)
  ├─ findings.sarif
  ├─ findings.json
  └─ performance_metrics.json (if benchmark=true)
```

---

## Key Entry Points for Debug Logging

Based on the flow diagrams above, here are the critical sections to instrument:

1. **main.rs:123** - `#[tokio::main] async fn main()`
   - Log: Command parsed, which command selected

2. **commands/scan.rs:43-80** - Smart defaults detection
   - Log: What environment detected, what features auto-enabled

3. **commands/scan.rs:82-88** - Profile loading
   - Log: Profile name, which settings applied

4. **scan_orchestrator.rs:78** - `ScanOrchestrator::run()`
   - Log: Orchestrator initialized, which phases enabled

5. **scan_orchestrator.rs:96-107** - Incremental check
   - Log: Files checked, cache decision reason

6. **scan_orchestrator.rs:109-127** - Cache check
   - Log: Cache key, cache hit/miss, file sizes

7. **scan_orchestrator.rs:144-155** - SBOM generation
   - Log: Build system detected, file count, generation time

8. **analyzers/sca.rs:29-54** - Advisory DB operations
   - Log: Sync decision, download size, time taken

9. **analyzers/sca.rs:56-102** - Vulnerability matching
   - Log: Components loaded, vulnerabilities found, enrichment details

10. **scan_orchestrator.rs:200+** - Report merging and output
    - Log: Files written, total vulnerabilities, report statistics
