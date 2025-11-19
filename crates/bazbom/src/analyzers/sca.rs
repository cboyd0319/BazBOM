use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use anyhow::{Context as _, Result};
use bazbom_vulnerabilities::{
    db_sync, is_version_affected, load_epss_scores, load_kev_catalog, Priority, VersionEvent,
    VersionRange,
};
use bazbom_formats::sarif::{
    ArtifactLocation, Configuration, Location, Message, MessageString, PhysicalLocation,
    Result as SarifResult, Rule, SarifReport,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

pub struct ScaAnalyzer;

impl Default for ScaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ScaAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn ensure_advisory_database(&self, ctx: &Context) -> Result<PathBuf> {
        let cache_dir = ctx.workspace.join(".bazbom").join("advisories");
        debug!("Advisory database cache directory: {:?}", cache_dir);

        // Check if we have a recent database (less than 24 hours old)
        let manifest_path = cache_dir.join("manifest.json");
        let needs_sync = if manifest_path.exists() {
            debug!("Manifest file exists at {:?}", manifest_path);
            let metadata = std::fs::metadata(&manifest_path)?;
            let modified = metadata.modified()?;
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or(std::time::Duration::from_secs(86400 * 2));
            debug!("Manifest age: {} seconds ({} hours)", age.as_secs(), age.as_secs() / 3600);
            age.as_secs() > 86400 // More than 24 hours old
        } else {
            debug!("Manifest file does not exist, needs initial sync");
            true
        };

        if needs_sync {
            info!("Syncing advisory database (EPSS and KEV)...");
            println!("[bazbom] syncing advisory database...");
            match db_sync(&cache_dir, false) {
                Ok(_) => {
                    info!("Advisory database sync completed successfully");
                    println!("[bazbom] advisory database synced");
                }
                Err(e) => {
                    warn!("Advisory database sync failed: {}", e);
                    return Err(anyhow::anyhow!("failed to sync advisory database: {}", e));
                }
            }
        } else {
            debug!("Using cached advisory database (less than 24 hours old)");
            println!("[bazbom] using cached advisory database");
        }

        Ok(cache_dir)
    }

    fn load_sbom_components(&self, ctx: &Context) -> Result<Vec<Component>> {
        // Try to load SPDX SBOM from sbom directory
        // NOTE: The file is named "sbom.spdx.json" not "spdx.json"
        let spdx_path = ctx.sbom_dir.join("sbom.spdx.json");
        if !spdx_path.exists() {
            println!(
                "[bazbom] no SBOM found at {:?}, running SCA with empty component list",
                spdx_path
            );
            return Ok(Vec::new());
        }

        println!("[bazbom] parsing SBOM from {:?}", spdx_path);
        let content = std::fs::read_to_string(&spdx_path).context("failed to read SPDX file")?;

        let doc: serde_json::Value =
            serde_json::from_str(&content).context("failed to parse SPDX JSON")?;

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
                            purl = ext_ref["referenceLocator"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            // Extract ecosystem from PURL (format: pkg:ecosystem/...)
                            if let Some(colon_pos) = purl.find(':') {
                                if let Some(slash_pos) = purl[colon_pos..].find('/') {
                                    ecosystem =
                                        purl[colon_pos + 1..colon_pos + slash_pos].to_string();
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

        println!(
            "[bazbom] extracted {} components from SBOM",
            components.len()
        );
        Ok(components)
    }

    fn match_vulnerabilities(
        &self,
        components: &[Component],
        advisory_dir: &PathBuf,
    ) -> Result<Vec<VulnerabilityMatch>> {
        debug!("Starting vulnerability matching for {} components", components.len());

        // Load EPSS scores
        debug!("Loading EPSS scores from {:?}", advisory_dir);
        let epss_scores = load_epss_scores(advisory_dir).unwrap_or_else(|e| {
            warn!("Failed to load EPSS scores: {}", e);
            println!("[bazbom] warning: failed to load EPSS scores: {}", e);
            HashMap::new()
        });

        // Load KEV catalog
        debug!("Loading KEV catalog from {:?}", advisory_dir);
        let kev_entries = load_kev_catalog(advisory_dir).unwrap_or_else(|e| {
            warn!("Failed to load KEV catalog: {}", e);
            println!("[bazbom] warning: failed to load KEV catalog: {}", e);
            HashMap::new()
        });

        info!("Loaded {} EPSS scores and {} KEV entries", epss_scores.len(), kev_entries.len());
        println!(
            "[bazbom] loaded {} EPSS scores and {} KEV entries",
            epss_scores.len(),
            kev_entries.len()
        );

        let mut matches = Vec::new();

        // TODO: Replace local OSV database with OSV API calls
        // The OSV database is too large to cache locally (~100GB+)
        // Instead, use the OSV API batch endpoint: POST https://api.osv.dev/v1/querybatch
        // See: https://google.github.io/osv.dev/post-v1-querybatch/

        // Load OSV database entries (TEMPORARY - will be replaced with API calls)
        let osv_dir = advisory_dir.join("osv");
        if osv_dir.exists() {
            debug!("Scanning local OSV database at {:?}", osv_dir);
            println!("[bazbom] scanning OSV database for vulnerabilities...");
            match self.scan_osv_database(&osv_dir, components, &epss_scores, &kev_entries) {
                Ok(mut osv_matches) => {
                    info!("Found {} OSV vulnerability matches", osv_matches.len());
                    println!("[bazbom] found {} OSV matches", osv_matches.len());
                    matches.append(&mut osv_matches);
                }
                Err(e) => {
                    warn!("OSV scan failed: {}", e);
                    println!("[bazbom] warning: OSV scan failed: {}", e);
                }
            }
        } else {
            warn!("OSV database not found at {:?} - consider implementing OSV API integration", osv_dir);
            println!("[bazbom] OSV database not found at {:?}", osv_dir);
            println!("[bazbom] NOTE: OSV database is too large to cache locally");
            println!("[bazbom] TODO: Implement OSV API integration (https://api.osv.dev/v1/querybatch)");
        }

        info!("Total vulnerability matches: {}", matches.len());
        println!("[bazbom] total vulnerability matches: {}", matches.len());
        Ok(matches)
    }

    fn scan_osv_database(
        &self,
        osv_dir: &PathBuf,
        components: &[Component],
        epss_scores: &HashMap<String, bazbom_vulnerabilities::EpssScore>,
        kev_entries: &HashMap<String, bazbom_vulnerabilities::KevEntry>,
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
                                            // Parse version ranges and check if component version is affected
                                            if let Some(ranges_json) = aff["ranges"].as_array() {
                                                let ranges: Vec<VersionRange> = ranges_json
                                                    .iter()
                                                    .filter_map(|r| self.parse_version_range(r))
                                                    .collect();

                                                if !ranges.is_empty() {
                                                    match is_version_affected(
                                                        &component.version,
                                                        &ranges,
                                                    ) {
                                                        Ok(true) => {
                                                            let epss = epss_scores
                                                                .get(&vuln_id)
                                                                .map(|e| e.score);
                                                            let in_kev =
                                                                kev_entries.contains_key(&vuln_id);

                                                            // Calculate priority based on severity, EPSS, and KEV
                                                            let priority = self.calculate_priority(
                                                                &vuln, epss, in_kev,
                                                            );

                                                            matches.push(VulnerabilityMatch {
                                                                vulnerability_id: vuln_id.clone(),
                                                                component_name: component
                                                                    .name
                                                                    .clone(),
                                                                component_version: component
                                                                    .version
                                                                    .clone(),
                                                                summary: summary.clone(),
                                                                epss_score: epss,
                                                                in_kev,
                                                                priority,
                                                                location: component
                                                                    .location
                                                                    .clone(),
                                                                reachable: None, // Will be populated later
                                                            });
                                                        }
                                                        Ok(false) => {
                                                            // Version not affected, skip
                                                        }
                                                        Err(e) => {
                                                            // Error parsing version, be conservative and include it
                                                            eprintln!("[bazbom]   warning: version check failed for {} {}: {}", 
                                                                component.name, component.version, e);

                                                            let epss = epss_scores
                                                                .get(&vuln_id)
                                                                .map(|e| e.score);
                                                            let in_kev =
                                                                kev_entries.contains_key(&vuln_id);
                                                            let priority = self.calculate_priority(
                                                                &vuln, epss, in_kev,
                                                            );

                                                            matches.push(VulnerabilityMatch {
                                                                vulnerability_id: vuln_id.clone(),
                                                                component_name: component
                                                                    .name
                                                                    .clone(),
                                                                component_version: component
                                                                    .version
                                                                    .clone(),
                                                                summary: summary.clone(),
                                                                epss_score: epss,
                                                                in_kev,
                                                                priority,
                                                                location: component
                                                                    .location
                                                                    .clone(),
                                                                reachable: None, // Will be populated later
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
            }
        }

        Ok(matches)
    }

    fn parse_version_range(&self, range_json: &serde_json::Value) -> Option<VersionRange> {
        let range_type = range_json["type"].as_str()?.to_string();
        let events_json = range_json["events"].as_array()?;

        let mut events = Vec::new();
        for event in events_json {
            if let Some(introduced) = event["introduced"].as_str() {
                events.push(VersionEvent::Introduced {
                    introduced: introduced.to_string(),
                });
            } else if let Some(fixed) = event["fixed"].as_str() {
                events.push(VersionEvent::Fixed {
                    fixed: fixed.to_string(),
                });
            } else if let Some(last_affected) = event["last_affected"].as_str() {
                events.push(VersionEvent::LastAffected {
                    last_affected: last_affected.to_string(),
                });
            }
        }

        if events.is_empty() {
            return None;
        }

        Some(VersionRange { range_type, events })
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
            cvss_score = severity
                .split(':')
                .nth(1)
                .and_then(|s| s.parse::<f64>().ok());
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

    /// Calculate priority from CVSS score and enrichment data (for polyglot vulnerabilities)
    fn calculate_priority_from_cvss(
        &self,
        cvss_score: Option<f64>,
        epss: Option<f64>,
        in_kev: bool,
    ) -> Option<Priority> {
        // P0: KEV entries or high EPSS + Critical/High severity
        if in_kev {
            return Some(Priority::P0);
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

        // Fallback based on CVSS score
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

    fn enrich_with_reachability(
        &self,
        ctx: &Context,
        matches: &mut [VulnerabilityMatch],
    ) -> Result<()> {
        // Load reachability data from polyglot-sbom.json
        let polyglot_sbom_path = ctx.sbom_dir.join("polyglot-sbom.json");

        if !polyglot_sbom_path.exists() {
            // No reachability data available, skip enrichment
            // This is normal for legacy scans or when reachability analysis is disabled
            tracing::debug!(
                "polyglot-sbom.json not found, skipping reachability enrichment"
            );
            return Ok(());
        }

        let content = std::fs::read_to_string(&polyglot_sbom_path)
            .context("Failed to read polyglot-sbom.json")?;
        let sbom: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse polyglot-sbom.json")?;

        // Extract reachability data from ecosystems
        let mut reachability_map: std::collections::HashMap<String, bool> =
            std::collections::HashMap::new();

        if let Some(ecosystems) = sbom.get("ecosystems").and_then(|e| e.as_array()) {
            for ecosystem in ecosystems {
                if let Some(reachability) = ecosystem.get("reachability") {
                    if let Some(vulnerable_packages) = reachability
                        .get("vulnerable_packages_reachable")
                        .and_then(|v| v.as_object())
                    {
                        for (package, is_reachable) in vulnerable_packages {
                            if let Some(reachable_bool) = is_reachable.as_bool() {
                                reachability_map.insert(package.clone(), reachable_bool);
                            }
                        }
                    }
                }
            }
        }

        if !reachability_map.is_empty() {
            println!(
                "[bazbom] loaded reachability data for {} packages",
                reachability_map.len()
            );

            // Enrich vulnerability matches with reachability data
            for vuln_match in matches.iter_mut() {
                // Try exact match first
                if let Some(&is_reachable) = reachability_map.get(&vuln_match.component_name) {
                    vuln_match.reachable = Some(is_reachable);
                } else {
                    // Try fuzzy matching (e.g., "@org/package" vs "org/package")
                    let simplified_name = vuln_match
                        .component_name
                        .trim_start_matches('@')
                        .replace(':', "/");

                    if let Some(&is_reachable) = reachability_map.get(&simplified_name) {
                        vuln_match.reachable = Some(is_reachable);
                    }
                }
            }

            let reachable_count = matches.iter().filter(|m| m.reachable == Some(true)).count();
            let unreachable_count = matches
                .iter()
                .filter(|m| m.reachable == Some(false))
                .count();

            if reachable_count > 0 || unreachable_count > 0 {
                println!(
                    "[bazbom] reachability analysis: {} reachable, {} unreachable",
                    reachable_count, unreachable_count
                );
            }
        }

        Ok(())
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

                let mut message_parts = vec![format!(
                    "Vulnerability {} found in {}",
                    m.vulnerability_id, m.component_name
                )];

                if let Some(summary) = &m.summary {
                    message_parts.push(summary.clone());
                }

                if let Some(epss) = m.epss_score {
                    message_parts.push(format!("EPSS: {:.2}%", epss * 100.0));
                }

                if m.in_kev {
                    message_parts.push(
                        "[!] Listed in CISA KEV (Known Exploited Vulnerabilities)".to_string(),
                    );
                }

                // Add reachability information
                match m.reachable {
                    Some(true) => {
                        message_parts.push(
                            "[!] Code is REACHABLE - vulnerability is exploitable".to_string(),
                        );
                    }
                    Some(false) => {
                        message_parts.push(
                            "[✓] Code is UNREACHABLE - vulnerability not exploitable".to_string(),
                        );
                    }
                    None => {
                        // Reachability unknown, don't add a message
                    }
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

                if let Some(reachable) = m.reachable {
                    properties["reachable"] = serde_json::json!(reachable);
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
                                uri_base_id: None,
                            },
                            region: None,
                        },
                    }]),
                    properties: match serde_json::to_value(properties) {
                        Ok(val) => Some(val),
                        Err(_) => Some(serde_json::json!({})),
                    },
                    fingerprints: None,
                }
            })
            .collect()
    }
}

#[derive(Debug)]
#[allow(dead_code)]
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
    reachable: Option<bool>,
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

        // Check if we have polyglot vulnerability data first
        let polyglot_vulns_path = ctx.findings_dir.join("polyglot-vulns.json");
        let mut matches = if polyglot_vulns_path.exists() {
            // Load vulnerabilities from polyglot scanner
            println!("[bazbom] loading polyglot vulnerability data...");
            let content = std::fs::read_to_string(&polyglot_vulns_path)
                .context("failed to read polyglot vulnerability data")?;

            let ecosystem_results: Vec<bazbom_scanner::EcosystemScanResult> =
                serde_json::from_str(&content)
                    .context("failed to parse polyglot vulnerability data")?;

            // Load EPSS and KEV data for enrichment
            let advisory_dir = match self.ensure_advisory_database(ctx) {
                Ok(dir) => dir,
                Err(e) => {
                    println!("[bazbom] warning: failed to sync advisory database: {}", e);
                    println!("[bazbom] continuing without EPSS/KEV enrichment");
                    ctx.workspace.join(".bazbom").join("advisories")
                }
            };

            let epss_path = advisory_dir.join("advisories").join("epss.csv");
            debug!("Loading EPSS scores from {:?}", epss_path);
            let epss_scores = load_epss_scores(&epss_path).unwrap_or_else(|e| {
                warn!("Failed to load EPSS scores: {}", e);
                println!("[bazbom] warning: failed to load EPSS scores: {}", e);
                HashMap::new()
            });

            let kev_path = advisory_dir.join("advisories").join("kev.json");
            debug!("Loading KEV catalog from {:?}", kev_path);
            let kev_entries = load_kev_catalog(&kev_path).unwrap_or_else(|e| {
                warn!("Failed to load KEV catalog: {}", e);
                println!("[bazbom] warning: failed to load KEV catalog: {}", e);
                HashMap::new()
            });

            info!("Loaded {} EPSS scores and {} KEV entries for polyglot enrichment",
                  epss_scores.len(), kev_entries.len());
            println!("[bazbom] loaded {} EPSS scores and {} KEV entries for enrichment",
                     epss_scores.len(), kev_entries.len());

            // Convert polyglot vulnerabilities to our VulnerabilityMatch format with enrichment
            let mut vuln_matches = Vec::new();
            for ecosystem_result in ecosystem_results {
                for vuln in ecosystem_result.vulnerabilities {
                    // Look up EPSS score for this vulnerability
                    let epss_score = epss_scores.get(&vuln.id).map(|e| e.score);

                    // Check if vulnerability is in CISA KEV catalog
                    let in_kev = kev_entries.contains_key(&vuln.id);

                    // Calculate priority using CVSS score from polyglot scanner
                    let priority = self.calculate_priority_from_cvss(
                        vuln.cvss_score,
                        epss_score,
                        in_kev
                    );

                    vuln_matches.push(VulnerabilityMatch {
                        vulnerability_id: vuln.id.clone(),
                        component_name: vuln.package_name.clone(),
                        component_version: vuln.package_version.clone(),
                        summary: Some(vuln.description.clone()),
                        epss_score,
                        in_kev,
                        priority,
                        location: format!("{}@{}", vuln.package_name, vuln.package_version),
                        reachable: None,   // Will be enriched if reachability data exists
                    });
                }
            }

            println!("[bazbom] loaded {} vulnerabilities from polyglot scanner", vuln_matches.len());
            vuln_matches
        } else {
            // Fallback to traditional SBOM-based vulnerability matching
            println!("[bazbom] no polyglot vulnerability data found, using traditional matching...");

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
            let components = self
                .load_sbom_components(ctx)
                .context("failed to load SBOM components")?;

            println!("[bazbom] loaded {} components from SBOM", components.len());

            // Match vulnerabilities
            let vuln_matches = self
                .match_vulnerabilities(&components, &advisory_dir)
                .context("failed to match vulnerabilities")?;

            println!("[bazbom] found {} vulnerability matches", vuln_matches.len());
            vuln_matches
        };

        // Enrich with reachability data (applies to both paths)
        self.enrich_with_reachability(ctx, &mut matches)?;

        // Create SARIF report
        let mut report = SarifReport::new("BazBOM-SCA", env!("CARGO_PKG_VERSION"));

        // Add rules for each unique vulnerability (using references to avoid cloning)
        let unique_vulns: std::collections::HashSet<&String> =
            matches.iter().map(|m| &m.vulnerability_id).collect();

        let rules: Vec<Rule> = unique_vulns
            .into_iter()
            .map(|id| Rule {
                id: id.clone(),
                short_description: MessageString {
                    text: format!("Vulnerability {}", id),
                },
                full_description: None,
                help: None,
                default_configuration: Some(Configuration {
                    level: "warning".to_string(),
                }),
            })
            .collect();

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
