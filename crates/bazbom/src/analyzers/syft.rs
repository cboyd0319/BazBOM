use crate::config::Config;
use crate::context::Context;
use crate::toolchain::{run_tool, ToolCache, ToolManifestLoader};
use anyhow::Result;
use std::path::PathBuf;

pub struct SyftRunner {
    strategy: ContainerStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerStrategy {
    Auto,
    Syft,
    Bazbom,
}

impl SyftRunner {
    pub fn new(strategy: ContainerStrategy) -> Self {
        Self { strategy }
    }

    /// Generate container SBOM using Syft
    /// Returns the path to the generated SBOM file
    pub fn generate_container_sbom(&self, ctx: &Context, image_or_path: &str) -> Result<PathBuf> {
        match self.strategy {
            ContainerStrategy::Syft => self.run_syft(ctx, image_or_path),
            ContainerStrategy::Auto => {
                // Try BazBOM first, fallback to Syft if not ready
                println!(
                    "[bazbom] container strategy=auto: using Syft (BazBOM implementation pending)"
                );
                self.run_syft(ctx, image_or_path)
            }
            ContainerStrategy::Bazbom => {
                println!("[bazbom] container strategy=bazbom: not yet implemented");
                Err(anyhow::anyhow!(
                    "BazBOM container SBOM not yet implemented; use --containers=syft"
                ))
            }
        }
    }

    fn run_syft(&self, ctx: &Context, image_or_path: &str) -> Result<PathBuf> {
        println!(
            "[bazbom] generating container SBOM with Syft for: {}",
            image_or_path
        );

        // Check if syft is available in PATH (user installed)
        let syft_result = std::process::Command::new("syft").arg("--version").output();

        let syft_bin = if syft_result.is_ok() {
            println!("[bazbom] using system-installed Syft");
            PathBuf::from("syft")
        } else {
            println!("[bazbom] Syft not found in PATH, checking tool cache...");

            match ToolManifestLoader::load() {
                Ok(loader) => {
                    match loader.get_descriptor("syft") {
                        Ok(desc) => {
                            let cache = ToolCache::new(ctx.tool_cache.clone());
                            match cache.ensure(&desc) {
                                Ok(path) => {
                                    println!("[bazbom] using managed Syft from cache");
                                    path
                                }
                                Err(e) => {
                                    println!("[bazbom] failed to download Syft: {}", e);
                                    println!("[bazbom] Install Syft: https://github.com/anchore/syft#installation");
                                    return Err(anyhow::anyhow!("Syft not available"));
                                }
                            }
                        }
                        Err(e) => {
                            println!("[bazbom] tool manifest error: {}", e);
                            println!("[bazbom] Install Syft: https://github.com/anchore/syft#installation");
                            return Err(anyhow::anyhow!("Syft not available"));
                        }
                    }
                }
                Err(e) => {
                    println!("[bazbom] failed to load tool manifest: {}", e);
                    return Err(anyhow::anyhow!("Syft not available"));
                }
            }
        };

        // Generate SPDX SBOM using Syft
        let output_path = ctx.sbom_dir.join("container.spdx.json");
        let output_path_str = output_path.to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in output path"))?;

        let args = vec![
            image_or_path,
            "-o",
            "spdx-json",
            "--file",
            output_path_str,
        ];

        let output = run_tool(&syft_bin, &args, &ctx.workspace, 300)?;

        if output.exit_code != 0 {
            eprintln!("[bazbom] Syft stderr: {}", output.stderr);
            return Err(anyhow::anyhow!(
                "Syft failed with exit code {}",
                output.exit_code
            ));
        }

        println!("[bazbom] wrote container SBOM to {:?}", output_path);
        Ok(output_path)
    }

    /// Strategy selector from config and CLI
    pub fn resolve_strategy(config: &Config, cli_override: Option<&str>) -> ContainerStrategy {
        if let Some(cli_val) = cli_override {
            return match cli_val.to_lowercase().as_str() {
                "syft" => ContainerStrategy::Syft,
                "bazbom" => ContainerStrategy::Bazbom,
                _ => ContainerStrategy::Auto,
            };
        }

        // Check config
        if let Some(ref strategy) = config.containers.strategy {
            match strategy.to_lowercase().as_str() {
                "syft" => return ContainerStrategy::Syft,
                "bazbom" => return ContainerStrategy::Bazbom,
                _ => {}
            }
        }

        // Default to Auto
        ContainerStrategy::Auto
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_strategy_cli() {
        let config = Config::default();

        let strategy = SyftRunner::resolve_strategy(&config, Some("syft"));
        assert_eq!(strategy, ContainerStrategy::Syft);

        let strategy = SyftRunner::resolve_strategy(&config, Some("bazbom"));
        assert_eq!(strategy, ContainerStrategy::Bazbom);

        let strategy = SyftRunner::resolve_strategy(&config, Some("auto"));
        assert_eq!(strategy, ContainerStrategy::Auto);
    }

    #[test]
    fn test_resolve_strategy_config() {
        let mut config = Config::default();
        config.containers.strategy = Some("syft".to_string());

        let strategy = SyftRunner::resolve_strategy(&config, None);
        assert_eq!(strategy, ContainerStrategy::Syft);
    }

    #[test]
    fn test_resolve_strategy_default() {
        let config = Config::default();
        let strategy = SyftRunner::resolve_strategy(&config, None);
        assert_eq!(strategy, ContainerStrategy::Auto);
    }

    #[test]
    fn test_syft_runner_creation() {
        let runner = SyftRunner::new(ContainerStrategy::Syft);
        assert_eq!(runner.strategy, ContainerStrategy::Syft);
    }

    #[test]
    fn test_bazbom_strategy_not_implemented() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");
        let ctx = Context::new(workspace, out_dir)?;

        let runner = SyftRunner::new(ContainerStrategy::Bazbom);
        let result = runner.generate_container_sbom(&ctx, "test-image");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not yet implemented"));

        Ok(())
    }
}
