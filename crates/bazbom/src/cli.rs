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
    },
    /// Apply policy checks and output SARIF/JSON verdicts
    Policy {
        #[command(subcommand)]
        action: PolicyCmd,
    },
    /// Show remediation suggestions or apply fixes
    Fix {
        /// Suggest fixes without applying changes
        #[arg(long)]
        suggest: bool,
        /// Apply fixes and open PRs
        #[arg(long)]
        apply: bool,
    },
    /// Advisory database operations (offline sync)
    Db {
        #[command(subcommand)]
        action: DbCmd,
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
}

#[derive(Subcommand, Debug)]
pub enum DbCmd {
    /// Sync local advisory mirrors for offline use
    Sync {},
}
