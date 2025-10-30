
# BazBOM — Orchestrated SCA + Static Analysis Integration Plan (Bazel 7+)

**Goal:** Keep BazBOM the “single button” JVM supply‑chain tool while **optionally** orchestrating Semgrep and CodeQL, enriching results, and offering **safe autofix**. Everything rolls up into **one SARIF** for GitHub Code Scanning **and** is stored as a build artifact. Developer UX stays brain‑dead simple.

---

## 0) Principles (user experience first)

- **One command, one report:** `bazbom scan .` produces: **SBOM (SPDX + optional CycloneDX)**, **SCA findings**, **optional Semgrep & CodeQL findings**, **enrichment**, plus **optional autofix suggestions/PRs**.
- **Toggles, not traps:** Heavier analyzers are **off by default**; enable with flags or project config.
- **Fast by default:** Scope analyzers to changed modules; cache everything.
- **No security‑engineer degree required:** sensible defaults, minimal knobs, helpful messages, links to fixes.

---

## 1) Architecture overview

```
bazbom scan .
 ├─ sbom/
 │   ├─ spdx.json (always)
 │   └─ cyclonedx.json (opt-in)
 ├─ findings/
 │   ├─ sca.sarif           # OSV/NVD/GHSA mapped to SBOM
 │   ├─ semgrep.sarif       # optional (curated JVM rules)
 │   ├─ codeql.sarif        # optional (default/security-extended)
 │   └─ merged.sarif        # single upload target
 ├─ enrich/
 │   └─ depsdev.json        # license, popularity, version intel
 ├─ fixes/
 │   └─ openrewrite/        # generated recipes & dry-run patches
 └─ publish/
     ├─ github-code-scanning: merged.sarif
     └─ artifact: all outputs
```

**Data model:** Internal merge layer normalizes tool metadata and **de-duplicates SARIF results**. Keep one run per tool in the SARIF `runs` array. (GitHub requires **SARIF 2.1.0**.)

**Planned publishers:** GitHub Code Scanning + artifact. Future: **GUAC**/**Dependency‑Track**.

**Attestations:** Adopt **bazel-contrib/supply-chain** for build metadata and future provenance.

Sources: Semgrep rules/registry, CodeQL query packs, deps.dev API, GUAC, Syft & CycloneDX, SARIF upload, CodeQL Java/Kotlin. [Citations at end.]

---

## 2) CLI & config

### CLI flags (short & readable)
- `--with-semgrep` — run Semgrep with BazBOM’s curated JVM ruleset (pinned SHAs).
- `--with-codeql[=suite]` — run CodeQL (`default` or `security-extended` suites).  
  e.g. `--with-codeql=security-extended`
- `--cyclonedx` — also emit CycloneDX (for interop and future publishers).
- `--autofix[=dry-run|pr]` — generate OpenRewrite recipes; `dry-run` by default; `pr` opens PRs.
- `--containers[=auto|syft|bazbom]` — container SBOM strategy. Default `auto` uses fastest available; fall back to **Syft** until BazBOM’s path is measurably faster.
- `--no-upload` — skip GitHub upload (local dev), still writes artifacts.
- `--target <module>` — limit to one module (PR/changed-path speedups).

### Project config (`bazbom.toml`)
```toml
[analysis]
cyclonedx = true
semgrep = { enabled = true, ruleset = "curated-jvm@sha256:..." }
codeql  = { enabled = false, suite = "default" }

[enrich]
depsdev = true

[autofix]
mode = "dry-run" # "off" | "dry-run" | "pr"
recipe_allowlist = ["commons-io", "jackson", "log4j", "spring-core"]

[containers]
strategy = "auto" # "auto" | "syft" | "bazbom"

[publish]
github_code_scanning = true
artifact = true
```

---

## 3) Semgrep integration (optional, fast)

**What we reuse:** Official **semgrep-rules** + curated third‑party JVM rules. BazBOM vendors a **pinned** ruleset (`rules/semgrep-jvm.yml`) and updates on a cadence.

**Runner:** `semgrep --config rules/semgrep-jvm.yml --json --sarif --timeout 120` against source files of only the **affected modules** (discovered from the build graph/SBOM). Respect `.semgrepignore` and BazBOM’s generated ignores (e.g., build dirs).

**Output:** `findings/semgrep.sarif` merged into `findings/merged.sarif` with a distinct `tool.driver.name = "Semgrep"` and `automationDetails`.


**Why:** Semgrep finds pattern‑level issues (insecure APIs, deserialization pitfalls) cheaply in PRs—complements SCA without heavyweight DB builds.

---

## 4) CodeQL integration (optional, deep)

**Maven/Gradle:** Use CodeQL autobuild.  
**Bazel:** Provide `bazbom codeql create-db` helper that builds class files via known Bazel targets, then runs `codeql database create --language=java --command '<build>'` and `codeql database analyze` with chosen suite (`default` or `security-extended`).

**Output:** `findings/codeql.sarif`, merged as separate `runs` entry.

**Why:** Taint/dataflow turns “package has CVE” into “reachable through this handler” and increases fix precision.

---

## 5) SBOM + SCA core (always on)

- **SBOM:** SPDX 2.3 always; `--cyclonedx` for interop. Optional internal use of **protobom** for lossless read/write if we need a neutral core representation.
- **SCA:** OSV/NVD/GHSA mapped to SBOM components → `findings/sca.sarif`.
- **Enrichment:** Query **deps.dev** by PURL for licenses, version streams, reverse‑deps/popularity; store in `enrich/depsdev.json` and surface “sane next version” in fix hints.

---

## 6) Autofix (always optional)

Use **OpenRewrite** recipes to:
- Upgrade unsafe dependencies (Maven/Gradle) with pinned safe ranges.
- Replace deprecated/unsafe artifacts (e.g., `commons-io:2.6 → 2.14.x`).

Modes:
- `off` — no changes; still propose recipes.
- `dry-run` — generate patches to `fixes/openrewrite/` and attach diffs in the SARIF `help`/`properties`.
- `pr` — open branch + PR with recipes applied; link each SARIF result to the PR diff.

**Safety rails:** allowlist packages, require green build, never mass‑edit across modules unless the build passes.

---

## 7) Containers

- **Default:** `--containers=auto` attempts BazBOM’s image discovery; if BazBOM’s generator is slower or incomplete, wrap **Syft** for image/filesystem SBOM and convert as needed.
- Measured goal: replace Syft only when “time-to-SBOM” and coverage are **as good or better**.

---

## 8) Publishing

- **GitHub Code Scanning:** Upload **`merged.sarif` (SARIF 2.1.0)** via the official `upload-sarif` step.
- **Artifacts:** Upload the whole `/sbom`, `/findings`, `/enrich`, and `/fixes` directories using `actions/upload-artifact` v4.
- **Future (planned):** Optional GUAC/Dependency‑Track publishers, gated by config.

---

## 9) GitHub Actions — drop‑in workflows

```yaml
name: BazBOM Scan
on:
  pull_request:
    paths-ignore: [ '**/*.md' ]
  push:
    branches: [ main ]
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up JDK
        uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: '21'

      - name: Install tools (Semgrep, CodeQL CLI)
        run: |
          pipx install semgrep
          CODEQL_VERSION=2.19.4
          curl -sL https://github.com/github/codeql-cli-binaries/releases/download/v${CODEQL_VERSION}/codeql-linux64.zip -o codeql.zip
          unzip -q codeql.zip -d $HOME && echo "$HOME/codeql" >> $GITHUB_PATH

      - name: BazBOM scan (fast defaults)
        run: |
          bazbom scan . --cyclonedx --with-semgrep

      - name: (Main only) deep analysis
        if: github.ref == 'refs/heads/main'
        run: |
          bazbom scan . --cyclonedx --with-semgrep --with-codeql=security-extended --autofix=dry-run

      - name: Upload SARIF to GitHub Code Scanning
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: findings/merged.sarif

      - name: Upload artifacts (SBOMs + findings + fixes + enrichment)
        uses: actions/upload-artifact@v4
        with:
          name: bazbom-results-${{ github.run_id }}
          path: |
            sbom/**
            findings/**
            enrich/**
            fixes/**
```

Notes:
- Keep CodeQL only on `main` or nightly to control cost/time.
- Ensure `merged.sarif` keeps separate `runs` for each tool (GitHub no longer combines identical runs).

---

## 10) Bazel (7+) wiring

- Add **bazel-contrib/supply-chain** to collect build metadata and enable future attestations.
- Provide Starlark macros:
  - `bazbom_sbom(name, targets=[...])`
  - `bazbom_semgrep(name, srcs=[...])`
  - `bazbom_codeql(name, targets=[...], suite="default")`
  - `bazbom_merge(name, inputs=[...])`
- Expose a top‑level `:bazbom_all` that depends on the above and writes outputs to predictable paths inside `bazel-bin/` which the CLI then syncs to the working `sbom/`, `findings/`, etc.

---

## 11) Performance tactics

- **Scope by change:** map PR diffs → modules via build graph/SBOM; analyze only changed modules by default.
- **Cache:** CodeQL DB cache keyed by commit + compiler inputs; Semgrep rule cache keyed by ruleset SHA.
- **Timeouts:** Cap Semgrep; default CodeQL to “default” suite unless `main`/nightly.
- **Parallel:** Run Semgrep and SCA concurrently; CodeQL as a separate phase.

---

## 12) Dev ergonomics

- **Actionable messages:** Every finding includes: coordinate/file, short title, risk, **fix command** (`bazbom fix --recipe ...`) or PR link, and **why it’s the chosen version** (deps.dev evidence).
- **Docs:** “90‑second quickstart” in README; `--help` shows only the top 10 flags; `--help all` for the rest.
- **Escape hatches:** `--no-upload`, `--target`, `--max-findings`, `--fail-on-critical`.

---

## 13) Security posture

- Pin external rules/packs by SHA or version.
- Validate SARIF against the schema before upload; bail with a clear error.
- Never run autofix PRs without passing builds; allowlist upgrades only.

---

## 14) Rollout plan

1. **MVP (2 weeks):** SBOM+SCA merge; Semgrep optional; SARIF merge + upload; artifacts.  
2. **Phase 2:** CodeQL optional (Maven/Gradle autobuild first); deps.dev enrichment; OpenRewrite dry‑run.  
3. **Phase 3:** Bazel CodeQL path + PR‑autofix; container auto with Syft fallback; performance polish.  
4. **Phase 4 (planned):** Optional publishers for **GUAC**/**Dependency‑Track**.

---

## 15) Open questions / decisions to lock

- Default CodeQL suite for `main`? (`default` vs `security-extended`)
- Package allowlist for autofix PRs: start with `log4j`, `jackson`, `spring-core`, `commons-*`?
- Container strategy: baseline with Syft now; measure and replace when we hit parity.

---

## Citations & refs

- **Semgrep rules/registry & usage:** semgrep-rules; registry & run-all; writing rules. citeturn0search0turn0search4turn0search8turn0search24
- **CodeQL Java/Kotlin & packs:** built-in queries; packs; language guide; query help. citeturn0search1turn0search5turn0search17turn0search9turn0search25
- **deps.dev (Open Source Insights):** API v3/alpha; docs; GitHub repo; blog. citeturn0search6turn0search2turn0search18turn0search10turn0search26
- **GUAC:** site; GitHub; OpenSSF; 1.0 announcement. citeturn0search3turn0search11turn0search7turn0search23
- **CycloneDX & Syft:** CycloneDX Maven plugin; Syft repo & blog; Tool Center. citeturn1search2turn1search7turn1search3turn1search13turn2search14
- **OSV‑Scanner:** project; site; GitHub Action. citeturn1search1turn1search6turn1search11
- **Bazel supply-chain:** repo; org; ecosystem note. citeturn1search0turn1search15turn1search20
- **OpenRewrite:** upgrade/change dependency recipes. citeturn2search0turn2search4turn2search8
- **SARIF upload & artifacts:** GitHub docs; upload-artifact action; SARIF support note. citeturn2search1turn2search2turn1search9


---

# Appendix — Rust Implementation Details

This appendix adds a **Rust-first execution plan**: module layout, core traits for analyzers/publishers, a **tool cache with checksum verification**, and a **starter scaffold** you can paste into a new crate. The goal is a single, static binary that’s safe, fast, and boring to operate.

## A.1 Module layout (proposed)

```
src/
├─ main.rs                  # CLI entrypoint
├─ cli.rs                   # clap args
├─ config.rs                # bazbom.toml types
├─ context.rs               # run context (paths, timeouts, cache)
├─ pipeline.rs              # orchestrates analyzers and merge
├─ util/
│  ├─ fs.rs                 # atomic writes, path helpers
│  ├─ hash.rs               # sha256 helpers
│  ├─ git.rs                # changed-files → module map
│  └─ logging.rs            # tracing setup
├─ toolchain/
│  ├─ mod.rs
│  ├─ tool_cache.rs         # download, verify sha256, pin versions
│  ├─ sandbox.rs            # safe subprocess wrapper (timeouts, env)
│  ├─ semgrep.rs            # Semgrep runner
│  ├─ codeql.rs             # CodeQL runner
│  └─ syft.rs               # Syft runner (optional container SBOM)
├─ formats/
│  ├─ mod.rs
│  ├─ sarif.rs              # serde types + merge/dedupe
│  ├─ spdx.rs               # write SPDX 2.3
│  └─ cyclonedx.rs          # optional CycloneDX emitter
├─ enrich/
│  └─ depsdev.rs            # deps.dev client (purl → intel)
├─ fixes/
│  ├─ openrewrite.rs        # recipe generation + apply
│  └─ pr.rs                 # PR creation (GitHub API)
└─ publish/
   ├─ github.rs             # upload SARIF
   └─ artifact.rs           # local archiving, etc.
```

## A.2 Core traits

Keep analyzers/publishers pluggable so toggles are trivial.

```rust
// formats/sarif.rs (simplified)
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifLog {
    pub version: String, // "2.1.0"
    pub runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifRun {
    pub tool: SarifTool,
    pub results: Vec<SarifResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automationDetails: Option<SarifAutomation>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifTool { pub driver: SarifDriver }
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifDriver { pub name: String, pub version: Option<String> }
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifAutomation { pub id: Option<String> }
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SarifResult { /* location, ruleId, message, properties, ... */ }

pub fn merge_runs(mut lhs: SarifLog, mut rhs: SarifLog) -> SarifLog {
    lhs.runs.append(&mut rhs.runs);
    // optional: dedupe identical results here
    lhs
}
```

```rust
// pipeline.rs
use crate::{config::Config, context::Context};
use crate::formats::sarif::SarifLog;
use async_trait::async_trait;

#[async_trait]
pub trait Analyzer: Send + Sync {
    fn id(&self) -> &'static str;
    fn enabled(&self, cfg: &Config) -> bool;
    async fn run(&self, ctx: &Context) -> anyhow::Result<SarifLog>;
}

#[async_trait]
pub trait Publisher: Send + Sync {
    fn id(&self) -> &'static str;
    fn enabled(&self, cfg: &Config) -> bool;
    async fn publish(&self, ctx: &Context, sarif: &SarifLog) -> anyhow::Result<()>;
}
```

Register analyzers in a list and iterate by `enabled(cfg)` to keep the main flow tiny.

## A.3 Tool cache with checksum verification

**Goal:** fetch pinned external tools (Semgrep, CodeQL, Syft), verify **SHA‑256**, store in a cache dir, and return absolute paths for subprocess runners.

- Cache root: `~/.cache/bazbom/tools/<name>/<version>/<os-arch>/`
- Manifest (TOML or JSON) checked into repo (e.g., `tool-versions.toml`):
  ```toml
  [semgrep]
  version = "1.78.0"
  linux_x86_64 = { url = "https://…/semgrep-1.78.0-linux", sha256 = "..." }
  macos_aarch64 = { url = "https://…/semgrep-1.78.0-macos-aarch64", sha256 = "..." }

  [codeql]
  version = "2.19.4"
  linux_x86_64 = { url = "https://…/codeql-linux64.zip", sha256 = "..." }
  ```

Minimal implementation:

```rust
// toolchain/tool_cache.rs
use std::{fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use reqwest::Client;

pub struct ToolDescriptor<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub url: &'a str,
    pub sha256: &'a str,
    pub executable: bool,
    pub archive: bool, // zip/tar.gz
}

pub struct ToolCache { root: PathBuf, http: Client }

impl ToolCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root, http: Client::new() }
    }

    pub async fn ensure(&self, desc: &ToolDescriptor<'_>) -> Result<PathBuf> {
        let dir = self.root.join(desc.name).join(desc.version);
        let marker = dir.join(".ok");
        let bin = dir.join(self.binary_name(desc.name));

        if marker.exists() && bin.exists() {
            return Ok(bin);
        }
        fs::create_dir_all(&dir)?;

        let tmp = NamedTempFile::new_in(&dir)?;
        let mut resp = self.http.get(desc.url).send().await?.error_for_status()?;
        let mut file = tmp.reopen()?;
        let mut hasher = Sha256::new();

        while let Some(chunk) = resp.chunk().await? {
            use std::io::Write;
            file.write_all(&chunk)?;
            hasher.update(&chunk);
        }
        let digest = format!("{:x}", hasher.finalize());
        anyhow::ensure!(digest == desc.sha256, "sha256 mismatch for {}", desc.name);

        let final_path = dir.join("download");
        tmp.persist(&final_path)?;

        // Extract or mark executable
        if desc.archive {
            // unzip/tar into dir; set `bin` path accordingly
            // (left as exercise; use `zip`/`tar` crates)
        } else {
            fs::rename(&final_path, &bin)?;
        }

        #[cfg(unix)]
        if desc.executable {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = fs::metadata(&bin)?.permissions();
            perm.set_mode(0o555);
            fs::set_permissions(&bin, perm)?;
        }

        fs::write(&marker, b"ok")?;
        Ok(bin)
    }

    fn binary_name(&self, name: &str) -> String {
        if cfg!(windows) { format!("{name}.exe") } else { name.to_string() }
    }
}
```

## A.4 Safe subprocess execution

```rust
// toolchain/sandbox.rs
use std::path::Path;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use anyhow::{Context, Result};

pub async fn run_tool(bin: &Path, args: &[&str], cwd: &Path, secs: u64) -> Result<(i32, String, String)> {
    let mut cmd = Command::new(bin);
    cmd.args(args).current_dir(cwd);

    // Minimal env: PATH only (or a curated whitelist)
    cmd.env_clear();
    if let Some(path) = std::env::var_os("PATH") { cmd.env("PATH", path); }

    let mut child = cmd.stdout(std::process::Stdio::piped())
                      .stderr(std::process::Stdio::piped())
                      .spawn()
                      .with_context(|| format!("spawn {:?}", bin))?;

    let out = timeout(Duration::from_secs(secs), child.wait_with_output()).await
        .context("timeout")??;

    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Ok((code, stdout, stderr))
}
```

## A.5 Starter scaffold (copy/paste)

**Cargo.toml**

```toml
[package]
name = "bazbom"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
async-trait = "0.1"
clap = { version = "4", features = ["derive"] }
directories = "5"
reqwest = { version = "0.12", features = ["json", "gzip", "brotli", "deflate", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
tempfile = "3"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "time"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "env-filter"] }
```

**src/cli.rs**

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bazbom")]
pub struct Args {
    #[arg(long)] pub cyclonedx: bool,
    #[arg(long)] pub with_semgrep: bool,
    #[arg(long)] pub with_codeql: Option<CodeqlSuite>,
    #[arg(long)] pub autofix: Option<AutoFixMode>,
    #[arg(long)] pub containers: Option<ContainerStrategy>,
    #[arg(long)] pub no_upload: bool,
    #[arg(long)] pub target: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CodeqlSuite { Default, SecurityExtended }

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum AutoFixMode { Off, DryRun, Pr }

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ContainerStrategy { Auto, Syft, Bazbom }
```

**src/config.rs**

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub analysis: Analysis,
    pub enrich: Enrich,
    pub autofix: AutoFix,
    pub containers: Containers,
    pub publish: Publish,
}

#[derive(Debug, Deserialize, Default)]
pub struct Analysis {
    pub cyclonedx: Option<bool>,
    pub semgrep: Option<bool>,
    pub codeql: Option<Codeql>,
}
#[derive(Debug, Deserialize, Default)]
pub struct Codeql { pub enabled: Option<bool>, pub suite: Option<String> }

#[derive(Debug, Deserialize, Default)]
pub struct Enrich { pub depsdev: Option<bool> }

#[derive(Debug, Deserialize, Default)]
pub struct AutoFix { pub mode: Option<String> }

#[derive(Debug, Deserialize, Default)]
pub struct Containers { pub strategy: Option<String> }

#[derive(Debug, Deserialize, Default)]
pub struct Publish { pub github_code_scanning: Option<bool>, pub artifact: Option<bool> }
```

**src/context.rs**

```rust
use directories::ProjectDirs;
use std::path::PathBuf;

pub struct Context {
    pub workspace: PathBuf,
    pub out_dir: PathBuf,
    pub tool_cache: PathBuf,
}

impl Context {
    pub fn new() -> anyhow::Result<Self> {
        let pwd = std::env::current_dir()?;
        let proj = ProjectDirs::from("io", "BazBOM", "bazbom")
            .ok_or_else(|| anyhow::anyhow!("no project dirs"))?;
        let cache = proj.cache_dir().to_path_buf();
        let out = pwd.join("findings");
        std::fs::create_dir_all(&out)?;
        std::fs::create_dir_all(&cache)?;
        Ok(Self { workspace: pwd, out_dir: out, tool_cache: cache })
    }
}
```

**src/pipeline.rs** (wire-up example)

```rust
use crate::{config::Config, context::Context, formats::sarif::{SarifLog, merge_runs}};
use crate::toolchain::{semgrep::SemgrepAnalyzer, codeql::CodeqlAnalyzer};
use crate::pipeline::Analyzer;
use anyhow::Result;

pub async fn run_all(cfg: &Config, ctx: &Context) -> Result<SarifLog> {
    let mut merged = SarifLog { version: "2.1.0".into(), runs: vec![] };

    let analyzers: Vec<Box<dyn Analyzer>> = vec![
        Box::new(SemgrepAnalyzer::default()),
        Box::new(CodeqlAnalyzer::default()),
        // SCA analyzer would be another entry returning SARIF from OSV/NVD/GHSA mapping
    ];

    for a in analyzers {
        if a.enabled(cfg) {
            let sarif = a.run(ctx).await?;
            merged = merge_runs(merged, sarif);
        }
    }
    Ok(merged)
}
```

**src/toolchain/semgrep.rs** (stub)

```rust
use async_trait::async_trait;
use anyhow::{Result, Context};
use std::path::PathBuf;
use crate::{config::Config, context::Context as RunCtx, pipeline::Analyzer, formats::sarif::{SarifLog, SarifRun, SarifTool, SarifDriver}};
use crate::toolchain::{tool_cache::ToolCache, sandbox::run_tool};

#[derive(Default)]
pub struct SemgrepAnalyzer;

#[async_trait]
impl Analyzer for SemgrepAnalyzer {
    fn id(&self) -> &'static str { "semgrep" }
    fn enabled(&self, cfg: &Config) -> bool {
        cfg.analysis.semgrep.unwrap_or(false)
    }

    async fn run(&self, ctx: &RunCtx) -> Result<SarifLog> {
        let cache = ToolCache::new(ctx.tool_cache.clone());
        // Resolve from manifest (hard-coded here for brevity)
        let desc = crate::toolchain::tool_cache::ToolDescriptor{
            name: "semgrep",
            version: "1.78.0",
            url: "https://example.invalid/semgrep", // fill in real URL
            sha256: "deadbeef",
            executable: true,
            archive: false,
        };
        let bin = cache.ensure(&desc).await?;

        let rules = PathBuf::from("rules/semgrep/semgrep-jvm.yml");
        let args = ["--config", rules.to_str().unwrap(), "--sarif", "--json", "."];
        let (_code, stdout, _stderr) = run_tool(&bin, &args, &ctx.workspace, 120).await
            .context("running semgrep")?;

        // Deserialize semgrep SARIF from stdout string
        let sarif: SarifLog = serde_json::from_str(&stdout)
            .context("parse semgrep SARIF")?;

        // Attach tool metadata if needed
        Ok(sarif)
    }
}
```

**src/toolchain/codeql.rs** (stub)

```rust
use async_trait::async_trait;
use anyhow::{Result};
use crate::{config::Config, context::Context as RunCtx, pipeline::Analyzer, formats::sarif::SarifLog};

#[derive(Default)]
pub struct CodeqlAnalyzer;

#[async_trait]
impl Analyzer for CodeqlAnalyzer {
    fn id(&self) -> &'static str { "codeql" }
    fn enabled(&self, cfg: &Config) -> bool {
        cfg.analysis.codeql.as_ref().and_then(|c| c.enabled).unwrap_or(false)
    }
    async fn run(&self, _ctx: &RunCtx) -> Result<SarifLog> {
        // Create DB, analyze with chosen pack, parse SARIF
        // (left as exercise: choose suite from config)
        Ok(SarifLog { version: "2.1.0".into(), runs: vec![] })
    }
}
```

**src/main.rs**

```rust
mod cli; mod config; mod context; mod pipeline;
mod formats; mod toolchain;

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt::SubscriberBuilder};
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SubscriberBuilder::default()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .init();

    let args = cli::Args::parse();
    let cfg = read_config()?;
    let ctx = context::Context::new()?;

    let sarif = pipeline::run_all(&cfg, &ctx).await?;

    fs::create_dir_all("findings")?;
    let out = "findings/merged.sarif";
    fs::write(out, serde_json::to_vec_pretty(&sarif)?)?;
    println!("Wrote {out}");

    Ok(())
}

fn read_config() -> anyhow::Result<config::Config> {
    let s = std::fs::read_to_string("bazbom.toml").unwrap_or_default();
    Ok(toml::from_str(&s).unwrap_or_default())
}
```

**bazbom.toml (sample)**

```toml
[analysis]
cyclonedx = true
semgrep = true
codeql = { enabled = false, suite = "default" }

[enrich]
depsdev = true

[autofix]
mode = "dry-run"

[containers]
strategy = "auto"

[publish]
github_code_scanning = true
artifact = true
```

> Tip: keep the SARIF writer lean initially (no dedupe). Add dedupe once you see real-world duplicates.

---

## A.6 Next steps (Rust work items)

- Implement **tool manifest** loader and platform resolution (OS/arch → URL/SHA).  
- Fill in real Semgrep/CodeQL/Syft URLs + SHAs and extraction logic.  
- Add **SCA analyzer** producing SARIF from your SPDX mapping + OSV/NVD/GHSA.  
- Implement **deps.dev client** and enrich SARIF `properties` with upgrade hints.  
- Wire **OpenRewrite** dry-run and PR flow.  
- Add **GitHub publisher** (SARIF upload) and **artifact** archiving.  
- Ship minimal integration tests: sample Maven/Gradle/Bazel projects.

That’s enough to compile a working binary, run Semgrep on a repo, and write a merged SARIF for uploading in your Action.


---

# Enhancements & Advanced Options

This section rolls in further upgrades focused on **signal quality**, **trustworthy autofix**, **performance**, **developer UX**, **hardening**, and **governance**. All are additive and optional; defaults stay friendly for PR workflows.

## E.1 Signal quality & triage

- **Exploit-aware prioritization:** Enrich SCA with **EPSS**, **CISA KEV**, and **SSVC** attributes. Add to SARIF `properties` per finding:
  - `properties.epss_score` (0–1), `properties.epss_percentile`
  - `properties.cisa_kev`: `true|false` (with date_added)
  - `properties.ssvc_decision`: e.g., `Act | Track | Defer`
- **Reachability (JVM):** Bytecode call-graph analysis (e.g., OPAL/Soot) to flag `properties.reachable = true|false` and capture the **top call chain** sample for PRs.
- **VEX support:** Accept repo/org VEX statements to mark **Not Affected**, **Affected**, **Fixed**, **Under Investigation**. Show as suppressions with expiry.

## E.2 Autofix that earns trust

- **Guardrail pipeline:** Before opening a PR, run `compile → unit tests → smoke/integration → minimal runtime boot`. Abort PR on failure; attach logs to SARIF and PR body.
- **Recipe provenance:** Version, sign (optional), and print the **exact OpenRewrite recipe** and rationale in the PR body; include a **Revert** recipe.
- **API compatibility preview:** For libraries, compute an **API diff** (e.g., japicmp) for major/minor changes; default upgrades to **patch/minor** unless `--allow-major` is set.

## E.3 Orchestration & performance

- **Incremental PR mode:** Analyze only modules reachable from changed files (build graph + SBOM); deep scans (CodeQL security-extended) reserved for `main`/nightly.
- **Rule profiles:** Semgrep presets: `minimal` (PR), `balanced` (default), `paranoid` (nightly). Switchable via CLI/config.
- **Timeout budgets:** Global PR budget (e.g., 12m). Allocate slices per tool; soft-cancel laggards with a clear SARIF note.

## E.4 UX upgrades

- **PR summary table:** One BazBOM check posts: severity, EPSS, KEV, reachability, fix version, and a button/link to **Autofix PR** when available.
- **IDE loop:** Optional VS Code/JetBrains extension runs `bazbom scan --quick` on staged changes and renders SARIF.
- **Helpful failures:** Tool crashes produce a “what failed and how to fix it” postcard in the PR (don’t make devs read 10k lines of logs).

## E.5 Rust hardening

- **Toolchain integrity:** Pin versions + SHA-256, and where supported verify **Sigstore/cosign** signatures. Print `tool.name@version` in SARIF `automationDetails`.
- **Sandboxing:** Namespaces/seccomp (Linux), job objects (Windows), strict env and rlimits (all). No shells; args only. Reject path traversal and symlink escapes.
- **Offline mode:** `--offline` uses cached advisories and disables network calls; good for hermetic CI.

## E.6 Policy & governance

- **Policy-as-code:** Optional **OPA/Rego** or **CEL** rules evaluate the merged model (severity, EPSS, KEV, reachability, VEX) → pass/fail and required actions.
- **Baselines with expiry:** Allow repo/team baselines in `bazbom.toml` (suppressed CVEs with `expires_at`). Surface upcoming expirations in PRs.

## E.7 Container SBOM path (replace Syft when ready)

- **Benchmark suite:** Repro cases (distroless, alpine, glibc, fat Spring Boot). Measure **coverage**, **package count**, **time-to-SBOM**. Flip default from Syft → BazBOM only when we win consistently.
- **Layer-aware diffs:** For image upgrades, emit “what changed” per layer and show newly introduced deps.

## E.8 Observability & reliability

- **Traces:** `tracing` spans per phase (sbom, sca, semgrep, codeql, enrich, merge, publish) with durations and cache hits. Print a timing table at the end.
- **Doctor:** `bazbom doctor` checks: tool cache, sandbox support, JDK presence, network reachability (deps.dev), and GitHub token configuration.
- **Retry policy:** Exponential backoff for tool downloads; checksum mismatch aborts with exact URL and expected vs actual digest.

## E.9 Formats & compatibility

- **Schema validation:** Validate SPDX/CycloneDX/SARIF; fail loud with line/field details.
- **Neutral core (optional):** Use a **protobom-like** core model to minimize loss between formats and make merges deterministic.

## E.10 Docs & onboarding

- **90‑second quickstart** GIF/video in README, plus a “How BazBOM prioritizes risk” page with EPSS/KEV/Reachability/VEX examples.
- **Sample repos:** Minimal Maven, Gradle, Bazel projects with intentional vulns; golden SARIF snapshots in tests.

---

# Risk Scoring Data Model (example)

BazBOM computes a `risk_score` (0–100) and a `priority` bucket. Suggested formula (tweakable in policy):

```
risk_score =
  w1 * base_severity(cvss) +
  w2 * exploit_probability(epss) +
  w3 * kev_present +
  w4 * reachability +
  w5 * asset_criticality +
  w6 * time_since_disclosure

priority:
  80–100  -> P0  (gate merge, open autofix PR immediately)
  60–79   -> P1  (block if reachable or KEV; else warn)
  40–59   -> P2  (warn, fix within sprint)
  <40     -> P3  (track)
```

Where:
- `base_severity(cvss)` maps to 0–100 (e.g., CVSSv3 base 9.8 ≈ 98).
- `exploit_probability(epss)` is EPSS percentile (0–100).
- `kev_present` = 100 if in KEV, else 0.
- `reachability` = 100 if reachable path exists, else 0.
- `asset_criticality` can default to 50 and be set per module in config.
- `time_since_disclosure` boosts fresh vulns to reduce window of exposure.

All components and weights are exposed in SARIF `properties` and evaluated by policy.

---

# Policy-as-Code Example (OPA/Rego)

```rego
package bazbom.policy

default allow = true
default block = []

# Inputs: merged finding document with properties populated
reach := input.properties.reachable
kev := input.properties.cisa_kev
epss := input.properties.epss_percentile
cvss := input.properties.cvss_base
priority := input.properties.priority

# P0: Block if KEV or reachable Critical and EPSS >= 70
violations[v] {
  kev == true
  v := {"id": input.ruleId, "reason": "CISA KEV listed"}
}

violations[v] {
  reach == true
  cvss >= 9.0
  epss >= 70
  v := {"id": input.ruleId, "reason": "Reachable critical with high EPSS"}
}

allow {
  count(violations) == 0
}

block := violations
```

You can run this after merging SARIF and fail the job if `count(block) > 0`. Expose a `--policy` flag pointing at policy bundles.

---

# Container Benchmark Recipe (Syft vs. BazBOM)

**Goal:** replace Syft only when BazBOM wins on **coverage** and **time-to-SBOM**.

1. **Corpus:**
   - `gcr.io/distroless/java21:latest`
   - `eclipse-temurin:21-jre-alpine`
   - `amazoncorretto:21`
   - A fat Jar Spring Boot image (your sample app)
2. **Metrics:**
   - Wall time (cold/warm cache)
   - Package count by ecosystem (apk/rpm/deb/java)
   - % packages with license populated
3. **Method:**
   - Run each tool 5x (first run cold cache, then warm).
   - Discard outliers (min/max); keep median.
4. **Threshold to flip default:**
   - BazBOM time <= Syft time × **0.95** (faster or equal)
   - Package coverage within **±2%** or better
   - Licenses coverage within **±2%** or better
5. **Output:**
   - Markdown table + JSON with raw numbers
   - Attach to release notes; print in CI when `--containers=bazbom` enabled

---

# CLI additions

- `--risk-profile <strict|default|relaxed>` tunes thresholds/weights.
- `--policy <path-or-bundle>` runs OPA/CEL against merged findings.
- `--vex-out` emits VEX statements for suppressions/decisions.
- `--offline` disables network calls (deps.dev, downloads).

Defaults remain conservative and dev-friendly.

---
