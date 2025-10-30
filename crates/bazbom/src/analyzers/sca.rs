use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use anyhow::{Context as _, Result};
use bazbom_advisories::{db_sync, load_epss_scores, load_kev_catalog, Priority};
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

        println!("[bazbom] parsing SBOM from {:?}", spdx_path);
        let content = std::fs::read_to_string(&spdx_path)
            .context("failed to read SPDX file")?;
        
        let doc: serde_json::Value = serde_json::from_str(&content)
            .context("failed to parse SPDX JSON")?;

        let mut components = Vec::new();

        if let Some(packages) = doc["packages"].as_array() {
            for pkg in packages {
                let name = pkg["name"].as_str().unwrap_or("unknown").to_string();
                let version = pkg["versionInfo"].as_str().unwrap_or("").to_string();
                
                // Try to extract PURL from externalRefs
                let mut ecosystem = "maven".to_string(); // Default to maven
                let mut purl = String::new();
                
                if let Some(refs) = pkg["externalRefs"].as_array() {
                    for ext_ref in refs {
                        if ext_ref["referenceType"].as_str() == Some("purl") {
                            purl = ext_ref["referenceLocator"].as_str().unwrap_or("").to_string();
                            // Extract ecosystem from PURL (format: pkg:ecosystem/...)
                            if let Some(colon_pos) = purl.find(':') {
                                if let Some(slash_pos) = purl[colon_pos..].find('/') {
                                    ecosystem = purl[colon_pos + 1..colon_pos + slash_pos].to_string();
                                }
                            }
                            break;
                        }
                    }
                }

                // Generate default PURL if not found
                if purl.is_empty() && !name.is_empty() && !version.is_empty() {
                    purl = format!("pkg:maven/{}@{}", name.replace('.', "/"), version);
                }

                components.push(Component {
                    name: name.clone(),
                    version: version.clone(),
                    ecosystem,
                    location: format!("{}@{}", name, version),
                    purl,
                });
            }
        }

        println!("[bazbom] extracted {} components from SBOM", components.len());
        Ok(components)
    }

    fn match_vulnerabilities(
        &self,
        components: &[Component],
        advisory_dir: &PathBuf,
    ) -> Result<Vec<VulnerabilityMatch>> {
        // Load EPSS scores
        let epss_scores = load_epss_scores(advisory_dir)
            .unwrap_or_else(|e| {
                println!("[bazbom] warning: failed to load EPSS scores: {}", e);
                HashMap::new()
            });

        // Load KEV catalog
        let kev_entries = load_kev_catalog(advisory_dir)
            .unwrap_or_else(|e| {
                println!("[bazbom] warning: failed to load KEV catalog: {}", e);
                HashMap::new()
            });

        println!("[bazbom] loaded {} EPSS scores and {} KEV entries", epss_scores.len(), kev_entries.len());

        let mut matches = Vec::new();

        // Load OSV database entries
        let osv_dir = advisory_dir.join("osv");
        if osv_dir.exists() {
            println!("[bazbom] scanning OSV database for vulnerabilities...");
            match self.scan_osv_database(&osv_dir, components, &epss_scores, &kev_entries) {
                Ok(mut osv_matches) => {
                    println!("[bazbom] found {} OSV matches", osv_matches.len());
                    matches.append(&mut osv_matches);
                }
                Err(e) => {
                    println!("[bazbom] warning: OSV scan failed: {}", e);
                }
            }
        } else {
            println!("[bazbom] OSV database not found at {:?}", osv_dir);
        }

        println!("[bazbom] total vulnerability matches: {}", matches.len());
        Ok(matches)
    }

    fn scan_osv_database(
        &self,
        osv_dir: &PathBuf,
        components: &[Component],
        epss_scores: &HashMap<String, bazbom_advisories::EpssScore>,
        kev_entries: &HashMap<String, bazbom_advisories::KevEntry>,
    ) -> Result<Vec<VulnerabilityMatch>> {
        use std::fs;

        let mut matches = Vec::new();
        
        // Create a lookup map of components by name for faster matching
        let mut component_map: HashMap<String, &Component> = HashMap::new();
        for comp in components {
            component_map.insert(comp.name.clone(), comp);
        }

        // Scan all JSON files in OSV directory
        if let Ok(entries) = fs::read_dir(osv_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(vuln) = serde_json::from_str::<serde_json::Value>(&content) {
                            // Extract vulnerability info
                            let vuln_id = vuln["id"].as_str().unwrap_or("").to_string();
                            let summary = vuln["summary"].as_str().map(|s| s.to_string());
                            
                            // Check if this vulnerability affects any of our components
                            if let Some(affected) = vuln["affected"].as_array() {
                                for aff in affected {
                                    if let Some(pkg) = aff["package"]["name"].as_str() {
                                        // Check if we have this component
                                        if let Some(component) = component_map.get(pkg) {
                                            // Simple version check - in production, use proper version range matching
                                            if let Some(ranges) = aff["ranges"].as_array() {
                                                // For now, assume affected if ranges exist
                                                // Full implementation would parse version ranges
                                                let has_affected_version = !ranges.is_empty();
                                                
                                                if has_affected_version {
                                                    let epss = epss_scores.get(&vuln_id).map(|e| e.score);
                                                    let in_kev = kev_entries.contains_key(&vuln_id);
                                                    
                                                    // Calculate priority based on severity, EPSS, and KEV
                                                    let priority = self.calculate_priority(&vuln, epss, in_kev);
                                                    
                                                    matches.push(VulnerabilityMatch {
                                                        vulnerability_id: vuln_id.clone(),
                                                        component_name: component.name.clone(),
                                                        component_version: component.version.clone(),
                                                        summary: summary.clone(),
                                                        epss_score: epss,
                                                        in_kev,
                                                        priority,
                                                        location: component.location.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(matches)
    }

    fn calculate_priority(
        &self,
        vuln: &serde_json::Value,
        epss: Option<f64>,
        in_kev: bool,
    ) -> Option<Priority> {
        // P0: KEV entries or high EPSS + Critical/High severity
        if in_kev {
            return Some(Priority::P0);
        }

        // Try to extract CVSS score
        let mut cvss_score: Option<f64> = None;
        if let Some(severity) = vuln["database_specific"]["severity"].as_str() {
            cvss_score = severity.split(':').nth(1).and_then(|s| s.parse::<f64>().ok());
        }

        if let Some(epss_val) = epss {
            if let Some(cvss) = cvss_score {
                if epss_val > 0.7 && cvss >= 9.0 {
                    return Some(Priority::P0);
                }
                if epss_val > 0.5 && cvss >= 7.0 {
                    return Some(Priority::P1);
                }
            }
        }

        // Fallback based on severity
        if let Some(cvss) = cvss_score {
            if cvss >= 9.0 {
                return Some(Priority::P1);
            } else if cvss >= 7.0 {
                return Some(Priority::P2);
            } else if cvss >= 4.0 {
                return Some(Priority::P3);
            } else {
                return Some(Priority::P4);
            }
        }

        // Default priority if we can't determine
        Some(Priority::P3)
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
                    properties: match serde_json::to_value(properties) {
                        Ok(val) => Some(val),
                        Err(_) => Some(serde_json::json!({})),
                    },
                }
            })
            .collect()
    }
}

#[derive(Debug)]
struct Component {
    name: String,
    version: String,
    ecosystem: String,
    location: String,
    purl: String,
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
        
        // Add rules for each unique vulnerability (using references to avoid cloning)
        let unique_vulns: std::collections::HashSet<&String> = matches.iter()
            .map(|m| &m.vulnerability_id)
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
