use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bazbom", version, about = "JVM SBOM, SCA, and dependency graph tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a project and generate SBOM + findings
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Enable reachability analysis (OPAL)
        #[arg(long)]
        reachability: bool,
        /// Fast mode: skip reachability analysis for speed (<10s scans)
        #[arg(long)]
        fast: bool,
        /// Output format (spdx|cyclonedx)
        #[arg(long, default_value = "spdx")]
        format: String,
        /// Output directory (defaults to current directory)
        #[arg(long, value_name = "DIR", default_value = ".")]
        out_dir: String,
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
        /// Also emit CycloneDX SBOM (for interop)
        #[arg(long)]
        cyclonedx: bool,
        /// Run Semgrep with BazBOM's curated JVM ruleset
        #[arg(long)]
        with_semgrep: bool,
        /// Run CodeQL analysis (optional suite: default or security-extended)
        #[arg(long, value_name = "SUITE")]
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
        #[arg(long)]
        incremental: bool,
        /// Git base reference for incremental analysis (e.g., main, HEAD~1)
        #[arg(long, value_name = "REF", default_value = "main")]
        base: String,
        /// Enable performance benchmarking and metrics reporting
        #[arg(long)]
        benchmark: bool,
        /// Use ML-enhanced risk scoring for vulnerability prioritization
        #[arg(long)]
        ml_risk: bool,
    },
    /// Complete container security analysis (SBOM + vulnerability scanning)
    ContainerScan {
        /// Container image to scan (name:tag or path to tar file)
        image: String,
        /// Output directory for results
        #[arg(long, value_name = "DIR", default_value = "./container-scan")]
        output: String,
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
