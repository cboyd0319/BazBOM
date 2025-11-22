use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bazbom", version, about = "JVM SBOM, SCA, and dependency graph tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Scan a project and generate SBOM + findings
    #[command(after_help = "EXAMPLES:
  # Quick local scan
  bazbom scan

  # Full scan with reachability (production-ready)
  bazbom scan --reachability

  # CI/CD optimized
  bazbom scan --json --format sarif > findings.sarif

  # Scan only changed code in PR
  bazbom scan --incremental --base main

  # Compare with last week's scan
  bazbom scan --diff --baseline baseline.json

  # Use pre-configured profile
  bazbom scan --profile production

QUICK COMMANDS:
  bazbom check   → Fast local dev scan
  bazbom ci      → CI-optimized (JSON + SARIF)
  bazbom pr      → PR mode (incremental + diff)
  bazbom full    → Everything enabled
  bazbom quick   → 5-second smoke test

PROFILES:
  Run 'bazbom init' to create bazbom.toml with profiles.
  Example profiles: dev, ci, production, strict")]
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Use a named profile from bazbom.toml (e.g., strict, fast, ci)
        #[arg(long, short = 'p', value_name = "PROFILE")]
        profile: Option<String>,
        /// Enable reachability analysis (OPAL)
        #[arg(long, short = 'r')]
        reachability: bool,
        /// Fast mode: skip reachability analysis for speed (<10s scans)
        #[arg(long)]
        fast: bool,
        /// Output format (spdx|cyclonedx)
        #[arg(long, short = 'f', default_value = "spdx")]
        format: String,
        /// Output directory (defaults to current directory)
        #[arg(long, short = 'o', value_name = "DIR", default_value = ".")]
        out_dir: String,
        /// Output results in JSON format (machine-readable)
        #[arg(long)]
        json: bool,
        /// Bazel-specific: query expression to select targets
        #[arg(long, value_name = "QUERY")]
        bazel_targets_query: Option<String>,
        /// Bazel-specific: explicit list of targets to scan
        #[arg(long, value_name = "TARGET", num_args = 1..)]
        bazel_targets: Option<Vec<String>>,
        /// Bazel-specific: scan only targets affected by these files
        #[arg(long, value_name = "FILE", num_args = 1..)]
        bazel_affected_by_files: Option<Vec<String>>,
        /// Bazel-specific: universe pattern for rdeps queries
        #[arg(long, value_name = "PATTERN", default_value = "//...")]
        bazel_universe: String,
        /// Bazel-specific: exclude targets matching pattern (e.g., //tests/..., //vendor/...)
        #[arg(long, value_name = "PATTERN", num_args = 0..)]
        bazel_exclude_targets: Option<Vec<String>>,
        /// Bazel-specific: path to Bazel workspace (for non-root workspaces)
        #[arg(long, value_name = "PATH")]
        bazel_workspace_path: Option<String>,
        /// Monorepo: scan only files matching path patterns (e.g., ui/**, backend/**)
        #[arg(long, value_name = "PATTERN", num_args = 0..)]
        include_path: Option<Vec<String>>,
        /// Monorepo: scan only specific languages (e.g., java, python, go, rust)
        #[arg(long, value_name = "LANG", num_args = 0.., value_delimiter = ',')]
        languages: Option<Vec<String>>,
        /// Bazel-specific: path to custom .bazelrc file
        #[arg(long, value_name = "PATH")]
        bazel_rc_path: Option<String>,
        /// Bazel-specific: additional flags to pass to bazel commands
        #[arg(long, value_name = "FLAGS")]
        bazel_flags: Option<String>,
        /// Bazel-specific: show internal targets (libraries, not just binaries)
        #[arg(long)]
        bazel_show_internal_targets: bool,
        /// Bazel-specific: path to Go vendor manifest (vendor/modules.txt)
        #[arg(long, value_name = "PATH")]
        bazel_vendor_manifest_path: Option<String>,
        /// Also emit CycloneDX SBOM (for interop)
        #[arg(long)]
        cyclonedx: bool,
        /// Run Semgrep with BazBOM's curated JVM ruleset
        #[arg(long, short = 's')]
        with_semgrep: bool,
        /// Run CodeQL analysis (optional suite: default or security-extended)
        #[arg(long, short = 'c', value_name = "SUITE")]
        with_codeql: Option<CodeqlSuite>,
        /// Generate OpenRewrite recipes (off, dry-run, or pr)
        #[arg(long, value_name = "MODE")]
        autofix: Option<AutofixMode>,
        /// Container SBOM strategy (auto, syft, or bazbom)
        #[arg(long, value_name = "STRATEGY")]
        containers: Option<ContainerStrategy>,
        /// Skip GitHub upload (local dev only)
        #[arg(long)]
        no_upload: bool,
        /// Limit analysis to one module (for PR/changed-path speedups)
        #[arg(long, value_name = "MODULE")]
        target: Option<String>,
        /// Enable incremental analysis (scan only changed code)
        #[arg(long, short = 'i')]
        incremental: bool,
        /// Git base reference for incremental analysis (e.g., main, HEAD~1)
        #[arg(long, short = 'b', value_name = "REF", default_value = "main")]
        base: String,
        /// Show diff of vulnerabilities compared to previous scan
        #[arg(long, short = 'd')]
        diff: bool,
        /// Path to baseline findings for diff comparison
        #[arg(long, value_name = "FILE")]
        baseline: Option<String>,
        /// Enable performance benchmarking and metrics reporting
        #[arg(long)]
        benchmark: bool,
        /// Use ML-enhanced risk scoring for vulnerability prioritization
        #[arg(long, short = 'm')]
        ml_risk: bool,
        /// Auto-create Jira tickets for vulnerabilities (v6.8)
        #[arg(long)]
        jira_create: bool,
        /// Dry-run mode for Jira ticket creation (show what would be created)
        #[arg(long)]
        jira_dry_run: bool,
        /// Auto-create GitHub PRs with fixes (v6.8)
        #[arg(long)]
        github_pr: bool,
        /// Dry-run mode for GitHub PR creation (show what would be created)
        #[arg(long)]
        github_pr_dry_run: bool,
        /// Full auto-remediation: create both Jira tickets and GitHub PRs (v6.8)
        #[arg(long)]
        auto_remediate: bool,
        /// Minimum severity for auto-remediation (CRITICAL, HIGH, MEDIUM, LOW)
        #[arg(long, value_name = "SEVERITY")]
        remediate_min_severity: Option<String>,
        /// Only auto-remediate reachable vulnerabilities
        #[arg(long)]
        remediate_reachable_only: bool,
        /// Limit the number of packages/targets to scan (useful for testing large monorepos)
        #[arg(long, value_name = "N")]
        limit: Option<usize>,
        /// Include CI/CD dependencies (GitHub Actions, GitLab CI, etc.) in SBOM
        #[arg(long)]
        include_cicd: bool,
        /// Fetch SHA256 checksums from package registries (slower but adds integrity verification)
        #[arg(long)]
        fetch_checksums: bool,
        /// Sign generated SBOM with Cosign (requires cosign in PATH)
        #[arg(long)]
        sign_sbom: bool,
    },

    // ========== QUICK COMMAND ALIASES ==========
    /// Quick local dev scan (fast mode, no reachability)
    #[command(
        name = "check",
        after_help = "EXAMPLES:
  # Scan current directory
  bazbom check

  # Scan specific project
  bazbom check ./my-project

WHAT IT DOES:
  • Fast mode (no reachability analysis)
  • No GitHub upload
  • Perfect for quick local validation
  • Runs in < 10 seconds"
    )]
    #[command(name = "check")]
    Check {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// CI-optimized scan (JSON + SARIF output, no GitHub upload)
    #[command(
        name = "ci",
        after_help = "EXAMPLES:
  # In GitHub Actions
  bazbom ci

  # In GitLab CI
  bazbom ci -o ./artifacts

WHAT IT DOES:
  • Auto-detects CI environment
  • Outputs JSON + SARIF formats
  • No GitHub upload (use your CI's upload)
  • Optimized for speed"
    )]
    #[command(name = "ci")]
    Ci {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output directory
        #[arg(long, short = 'o', default_value = ".")]
        out_dir: String,
    },

    /// PR-optimized scan (incremental + diff mode)
    #[command(
        name = "pr",
        after_help = "EXAMPLES:
  # Compare with main branch
  bazbom pr

  # Compare with specific branch
  bazbom pr --base develop

  # With existing baseline
  bazbom pr --baseline main-findings.json

WHAT IT DOES:
  • Incremental analysis (only changed code)
  • Shows diff of vulnerabilities
  • Perfect for PR checks
  • Shows what's new vs fixed"
    )]
    #[command(name = "pr")]
    Pr {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Git base reference for comparison
        #[arg(long, short = 'b', default_value = "main")]
        base: String,
        /// Path to baseline findings
        #[arg(long)]
        baseline: Option<String>,
    },

    /// Full scan with all features (reachability + all formats)
    #[command(
        name = "full",
        after_help = "EXAMPLES:
  # Complete scan with everything
  bazbom full

  # Save to specific directory
  bazbom full -o ./security-scan

  # Limit to 5 packages (useful for testing large monorepos)
  bazbom full --limit 5

WHAT IT DOES:
  • Reachability analysis (reduces noise by 70-90%)
  • Both SPDX and CycloneDX SBOMs
  • ML risk scoring
  • Performance benchmarking
  • Most comprehensive scan available"
    )]
    #[command(name = "full")]
    Full {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output directory
        #[arg(long, short = 'o', default_value = ".")]
        out_dir: String,
        /// Limit the number of packages/targets to scan (useful for testing large monorepos)
        #[arg(long, value_name = "N")]
        limit: Option<usize>,
    },

    /// Super-fast smoke test (< 5 seconds)
    #[command(
        name = "quick",
        after_help = "EXAMPLES:
  # Lightning-fast validation
  bazbom quick

WHAT IT DOES:
  • Ultra-fast scan (< 5 seconds)
  • Basic dependency check
  • Critical vulnerabilities only
  • Perfect for pre-commit hooks"
    )]
    #[command(name = "quick")]
    Quick {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    #[command(after_help = "EXAMPLES:
  bazbom container-scan nginx:latest
      Quick scan with all capabilities enabled (default)

  bazbom container-scan myapp:v1 --preset quick
      Fast CI check without reachability analysis

  bazbom container-scan myapp:v1 --show p0
      Show only P0 (must-fix-today) vulnerabilities

  bazbom container-scan myapp:v1 --compare nginx:latest
      Compare vulnerabilities between two images
")]
    /// Complete container security analysis (SBOM + vulnerability scanning)
    ContainerScan {
        /// Container image to scan (name:tag or path to tar file)
        image: String,
        /// Scan preset: quick (fast CI check), standard (default), full (with reachability), compliance (all reports)
        #[arg(long, value_name = "MODE")]
        preset: Option<String>,
        /// Output directory for results (defaults to ~/Documents/container-scans/<image-name>)
        #[arg(long, value_name = "DIR")]
        output: Option<String>,
        /// Output format (spdx|cyclonedx)
        #[arg(long, default_value = "spdx")]
        format: String,
        /// Save results as baseline for future comparisons
        #[arg(long)]
        baseline: bool,
        /// Compare against saved baseline
        #[arg(long)]
        compare_baseline: bool,
        /// Compare with another image
        #[arg(long, value_name = "IMAGE")]
        compare: Option<String>,
        /// Create GitHub issues for vulnerabilities (requires gh CLI)
        #[arg(long, value_name = "REPO")]
        create_issues: Option<String>,
        /// Interactive TUI for detailed exploration
        #[arg(long)]
        interactive: bool,
        /// Generate executive report
        #[arg(long, value_name = "FILE")]
        report: Option<String>,
        /// Show only specific priority vulnerabilities (p0, p1, p2, fixable, quick-wins)
        #[arg(long, value_name = "FILTER")]
        show: Option<String>,
        /// Enable reachability analysis (extracts container filesystem and analyzes code)
        #[arg(long)]
        with_reachability: bool,
        /// Skip pulling the image (assumes it is already present locally)
        #[arg(long)]
        skip_pull: bool,
        /// Allow unsigned/unknown-provenance images without failing the scan
        #[arg(long)]
        allow_unsigned: bool,
        /// Offline mode: skip networked steps such as docker pull and signature verification
        #[arg(long)]
        offline: bool,
    },
    /// Apply policy checks and output SARIF/JSON verdicts
    Policy {
        #[command(subcommand)]
        action: PolicyCmd,
    },
    /// Show remediation suggestions or apply fixes
    Fix {
        /// Package to analyze/fix (e.g., org.apache.logging.log4j:log4j-core)
        package: Option<String>,
        /// Suggest fixes without applying changes
        #[arg(long)]
        suggest: bool,
        /// Apply fixes automatically
        #[arg(long)]
        apply: bool,
        /// Create a pull request with fixes (requires GitHub authentication)
        #[arg(long)]
        pr: bool,
        /// Interactive mode with smart batch processing
        #[arg(long)]
        interactive: bool,
        /// Show detailed upgrade impact analysis (breaking changes, transitive deps)
        #[arg(long)]
        explain: bool,
        /// Use ML-enhanced prioritization for vulnerability fixes
        #[arg(long)]
        ml_prioritize: bool,
        /// Use LLM-powered fix generation (privacy-first: uses Ollama by default)
        #[arg(long)]
        llm: bool,
        /// LLM provider (ollama, anthropic, or openai). Defaults to ollama for privacy.
        #[arg(long, value_name = "PROVIDER", default_value = "ollama")]
        llm_provider: String,
        /// LLM model (e.g., codellama, gpt-4, claude-3-opus)
        #[arg(long, value_name = "MODEL")]
        llm_model: Option<String>,
    },
    /// Advisory database operations (offline sync)
    Db {
        #[command(subcommand)]
        action: DbCmd,
    },
    /// License compliance operations
    License {
        #[command(subcommand)]
        action: LicenseCmd,
    },
    /// Install git pre-commit hooks for vulnerability scanning
    InstallHooks {
        /// Policy file to use (defaults to bazbom.yml)
        #[arg(long, value_name = "FILE", default_value = "bazbom.yml")]
        policy: String,
        /// Fast scan mode (skip reachability for speed)
        #[arg(long)]
        fast: bool,
    },
    /// Install CI/CD configuration templates
    #[command(
        name = "install",
        after_help = "EXAMPLES:
  # Install GitHub Actions workflow
  bazbom install github

  # Install GitLab CI config
  bazbom install gitlab

  # Install CircleCI config
  bazbom install circleci

  # List all available templates
  bazbom install --list

SUPPORTED PROVIDERS:
  • github    → GitHub Actions (.github/workflows/bazbom.yml)
  • gitlab    → GitLab CI (.gitlab-ci.yml)
  • circleci  → CircleCI (.circleci/config.yml)
  • jenkins   → Jenkins (Jenkinsfile.bazbom)
  • travis    → Travis CI (.travis.yml)"
    )]
    Install {
        /// CI provider (github, gitlab, circleci, jenkins, travis)
        provider: Option<String>,
        /// List available templates
        #[arg(long)]
        list: bool,
    },
    /// Interactive setup wizard for new projects
    Init {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },
    /// Interactive dependency graph explorer (TUI)
    Explore {
        /// Path to SBOM file or findings JSON
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Path to findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
    },
    /// Start web dashboard server
    Dashboard {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
        /// Export static HTML instead of starting server
        #[arg(long, value_name = "FILE")]
        export: Option<String>,
    },
    /// Explain vulnerability details with call chain analysis
    Explain {
        /// CVE identifier (e.g., CVE-2024-1234)
        cve_id: String,
        /// Path to findings JSON file (defaults to latest scan)
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Show full call chain (verbose mode)
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Quick security status overview
    #[command(after_help = "EXAMPLES:
  # Show current security status
  bazbom status

  # Detailed status with full breakdown
  bazbom status --verbose

WHAT IT SHOWS:
  • Security score (0-100)
  • Total vulnerabilities by severity
  • Reachable vulnerabilities count
  • Last scan timestamp
  • Recent trend (improving/worsening)
  • Quick action recommendations")]
    Status {
        /// Show detailed breakdown
        #[arg(long, short = 'v')]
        verbose: bool,
        /// Path to findings file (defaults to latest)
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
    },
    /// Compare security posture between branches or commits
    #[command(after_help = "EXAMPLES:
  # Compare feature branch with main
  bazbom compare main feature/new-api

  # Compare current branch with main
  bazbom compare main

  # Compare two specific commits
  bazbom compare abc123 def456

WHAT IT SHOWS:
  • New vulnerabilities introduced
  • Vulnerabilities fixed
  • Security score delta
  • Risk assessment (better/worse/same)")]
    Compare {
        /// Base branch or commit (e.g., main, HEAD~1, abc123)
        base: String,
        /// Target branch or commit (defaults to current HEAD)
        target: Option<String>,
        /// Show detailed diff
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Continuous monitoring mode (watch for changes and re-scan)
    #[command(after_help = "EXAMPLES:
  # Start watching current directory
  bazbom watch

  # Watch with custom check interval
  bazbom watch --interval 300  # Every 5 minutes

  # Watch and only show critical issues
  bazbom watch --critical-only

WHAT IT DOES:
  • Monitors dependency files for changes
  • Automatically re-scans when changes detected
  • Shows real-time vulnerability alerts
  • Perfect for development workflows")]
    Watch {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Check interval in seconds (default: 60)
        #[arg(long, short = 'i', default_value = "60")]
        interval: u64,
        /// Only show critical vulnerabilities
        #[arg(long)]
        critical_only: bool,
    },
    /// Team coordination and assignment management
    Team {
        #[command(subcommand)]
        action: TeamCmd,
    },
    /// Generate security and compliance reports
    Report {
        #[command(subcommand)]
        action: ReportCmd,
    },
    /// Jira integration commands (v6.8)
    #[command(after_help = "EXAMPLES:
  # Set up Jira integration
  bazbom jira init

  # Create a ticket manually
  bazbom jira create --cve CVE-2024-1234 --package log4j --severity CRITICAL

  # Get ticket details
  bazbom jira get SEC-123

  # Update ticket status
  bazbom jira update SEC-123 --status Done

  # Synchronize tickets with BazBOM
  bazbom jira sync

FEATURES (v6.8):
  • Auto-create tickets during scans
  • Bidirectional sync (Jira ↔ BazBOM)
  • Template-based ticket creation
  • Component routing and assignment
  • Full intelligence integration")]
    Jira {
        #[command(subcommand)]
        action: JiraCmd,
    },
    /// GitHub integration commands (v6.8)
    #[command(after_help = "EXAMPLES:
  # Set up GitHub integration
  bazbom github init

  # Create a PR manually
  bazbom github pr create myorg myrepo --head fix/cve-2024-1234 --cve CVE-2024-1234 --package log4j

  # Get PR details
  bazbom github pr get myorg myrepo 42

  # List PRs
  bazbom github pr list myorg myrepo --state open

FEATURES (v6.8):
  • Auto-create PRs during scans
  • Multi-PR orchestration
  • Template-based PR descriptions
  • Auto-merge capabilities
  • Full intelligence integration")]
    GitHub {
        #[command(subcommand)]
        action: GitHubCmd,
    },
    /// VEX (Vulnerability Exploitability eXchange) management
    #[command(after_help = "EXAMPLES:
  # Create VEX statement for a CVE
  bazbom vex create CVE-2023-12345 --status not_affected \\
    --justification vulnerable_code_not_in_execute_path \\
    --impact \"The vulnerable code path is never reached\"

  # Apply VEX statements to filter findings
  bazbom vex apply --vex-dir vex/statements --findings sca_findings.json

  # List all VEX statements
  bazbom vex list --vex-dir vex/statements

VEX STATUS VALUES:
  not_affected         - Vulnerability does not impact this product
  affected             - Vulnerability impacts this product
  fixed                - Vulnerability was fixed
  under_investigation  - Status unknown, investigating

JUSTIFICATION VALUES (for not_affected):
  component_not_present
  vulnerable_code_not_present
  vulnerable_code_not_in_execute_path
  vulnerable_code_cannot_be_controlled_by_adversary
  inline_mitigations_already_exist")]
    Vex {
        #[command(subcommand)]
        action: VexCmd,
    },
    /// Threat intelligence scanning (typosquatting, dependency confusion, supply chain)
    #[command(after_help = "EXAMPLES:
  # Run full threat detection on project
  bazbom threats scan

  # Check for typosquatting attacks
  bazbom threats scan --typosquatting

  # Check for dependency confusion vulnerabilities
  bazbom threats scan --dep-confusion

  # Run OpenSSF Scorecard analysis
  bazbom threats scan --scorecard

  # Output threats as JSON
  bazbom threats scan --json

THREAT TYPES:
  typosquatting        - Detect packages with names similar to popular packages
  dependency-confusion - Detect internal package name collisions with public registries
  maintainer-takeover  - Detect compromised or transferred maintainer accounts
  supply-chain         - Detect supply chain attack indicators

ECOSYSTEM SUPPORT:
  npm, PyPI, crates.io, RubyGems, Maven Central, Go modules")]
    Threats {
        #[command(subcommand)]
        action: ThreatsCmd,
    },
    /// Notification configuration (Slack, Teams, Email, GitHub)
    #[command(after_help = "EXAMPLES:
  # Configure Slack notifications
  bazbom notify configure --slack-webhook https://hooks.slack.com/...

  # Configure email notifications
  bazbom notify configure --email security@example.com --smtp-host smtp.example.com

  # Test notification delivery
  bazbom notify test --channel slack

  # View notification history
  bazbom notify history")]
    Notify {
        #[command(subcommand)]
        action: NotifyCmd,
    },
    /// ML-based anomaly detection for supply chain risks
    #[command(after_help = "EXAMPLES:
  # Run anomaly detection on current project
  bazbom anomaly scan

  # Train model on historical scan data
  bazbom anomaly train --from-dir scan-history/

  # Generate anomaly report
  bazbom anomaly report --output anomaly-report.html

DETECTION TYPES:
  - Unusual transitive dependency counts
  - Low maintainer reputation scores
  - Suspicious release patterns
  - Low package popularity (potential typosquat)")]
    Anomaly {
        #[command(subcommand)]
        action: AnomalyCmd,
    },
    /// LSP server for IDE integration (VS Code, IntelliJ, etc.)
    #[command(after_help = "EXAMPLES:
  # Start LSP server (stdio mode)
  bazbom lsp

  # VS Code: Add to settings.json:
  \"bazbom.lsp.path\": \"bazbom-lsp\"

  # IntelliJ: Install via LSP4IJ plugin

FEATURES:
  - Real-time vulnerability diagnostics
  - Quick fixes to upgrade vulnerable dependencies
  - Hover info with CVE details
  - Supports: pom.xml, build.gradle, BUILD.bazel

SETUP:
  The LSP server is a separate binary. Install with:
  cargo install bazbom-lsp")]
    Lsp {
        /// Use stdio transport (default)
        #[arg(long, default_value = "true")]
        stdio: bool,
    },
    /// Authentication and RBAC management
    #[command(after_help = "EXAMPLES:
  # Initialize authentication system
  bazbom auth init

  # Create a new user
  bazbom auth user add alice@example.com --role admin

  # List users
  bazbom auth user list

  # Create API token
  bazbom auth token create --name ci-pipeline --scope read

  # Revoke API token
  bazbom auth token revoke <TOKEN_ID>

ROLES:
  admin           - Full access to all features
  security-lead   - Manage security policies, assign work
  developer       - Read/write SBOMs, read vulnerabilities
  user            - Read-only access
  ci              - Automated scanning permissions

PERMISSIONS:
  read-sbom, write-sbom, delete-sbom
  read-vulnerabilities, manage-policy
  manage-users, view-audit-log, manage-tokens")]
    Auth {
        #[command(subcommand)]
        action: AuthCmd,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CodeqlSuite {
    Default,
    SecurityExtended,
}

impl CodeqlSuite {
    pub fn as_str(&self) -> &str {
        match self {
            CodeqlSuite::Default => "default",
            CodeqlSuite::SecurityExtended => "security-extended",
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum AutofixMode {
    Off,
    DryRun,
    Pr,
}

impl AutofixMode {
    pub fn as_str(&self) -> &str {
        match self {
            AutofixMode::Off => "off",
            AutofixMode::DryRun => "dry-run",
            AutofixMode::Pr => "pr",
        }
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ContainerStrategy {
    Auto,
    Syft,
    Bazbom,
}

impl ContainerStrategy {
    pub fn as_str(&self) -> &str {
        match self {
            ContainerStrategy::Auto => "auto",
            ContainerStrategy::Syft => "syft",
            ContainerStrategy::Bazbom => "bazbom",
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum PolicyCmd {
    /// Run policy checks
    Check {},
    /// Initialize a policy template
    Init {
        /// List available policy templates
        #[arg(long)]
        list: bool,
        /// Template ID to initialize (e.g., pci-dss, hipaa, fedramp-moderate, soc2, corporate-permissive)
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,
        /// Output path (defaults to current directory)
        #[arg(long, value_name = "PATH", default_value = ".")]
        output: String,
    },
    /// Validate a policy file
    Validate {
        /// Path to policy file to validate
        #[arg(default_value = "bazbom.yml")]
        policy_file: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbCmd {
    /// Sync local advisory mirrors for offline use
    Sync {},
}

#[derive(Subcommand, Debug)]
pub enum LicenseCmd {
    /// Generate license obligations report
    Obligations {
        /// SBOM file to analyze (SPDX or CycloneDX)
        #[arg(value_name = "FILE")]
        sbom_file: Option<String>,
    },
    /// Check license compatibility
    Compatibility {
        /// Project license (e.g., MIT, Apache-2.0)
        #[arg(long, value_name = "LICENSE")]
        project_license: String,
        /// SBOM file to analyze
        #[arg(value_name = "FILE")]
        sbom_file: Option<String>,
    },
    /// Detect copyleft contamination
    Contamination {
        /// SBOM file to analyze
        #[arg(value_name = "FILE")]
        sbom_file: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum TeamCmd {
    /// Assign a vulnerability to a team member
    Assign {
        /// CVE identifier (e.g., CVE-2021-44228)
        cve: String,
        /// Team member email
        #[arg(long, value_name = "EMAIL")]
        to: String,
    },
    /// List all vulnerability assignments
    List {},
    /// Show assignments for current user
    Mine {},
    /// Export audit log
    AuditLog {
        /// Export format (json or csv)
        #[arg(long, default_value = "json")]
        format: String,
        /// Output file path
        #[arg(long, value_name = "FILE")]
        output: Option<String>,
    },
    /// Configure team settings
    Config {
        /// Team name
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
        /// Add team member email
        #[arg(long, value_name = "EMAIL")]
        add_member: Option<String>,
        /// Remove team member email
        #[arg(long, value_name = "EMAIL")]
        remove_member: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReportCmd {
    /// Generate executive summary report (1-page)
    Executive {
        /// SBOM file to analyze (SPDX or CycloneDX)
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Output file path
        #[arg(long, value_name = "FILE", default_value = "executive-report.html")]
        output: String,
    },
    /// Generate compliance report for specific framework
    Compliance {
        /// Compliance framework
        #[arg(value_enum)]
        framework: ComplianceFrameworkArg,
        /// SBOM file to analyze
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Output file path
        #[arg(long, value_name = "FILE", default_value = "compliance-report.html")]
        output: String,
    },
    /// Generate detailed developer report
    Developer {
        /// SBOM file to analyze
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Output file path
        #[arg(long, value_name = "FILE", default_value = "developer-report.html")]
        output: String,
    },
    /// Generate historical trend report
    Trend {
        /// SBOM file to analyze
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Output file path
        #[arg(long, value_name = "FILE", default_value = "trend-report.html")]
        output: String,
    },
    /// Generate all report types
    All {
        /// SBOM file to analyze
        #[arg(long, value_name = "FILE")]
        sbom: Option<String>,
        /// Findings JSON file
        #[arg(long, value_name = "FILE")]
        findings: Option<String>,
        /// Output directory
        #[arg(long, value_name = "DIR", default_value = "reports")]
        output_dir: String,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ComplianceFrameworkArg {
    PciDss,
    Hipaa,
    FedRampModerate,
    Soc2,
    Gdpr,
    Iso27001,
    NistCsf,
}

#[derive(Subcommand, Debug)]
pub enum JiraCmd {
    /// Initialize Jira integration (interactive setup)
    Init,
    /// Create a Jira ticket
    Create {
        /// Path to vulnerability findings file (JSON)
        #[arg(long, value_name = "FILE")]
        file: Option<String>,
        /// CVE ID (for manual creation)
        #[arg(long, value_name = "CVE")]
        cve: Option<String>,
        /// Package name (for manual creation)
        #[arg(long, value_name = "PACKAGE")]
        package: Option<String>,
        /// Severity (for manual creation: CRITICAL, HIGH, MEDIUM, LOW)
        #[arg(long, value_name = "SEVERITY")]
        severity: Option<String>,
    },
    /// Get Jira ticket details
    Get {
        /// Jira issue key (e.g., SEC-123)
        key: String,
    },
    /// Update a Jira ticket
    Update {
        /// Jira issue key
        key: String,
        /// New status
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        /// New assignee
        #[arg(long, value_name = "USER")]
        assignee: Option<String>,
    },
    /// Synchronize Jira tickets with BazBOM
    Sync,
}

#[derive(Subcommand, Debug)]
pub enum GitHubCmd {
    /// GitHub PR subcommands
    #[command(subcommand)]
    Pr(GitHubPrCmd),
    /// Initialize GitHub integration (interactive setup)
    Init,
}

#[derive(Subcommand, Debug)]
pub enum GitHubPrCmd {
    /// Create a pull request
    Create {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// Head branch with fixes
        #[arg(long, value_name = "BRANCH")]
        head: String,
        /// Base branch (default: main)
        #[arg(long, value_name = "BRANCH")]
        base: Option<String>,
        /// PR title (optional)
        #[arg(long, value_name = "TITLE")]
        title: Option<String>,
        /// CVE ID (for metadata)
        #[arg(long, value_name = "CVE")]
        cve: Option<String>,
        /// Package name (for metadata)
        #[arg(long, value_name = "PACKAGE")]
        package: Option<String>,
    },
    /// Get PR details
    Get {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// PR number
        number: u64,
    },
    /// List PRs
    List {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// PR state (open, closed, all)
        #[arg(long, value_name = "STATE")]
        state: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum VexCmd {
    /// Create a VEX statement
    Create {
        /// CVE ID (e.g., CVE-2023-12345)
        cve: String,
        /// VEX status
        #[arg(long, value_name = "STATUS")]
        status: String,
        /// Justification (for not_affected status)
        #[arg(long, value_name = "JUSTIFICATION")]
        justification: Option<String>,
        /// Impact statement explaining why the status applies
        #[arg(long, value_name = "TEXT")]
        impact: Option<String>,
        /// Package PURL (optional, for package-specific VEX)
        #[arg(long, value_name = "PURL")]
        package: Option<String>,
        /// Author email
        #[arg(long, value_name = "EMAIL", default_value = "security@example.com")]
        author: String,
        /// Output file path
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
    },
    /// Apply VEX statements to filter findings
    Apply {
        /// Directory containing VEX statements
        #[arg(long, value_name = "DIR")]
        vex_dir: String,
        /// Input findings file (JSON)
        #[arg(long, value_name = "FILE")]
        findings: String,
        /// Output file for filtered findings
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
    },
    /// List VEX statements
    List {
        /// Directory containing VEX statements
        #[arg(long, value_name = "DIR", default_value = "vex/statements")]
        vex_dir: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ThreatsCmd {
    /// Run threat detection scan
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Enable typosquatting detection
        #[arg(long)]
        typosquatting: bool,
        /// Enable dependency confusion detection
        #[arg(long)]
        dep_confusion: bool,
        /// Enable maintainer takeover detection
        #[arg(long)]
        maintainer_takeover: bool,
        /// Run OpenSSF Scorecard analysis
        #[arg(long)]
        scorecard: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Output file path
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
        /// Minimum threat level to report (critical, high, medium, low)
        #[arg(long, value_name = "LEVEL", default_value = "low")]
        min_level: String,
    },
    /// Configure threat feeds
    Configure {
        /// Add custom threat feed URL
        #[arg(long, value_name = "URL")]
        add_feed: Option<String>,
        /// Remove threat feed
        #[arg(long, value_name = "NAME")]
        remove_feed: Option<String>,
        /// List configured feeds
        #[arg(long)]
        list: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum NotifyCmd {
    /// Configure notification channels
    Configure {
        /// Slack webhook URL
        #[arg(long, value_name = "URL")]
        slack_webhook: Option<String>,
        /// Microsoft Teams webhook URL
        #[arg(long, value_name = "URL")]
        teams_webhook: Option<String>,
        /// Email address for notifications
        #[arg(long, value_name = "EMAIL")]
        email: Option<String>,
        /// SMTP host for email
        #[arg(long, value_name = "HOST")]
        smtp_host: Option<String>,
        /// GitHub repository for issue creation (owner/repo)
        #[arg(long, value_name = "REPO")]
        github_repo: Option<String>,
        /// Minimum severity to notify (critical, high, medium, low)
        #[arg(long, value_name = "LEVEL", default_value = "high")]
        min_severity: String,
    },
    /// Test notification delivery
    Test {
        /// Channel to test (slack, teams, email, github)
        #[arg(long, value_name = "CHANNEL")]
        channel: String,
    },
    /// View notification history
    History {
        /// Number of entries to show
        #[arg(long, value_name = "N", default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum AnomalyCmd {
    /// Run anomaly detection scan
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Output file path
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
    },
    /// Train anomaly detection model on historical data
    Train {
        /// Directory containing historical scan data
        #[arg(long, value_name = "DIR")]
        from_dir: String,
        /// Output model file
        #[arg(
            long,
            short = 'o',
            value_name = "FILE",
            default_value = ".bazbom/anomaly-model.json"
        )]
        output: String,
    },
    /// Generate anomaly detection report
    Report {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output file path
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthCmd {
    /// Initialize authentication system
    Init {},
    /// User management
    #[command(subcommand)]
    User(AuthUserCmd),
    /// API token management
    #[command(subcommand)]
    Token(AuthTokenCmd),
    /// View audit log
    AuditLog {
        /// Number of entries to show
        #[arg(long, value_name = "N", default_value = "50")]
        limit: usize,
        /// Filter by event type
        #[arg(long, value_name = "TYPE")]
        event_type: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthUserCmd {
    /// Add a new user
    Add {
        /// User email
        email: String,
        /// User role (admin, security-lead, developer, user, ci)
        #[arg(long, value_name = "ROLE", default_value = "user")]
        role: String,
    },
    /// List all users
    List {},
    /// Remove a user
    Remove {
        /// User email
        email: String,
    },
    /// Update user role
    SetRole {
        /// User email
        email: String,
        /// New role
        #[arg(long, value_name = "ROLE")]
        role: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthTokenCmd {
    /// Create a new API token
    Create {
        /// Token name/description
        #[arg(long, value_name = "NAME")]
        name: String,
        /// Token scope (read, write, admin)
        #[arg(long, value_name = "SCOPE", default_value = "read")]
        scope: String,
        /// Expiration in days (0 for no expiration)
        #[arg(long, value_name = "DAYS", default_value = "365")]
        expires: u32,
    },
    /// List all tokens
    List {},
    /// Revoke a token
    Revoke {
        /// Token ID
        token_id: String,
    },
}
