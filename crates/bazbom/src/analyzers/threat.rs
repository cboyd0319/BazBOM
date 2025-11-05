use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use anyhow::{Context as _, Result};
use bazbom_formats::sarif::{
    ArtifactLocation, Location, Message, MessageString, PhysicalLocation, Result as SarifResult,
    Rule, SarifReport,
};
use bazbom_threats::{
    database_integration::MaliciousPackageDatabase,
    dependency_confusion::DependencyConfusionDetector, typosquatting, ThreatIndicator, ThreatLevel,
    ThreatType,
};
use std::collections::HashMap;
use std::fmt;

/// Threat intelligence analyzer
pub struct ThreatAnalyzer {
    level: ThreatDetectionLevel,
}

/// Threat detection level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreatDetectionLevel {
    /// No threat detection
    Off = 0,
    /// Basic threat detection (known malicious packages only)
    Basic = 1,
    /// Standard threat detection (malicious + typosquatting)
    Standard = 2,
    /// Full threat detection (all checks including supply chain)
    Full = 3,
}

impl fmt::Display for ThreatDetectionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThreatDetectionLevel::Off => write!(f, "off"),
            ThreatDetectionLevel::Basic => write!(f, "basic"),
            ThreatDetectionLevel::Standard => write!(f, "standard"),
            ThreatDetectionLevel::Full => write!(f, "full"),
        }
    }
}

impl Default for ThreatAnalyzer {
    fn default() -> Self {
        Self::new(ThreatDetectionLevel::Standard)
    }
}

impl ThreatAnalyzer {
    /// Create a new threat analyzer with specified detection level
    pub fn new(level: ThreatDetectionLevel) -> Self {
        Self { level }
    }

    /// Create analyzer from configuration
    pub fn from_config(config: &Config) -> Self {
        let level = config
            .threats
            .as_ref()
            .and_then(|t| t.detection_level.as_deref())
            .and_then(|s| match s {
                "off" => Some(ThreatDetectionLevel::Off),
                "basic" => Some(ThreatDetectionLevel::Basic),
                "standard" => Some(ThreatDetectionLevel::Standard),
                "full" => Some(ThreatDetectionLevel::Full),
                _ => None,
            })
            .unwrap_or(ThreatDetectionLevel::Standard);

        Self::new(level)
    }

    fn load_sbom_components(&self, ctx: &Context) -> Result<Vec<Component>> {
        let spdx_path = ctx.sbom_dir.join("spdx.json");
        if !spdx_path.exists() {
            println!(
                "[bazbom] no SBOM found for threat analysis at {:?}",
                spdx_path
            );
            return Ok(Vec::new());
        }

        println!(
            "[bazbom] loading SBOM for threat analysis from {:?}",
            spdx_path
        );
        let content = std::fs::read_to_string(&spdx_path).context("failed to read SPDX file")?;

        let doc: serde_json::Value =
            serde_json::from_str(&content).context("failed to parse SPDX JSON")?;

        let mut components = Vec::new();

        if let Some(packages) = doc["packages"].as_array() {
            for pkg in packages {
                let name = pkg["name"].as_str().unwrap_or("unknown").to_string();
                let version = pkg["versionInfo"].as_str().unwrap_or("").to_string();

                // Extract PURL if available
                let mut purl = None;
                if let Some(refs) = pkg["externalRefs"].as_array() {
                    for ext_ref in refs {
                        if ext_ref["referenceType"].as_str() == Some("purl") {
                            purl = ext_ref["referenceLocator"].as_str().map(|s| s.to_string());
                            break;
                        }
                    }
                }

                components.push(Component {
                    name,
                    version,
                    purl,
                });
            }
        }

        println!(
            "[bazbom] loaded {} components for threat analysis",
            components.len()
        );
        Ok(components)
    }

    fn detect_threats(&self, components: &[Component]) -> Result<Vec<ThreatIndicator>> {
        let mut threats = Vec::new();

        if self.level == ThreatDetectionLevel::Off {
            return Ok(threats);
        }

        println!(
            "[bazbom] running threat detection (level: {:?})...",
            self.level
        );

        // 1. Query malicious package database (basic level)
        if self.level >= ThreatDetectionLevel::Basic {
            let _db = MaliciousPackageDatabase::new();
            // In a real implementation, we would load actual malicious package data
            // For now, we'll check against known patterns

            for component in components {
                // Check for common malicious patterns in package names
                let lower_name = component.name.to_lowercase();
                if lower_name.contains("malicious")
                    || lower_name.contains("backdoor")
                    || lower_name.contains("trojan")
                {
                    threats.push(ThreatIndicator {
                        package_name: component.name.clone(),
                        package_version: component.version.clone(),
                        threat_level: ThreatLevel::Critical,
                        threat_type: ThreatType::MaliciousPackage,
                        description: "Package name contains suspicious keywords".to_string(),
                        evidence: vec!["Package name matches malicious pattern".to_string()],
                        recommendation: "Remove this package immediately and review dependencies."
                            .to_string(),
                    });
                }
            }
        }

        // 2. Typosquatting detection (standard level)
        if self.level >= ThreatDetectionLevel::Standard {
            // Build a set of well-known packages for comparison
            let mut known_packages = std::collections::HashSet::new();
            known_packages.insert("commons-io".to_string());
            known_packages.insert("spring-core".to_string());
            known_packages.insert("log4j-core".to_string());
            known_packages.insert("jackson-databind".to_string());

            for component in components {
                if let Some(indicator) =
                    typosquatting::check_typosquatting(&component.name, &known_packages)
                {
                    threats.push(indicator);
                }
            }
        }

        // 3. Dependency confusion detection (full level)
        if self.level >= ThreatDetectionLevel::Full {
            let mut detector = DependencyConfusionDetector::new();
            // Load internal package patterns (common patterns for internal packages)
            let internal_patterns = vec![
                "internal-".to_string(),
                "company-".to_string(),
                "private-".to_string(),
            ];
            detector.load_internal_packages(internal_patterns);

            for component in components {
                // Check if component name starts with an internal pattern
                if component.name.starts_with("internal-")
                    || component.name.starts_with("company-")
                    || component.name.starts_with("private-")
                {
                    threats.push(ThreatIndicator {
                        package_name: component.name.clone(),
                        package_version: component.version.clone(),
                        threat_level: ThreatLevel::High,
                        threat_type: ThreatType::SupplyChainAttack,
                        description: "Potential dependency confusion - internal package naming pattern detected".to_string(),
                        evidence: vec!["Package has internal naming pattern".to_string()],
                        recommendation: "Verify package source and consider using scoped packages.".to_string(),
                    });
                }
            }
        }

        println!(
            "[bazbom] threat detection complete: {} threats found",
            threats.len()
        );
        Ok(threats)
    }

    fn threats_to_sarif(&self, threats: Vec<ThreatIndicator>, ctx: &Context) -> SarifReport {
        let mut report = SarifReport::new("BazBOM Threat Intelligence", env!("CARGO_PKG_VERSION"));
        let mut rules_seen = HashMap::new();

        for threat in threats {
            let rule_id = format!("{:?}", threat.threat_type)
                .to_lowercase()
                .replace(' ', "-");

            // Add rule if not already present
            if !rules_seen.contains_key(&rule_id) {
                let rule = Rule {
                    id: rule_id.clone(),
                    short_description: MessageString {
                        text: format!("{:?} detected", threat.threat_type),
                    },
                    full_description: Some(MessageString {
                        text: "Supply chain threat detected in dependencies".to_string(),
                    }),
                    help: Some(MessageString {
                        text: threat.recommendation.clone(),
                    }),
                    default_configuration: None,
                };
                report.add_rule(rule);
                rules_seen.insert(rule_id.clone(), true);
            }

            // Create SARIF result
            let level_str = match threat.threat_level {
                ThreatLevel::Critical => "error",
                ThreatLevel::High => "error",
                ThreatLevel::Medium => "warning",
                ThreatLevel::Low => "note",
                ThreatLevel::None => "none",
            };

            let evidence_text = if !threat.evidence.is_empty() {
                format!("\n\nEvidence:\n- {}", threat.evidence.join("\n- "))
            } else {
                String::new()
            };

            let result = SarifResult {
                rule_id,
                message: Message {
                    text: format!(
                        "{}: {} v{}{}",
                        threat.description,
                        threat.package_name,
                        threat.package_version,
                        evidence_text
                    ),
                },
                level: level_str.to_string(),
                locations: Some(vec![Location {
                    physical_location: PhysicalLocation {
                        artifact_location: ArtifactLocation {
                            uri: ctx.workspace.join("pom.xml").to_string_lossy().to_string(),
                            uri_base_id: None,
                        },
                        region: None,
                    },
                }]),
                properties: None,
                fingerprints: None,
            };
            report.add_result(result);
        }

        report
    }
}

impl Analyzer for ThreatAnalyzer {
    fn id(&self) -> &'static str {
        "bazbom-threat-intelligence"
    }

    fn enabled(&self, config: &Config, _explicit: bool) -> bool {
        // Enabled if threats section exists and is not explicitly disabled
        config
            .threats
            .as_ref()
            .and_then(|t| t.enabled)
            .unwrap_or(true)
            && self.level != ThreatDetectionLevel::Off
    }

    fn run(&self, ctx: &Context) -> Result<SarifReport> {
        println!("[bazbom] running threat intelligence analysis...");

        // Load components from SBOM
        let components = self.load_sbom_components(ctx)?;

        if components.is_empty() {
            println!("[bazbom] no components to analyze for threats");
            return Ok(SarifReport::new(
                "BazBOM Threat Intelligence",
                env!("CARGO_PKG_VERSION"),
            ));
        }

        // Detect threats
        let threats = self.detect_threats(&components)?;

        // Convert to SARIF
        let report = self.threats_to_sarif(threats, ctx);

        Ok(report)
    }
}

#[derive(Debug, Clone)]
struct Component {
    name: String,
    version: String,
    #[allow(dead_code)]
    purl: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_threat_analyzer_creation() {
        let analyzer = ThreatAnalyzer::new(ThreatDetectionLevel::Standard);
        assert_eq!(analyzer.level, ThreatDetectionLevel::Standard);
    }

    #[test]
    fn test_threat_analyzer_from_config() {
        let config = Config::default();
        let analyzer = ThreatAnalyzer::from_config(&config);
        assert_eq!(analyzer.level, ThreatDetectionLevel::Standard);
    }

    #[test]
    fn test_threat_analyzer_enabled() {
        let config = Config::default();
        let analyzer = ThreatAnalyzer::new(ThreatDetectionLevel::Standard);
        assert!(analyzer.enabled(&config, false));
    }

    #[test]
    fn test_threat_analyzer_run() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");
        let ctx = Context::new(workspace, out_dir)?;

        let analyzer = ThreatAnalyzer::new(ThreatDetectionLevel::Standard);
        let report = analyzer.run(&ctx)?;

        // Should return empty report when no SBOM exists
        assert_eq!(report.runs[0].results.len(), 0);

        Ok(())
    }
}
