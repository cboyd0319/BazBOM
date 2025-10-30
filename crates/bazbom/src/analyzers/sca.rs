use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use anyhow::{Context as _, Result};
use bazbom_advisories::{db_sync, load_epss_scores, load_kev_catalog, Priority, Vulnerability};
use bazbom_formats::sarif::{
    ArtifactLocation, Location, Message, PhysicalLocation, Result as SarifResult, Rule,
    SarifReport, MessageString, Configuration,
};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ScaAnalyzer;

impl ScaAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn ensure_advisory_database(&self, ctx: &Context) -> Result<PathBuf> {
        let cache_dir = ctx.workspace.join(".bazbom").join("advisories");
        
        // Check if we have a recent database (less than 24 hours old)
        let manifest_path = cache_dir.join("manifest.json");
        let needs_sync = if manifest_path.exists() {
            let metadata = std::fs::metadata(&manifest_path)?;
            let modified = metadata.modified()?;
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or(std::time::Duration::from_secs(86400 * 2));
            age.as_secs() > 86400 // More than 24 hours old
        } else {
            true
        };

        if needs_sync {
            println!("[bazbom] syncing advisory database...");
            db_sync(&cache_dir, false).context("failed to sync advisory database")?;
            println!("[bazbom] advisory database synced");
        } else {
            println!("[bazbom] using cached advisory database");
        }

        Ok(cache_dir)
    }

    fn load_sbom_components(&self, ctx: &Context) -> Result<Vec<Component>> {
        // Try to load SPDX SBOM from sbom directory
        let spdx_path = ctx.sbom_dir.join("spdx.json");
        if !spdx_path.exists() {
            println!("[bazbom] no SBOM found at {:?}, running SCA with empty component list", spdx_path);
            return Ok(Vec::new());
        }

        // For now, return empty vector as we need to parse SPDX
        // In a full implementation, this would parse the SPDX file
        println!("[bazbom] SBOM parsing not yet implemented, returning empty component list");
        Ok(Vec::new())
    }

    fn match_vulnerabilities(
        &self,
        _components: &[Component],
        advisory_dir: &PathBuf,
    ) -> Result<Vec<VulnerabilityMatch>> {
        // Load EPSS scores
        let _epss_scores = load_epss_scores(advisory_dir)
            .unwrap_or_else(|_| HashMap::new());

        // Load KEV catalog
        let _kev_entries = load_kev_catalog(advisory_dir)
            .unwrap_or_else(|_| HashMap::new());

        // For now, return empty matches as we need OSV/NVD/GHSA parsers
        // In a full implementation, this would:
        // 1. Parse OSV/NVD/GHSA advisories
        // 2. Match components against vulnerabilities
        // 3. Enrich with EPSS and KEV data
        // 4. Calculate priorities
        
        println!("[bazbom] vulnerability matching not yet fully implemented");
        Ok(Vec::new())
    }

    fn create_sarif_results(&self, matches: Vec<VulnerabilityMatch>) -> Vec<SarifResult> {
        matches
            .into_iter()
            .map(|m| {
                let level = match m.priority {
                    Some(Priority::P0) => "error",
                    Some(Priority::P1) => "error",
                    Some(Priority::P2) => "warning",
                    Some(Priority::P3) => "note",
                    Some(Priority::P4) => "note",
                    None => "warning",
                };

                let mut message_parts = vec![
                    format!("Vulnerability {} found in {}", m.vulnerability_id, m.component_name),
                ];

                if let Some(summary) = &m.summary {
                    message_parts.push(summary.clone());
                }

                if let Some(epss) = m.epss_score {
                    message_parts.push(format!("EPSS: {:.2}%", epss * 100.0));
                }

                if m.in_kev {
                    message_parts.push("⚠️  Listed in CISA KEV (Known Exploited Vulnerabilities)".to_string());
                }

                let mut properties = serde_json::json!({
                    "vulnerability_id": m.vulnerability_id,
                    "component": m.component_name,
                    "version": m.component_version,
                });

                if let Some(epss) = m.epss_score {
                    properties["epss_score"] = serde_json::json!(epss);
                }

                if m.in_kev {
                    properties["cisa_kev"] = serde_json::json!(true);
                }

                if let Some(priority) = m.priority {
                    properties["priority"] = serde_json::json!(format!("{:?}", priority));
                }

                SarifResult {
                    rule_id: m.vulnerability_id.clone(),
                    level: level.to_string(),
                    message: Message {
                        text: message_parts.join(". "),
                    },
                    locations: Some(vec![Location {
                        physical_location: PhysicalLocation {
                            artifact_location: ArtifactLocation {
                                uri: m.location.clone(),
                            },
                        },
                    }]),
                    properties: Some(serde_json::to_value(properties).ok().unwrap_or_default()),
                }
            })
            .collect()
    }
}

#[derive(Debug)]
#[allow(dead_code)]  // Will be used when SBOM parsing is implemented
struct Component {
    name: String,
    version: String,
    ecosystem: String,
    location: String,
}

#[derive(Debug)]
struct VulnerabilityMatch {
    vulnerability_id: String,
    component_name: String,
    component_version: String,
    summary: Option<String>,
    epss_score: Option<f64>,
    in_kev: bool,
    priority: Option<Priority>,
    location: String,
}

impl Analyzer for ScaAnalyzer {
    fn id(&self) -> &'static str {
        "bazbom-sca"
    }

    fn enabled(&self, _cfg: &Config, _cli_override: bool) -> bool {
        // SCA is always enabled
        true
    }

    fn run(&self, ctx: &Context) -> Result<SarifReport> {
        println!("[bazbom] running SCA analysis...");

        // Ensure we have advisory database
        let advisory_dir = match self.ensure_advisory_database(ctx) {
            Ok(dir) => dir,
            Err(e) => {
                println!("[bazbom] warning: failed to sync advisory database: {}", e);
                println!("[bazbom] continuing with potentially stale data");
                ctx.workspace.join(".bazbom").join("advisories")
            }
        };

        // Load SBOM components
        let components = self.load_sbom_components(ctx)
            .context("failed to load SBOM components")?;

        println!("[bazbom] loaded {} components from SBOM", components.len());

        // Match vulnerabilities
        let matches = self.match_vulnerabilities(&components, &advisory_dir)
            .context("failed to match vulnerabilities")?;

        println!("[bazbom] found {} vulnerability matches", matches.len());

        // Create SARIF report
        let mut report = SarifReport::new("BazBOM-SCA", env!("CARGO_PKG_VERSION"));
        
        // Add rules for each unique vulnerability
        let unique_vulns: std::collections::HashSet<_> = matches.iter()
            .map(|m| m.vulnerability_id.clone())
            .collect();
        
        let rules: Vec<Rule> = unique_vulns.into_iter().map(|id| {
            Rule {
                id: id.clone(),
                short_description: MessageString {
                    text: format!("Vulnerability {}", id),
                },
                full_description: None,
                help: None,
                default_configuration: Some(Configuration {
                    level: "warning".to_string(),
                }),
            }
        }).collect();

        if !rules.is_empty() {
            report.runs[0].tool.driver.rules = Some(rules);
        }

        // Add results
        let results = self.create_sarif_results(matches);
        report.runs[0].results = results;

        // Write SARIF to findings directory
        let output_path = ctx.findings_dir.join("sca.sarif");
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&output_path, json)?;

        println!("[bazbom] wrote SCA findings to {:?}", output_path);

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sca_analyzer_enabled() {
        let analyzer = ScaAnalyzer::new();
        let config = Config::default();
        assert!(analyzer.enabled(&config, false));
    }

    #[test]
    fn test_sca_analyzer_run() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");
        let ctx = Context::new(workspace, out_dir)?;

        let analyzer = ScaAnalyzer::new();
        let report = analyzer.run(&ctx)?;
        
        assert_eq!(report.runs.len(), 1);
        assert_eq!(report.runs[0].tool.driver.name, "BazBOM-SCA");
        
        Ok(())
    }
}
