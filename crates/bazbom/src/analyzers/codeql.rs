use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use crate::toolchain::{ToolCache, ToolManifestLoader};
use anyhow::{Context as _, Result};
use bazbom_formats::sarif::SarifReport;
use std::path::PathBuf;

pub struct CodeqlAnalyzer {
    suite: String,
}

impl CodeqlAnalyzer {
    pub fn new(suite: Option<String>) -> Self {
        Self {
            suite: suite.unwrap_or_else(|| "default".to_string()),
        }
    }

    fn detect_build_system(workspace: &PathBuf) -> Option<BuildSystem> {
        if workspace.join("pom.xml").exists() {
            Some(BuildSystem::Maven)
        } else if workspace.join("build.gradle").exists() || workspace.join("build.gradle.kts").exists() {
            Some(BuildSystem::Gradle)
        } else if workspace.join("BUILD").exists() || workspace.join("BUILD.bazel").exists() {
            Some(BuildSystem::Bazel)
        } else {
            None
        }
    }

    fn get_codeql_binary(&self, ctx: &Context) -> Result<PathBuf> {
        // First check if codeql is in PATH
        if std::process::Command::new("codeql")
            .arg("version")
            .output()
            .is_ok()
        {
            return Ok(PathBuf::from("codeql"));
        }

        // Try to use managed installation from tool cache
        println!("[bazbom] CodeQL not found in PATH, checking tool cache...");
        let loader = ToolManifestLoader::load()?;
        let desc = loader.get_descriptor("codeql")?;
        let cache = ToolCache::new(ctx.tool_cache.clone());
        let path = cache.ensure(&desc)?;
        println!("[bazbom] using managed CodeQL from cache");
        Ok(path)
    }

    fn create_database(&self, codeql_bin: &PathBuf, ctx: &Context, build_system: &BuildSystem) -> Result<PathBuf> {
        let db_dir = ctx.out_dir.join("codeql-db");
        
        // Clean up old database if it exists
        if db_dir.exists() {
            std::fs::remove_dir_all(&db_dir)
                .context("failed to remove old CodeQL database")?;
        }

        println!("[bazbom] creating CodeQL database for {:?}...", build_system);

        let build_command = match build_system {
            BuildSystem::Maven => "mvn clean compile -DskipTests",
            BuildSystem::Gradle => "./gradlew clean compileJava -x test",
            BuildSystem::Bazel => "bazel build //...",
        };

        // Run codeql database create
        let db_path_str = db_dir.to_str().unwrap();
        let command_arg = format!("--command={}", build_command);
        
        let args = vec![
            "database",
            "create",
            db_path_str,
            "--language=java",
            &command_arg,
            "--overwrite",
        ];

        println!("[bazbom] running: codeql {}", args.join(" "));
        
        let status = std::process::Command::new(codeql_bin)
            .args(&args)
            .current_dir(&ctx.workspace)
            .status()
            .context("failed to run codeql database create")?;

        if !status.success() {
            anyhow::bail!("codeql database create failed with status: {}", status);
        }

        println!("[bazbom] CodeQL database created successfully");
        Ok(db_dir)
    }

    fn analyze_database(&self, codeql_bin: &PathBuf, ctx: &Context, db_dir: &PathBuf) -> Result<SarifReport> {
        let output_path = ctx.findings_dir.join("codeql.sarif");

        // Map suite to CodeQL query pack
        let query_pack = match self.suite.as_str() {
            "security-extended" => "codeql/java-queries:codeql-suites/java-security-extended.qls",
            _ => "codeql/java-queries:codeql-suites/java-security-and-quality.qls",
        };

        println!("[bazbom] analyzing database with query pack: {}", query_pack);

        let db_path_str = db_dir.to_str().unwrap();
        let output_path_str = output_path.to_str().unwrap();
        let format_arg = String::from("--format=sarif-latest");
        let output_arg = format!("--output={}", output_path_str);
        
        let args = vec![
            "database",
            "analyze",
            db_path_str,
            query_pack,
            &format_arg,
            &output_arg,
            "--sarif-category=bazbom-codeql",
        ];

        println!("[bazbom] running: codeql {}", args.join(" "));

        let status = std::process::Command::new(codeql_bin)
            .args(&args)
            .current_dir(&ctx.workspace)
            .status()
            .context("failed to run codeql database analyze")?;

        if !status.success() {
            anyhow::bail!("codeql database analyze failed with status: {}", status);
        }

        // Read the generated SARIF file
        let sarif_content = std::fs::read_to_string(&output_path)
            .context("failed to read CodeQL SARIF output")?;
        let report: SarifReport = serde_json::from_str(&sarif_content)
            .context("failed to parse CodeQL SARIF output")?;

        println!("[bazbom] CodeQL analysis complete, wrote findings to {:?}", output_path);

        Ok(report)
    }
}

#[derive(Debug)]
enum BuildSystem {
    Maven,
    Gradle,
    Bazel,
}

impl Analyzer for CodeqlAnalyzer {
    fn id(&self) -> &'static str {
        "codeql"
    }

    fn enabled(&self, cfg: &Config, cli_override: bool) -> bool {
        if cli_override {
            return true;
        }
        cfg.analysis
            .codeql
            .as_ref()
            .and_then(|c| c.enabled)
            .unwrap_or(false)
    }

    fn run(&self, ctx: &Context) -> Result<SarifReport> {
        println!("[bazbom] running CodeQL analysis (suite: {})...", self.suite);

        // Detect build system
        let build_system = match Self::detect_build_system(&ctx.workspace) {
            Some(bs) => bs,
            None => {
                println!("[bazbom] no supported build system detected (Maven, Gradle, or Bazel)");
                println!("[bazbom] skipping CodeQL analysis");
                let report = SarifReport::new("CodeQL", "no-build-system");
                let output_path = ctx.findings_dir.join("codeql.sarif");
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                return Ok(report);
            }
        };

        // Get CodeQL binary
        let codeql_bin = match self.get_codeql_binary(ctx) {
            Ok(bin) => bin,
            Err(e) => {
                println!("[bazbom] failed to get CodeQL binary: {}", e);
                println!("[bazbom] Install CodeQL CLI: https://github.com/github/codeql-cli-binaries/releases");
                println!("[bazbom] Or let BazBOM download it automatically on next run");
                let report = SarifReport::new("CodeQL", "not-installed");
                let output_path = ctx.findings_dir.join("codeql.sarif");
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                return Ok(report);
            }
        };

        // Create database
        let db_dir = match self.create_database(&codeql_bin, ctx, &build_system) {
            Ok(dir) => dir,
            Err(e) => {
                println!("[bazbom] failed to create CodeQL database: {}", e);
                println!("[bazbom] this is expected if the project doesn't compile cleanly");
                let report = SarifReport::new("CodeQL", "database-creation-failed");
                let output_path = ctx.findings_dir.join("codeql.sarif");
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                return Ok(report);
            }
        };

        // Analyze database
        match self.analyze_database(&codeql_bin, ctx, &db_dir) {
            Ok(report) => Ok(report),
            Err(e) => {
                println!("[bazbom] failed to analyze CodeQL database: {}", e);
                let report = SarifReport::new("CodeQL", "analysis-failed");
                let output_path = ctx.findings_dir.join("codeql.sarif");
                let json = serde_json::to_string_pretty(&report)?;
                std::fs::write(&output_path, json)?;
                Ok(report)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codeql_analyzer_enabled() {
        let analyzer = CodeqlAnalyzer::new(None);
        
        // Test CLI override
        let config = Config::default();
        assert!(analyzer.enabled(&config, true));
        
        // Test config enabled
        let mut config = Config::default();
        config.analysis.codeql = Some(crate::config::CodeqlConfig {
            enabled: Some(true),
            suite: None,
        });
        assert!(analyzer.enabled(&config, false));
        
        // Test disabled by default
        let config = Config::default();
        assert!(!analyzer.enabled(&config, false));
    }

    #[test]
    fn test_codeql_analyzer_id() {
        let analyzer = CodeqlAnalyzer::new(None);
        assert_eq!(analyzer.id(), "codeql");
    }

    #[test]
    fn test_codeql_analyzer_suite() {
        let analyzer1 = CodeqlAnalyzer::new(None);
        assert_eq!(analyzer1.suite, "default");
        
        let analyzer2 = CodeqlAnalyzer::new(Some("security-extended".to_string()));
        assert_eq!(analyzer2.suite, "security-extended");
    }
}
