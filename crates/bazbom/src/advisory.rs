use anyhow::{Context, Result};
use bazbom_advisories::{
    load_epss_scores, load_kev_catalog, calculate_priority, Vulnerability,
};
use bazbom_advisories::parsers::{
    parse_ghsa_entry, parse_nvd_entry, parse_osv_entry,
    GhsaEntry, NvdEntry, OsvEntry,
};
use bazbom_graph::Component;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load and enrich vulnerabilities from the advisory cache
pub fn load_advisories<P: AsRef<Path>>(cache_dir: P) -> Result<Vec<Vulnerability>> {
    let cache_dir = cache_dir.as_ref();
    let mut vulnerabilities = Vec::new();

    // Load OSV advisories
    let osv_path = cache_dir.join("advisories/osv.json");
    if osv_path.exists() {
        let content = fs::read_to_string(&osv_path)
            .with_context(|| format!("failed to read OSV file at {:?}", osv_path))?;
        
        // Try to parse as a single entry or skip if it's a placeholder
        if !content.contains("\"note\"") {
            if let Ok(osv_entry) = serde_json::from_str::<OsvEntry>(&content) {
                if let Ok(vuln) = parse_osv_entry(&osv_entry) {
                    vulnerabilities.push(vuln);
                }
            }
        }
    }

    // Load NVD advisories
    let nvd_path = cache_dir.join("advisories/nvd.json");
    if nvd_path.exists() {
        let content = fs::read_to_string(&nvd_path)
            .with_context(|| format!("failed to read NVD file at {:?}", nvd_path))?;
        
        // Try to parse as a single entry or skip if it's a placeholder
        if !content.contains("\"note\"") {
            if let Ok(nvd_entry) = serde_json::from_str::<NvdEntry>(&content) {
                if let Ok(vuln) = parse_nvd_entry(&nvd_entry) {
                    vulnerabilities.push(vuln);
                }
            }
        }
    }

    // Load GHSA advisories
    let ghsa_path = cache_dir.join("advisories/ghsa.json");
    if ghsa_path.exists() {
        let content = fs::read_to_string(&ghsa_path)
            .with_context(|| format!("failed to read GHSA file at {:?}", ghsa_path))?;
        
        // Try to parse as a single entry or skip if it's a placeholder
        if !content.contains("\"note\"") {
            if let Ok(ghsa_entry) = serde_json::from_str::<GhsaEntry>(&content) {
                if let Ok(vuln) = parse_ghsa_entry(&ghsa_entry) {
                    vulnerabilities.push(vuln);
                }
            }
        }
    }

    // Load enrichment data
    let kev_path = cache_dir.join("advisories/kev.json");
    let kev_catalog = if kev_path.exists() {
        let content = fs::read_to_string(&kev_path)?;
        load_kev_catalog(&content).ok()
    } else {
        None
    };

    let epss_path = cache_dir.join("advisories/epss.csv");
    let epss_scores = if epss_path.exists() {
        // Try to read as string, but gracefully handle gzipped content
        match fs::read_to_string(&epss_path) {
            Ok(content) => load_epss_scores(&content).ok(),
            Err(_) => {
                // File might be gzipped or binary, skip for now
                None
            }
        }
    } else {
        None
    };

    // Enrich vulnerabilities with KEV and EPSS data
    if let Some(kev_map) = kev_catalog {
        for vuln in &mut vulnerabilities {
            // Check if this vulnerability is in KEV
            if let Some(kev_entry) = kev_map.get(&vuln.id) {
                vuln.kev = Some(kev_entry.clone());
            }
            // Also check aliases
            for alias in &vuln.aliases {
                if let Some(kev_entry) = kev_map.get(alias) {
                    vuln.kev = Some(kev_entry.clone());
                    break;
                }
            }
        }
    }

    if let Some(epss_map) = epss_scores {
        for vuln in &mut vulnerabilities {
            // Check if this vulnerability has EPSS data
            if let Some(epss_score) = epss_map.get(&vuln.id) {
                vuln.epss = Some(epss_score.clone());
            }
            // Also check aliases
            for alias in &vuln.aliases {
                if let Some(epss_score) = epss_map.get(alias) {
                    vuln.epss = Some(epss_score.clone());
                    break;
                }
            }
        }
    }

    // Calculate priorities for all vulnerabilities
    for vuln in &mut vulnerabilities {
        vuln.priority = Some(calculate_priority(&vuln.severity, &vuln.kev, &vuln.epss));
    }

    Ok(vulnerabilities)
}

/// Match vulnerabilities to components based on package ecosystem and name
pub fn match_vulnerabilities(
    components: &[Component],
    vulnerabilities: &[Vulnerability],
) -> HashMap<String, Vec<Vulnerability>> {
    let mut matches: HashMap<String, Vec<Vulnerability>> = HashMap::new();

    for component in components {
        let component_key = format!("{}:{}", component.name, component.version);
        
        for vuln in vulnerabilities {
            for affected in &vuln.affected {
                // Simple matching based on package name
                // In a real implementation, this would need version range checking
                if affected.package == component.name {
                    matches
                        .entry(component_key.clone())
                        .or_insert_with(Vec::new)
                        .push(vuln.clone());
                }
            }
        }
    }

    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use bazbom_advisories::{AffectedPackage, Severity, SeverityLevel, VersionRange};
    use bazbom_graph::ComponentId;

    #[test]
    fn test_load_advisories_empty_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();

        let result = load_advisories(&cache_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_match_vulnerabilities_no_matches() {
        let components = vec![Component {
            id: ComponentId::new("test:safe-lib:1.0"),
            name: "safe-lib".to_string(),
            version: "1.0".to_string(),
            purl: Some("pkg:maven/test/safe-lib@1.0".to_string()),
            license: None,
            scope: None,
            hash: None,
        }];

        let vulnerabilities = vec![Vulnerability {
            id: "CVE-2024-1234".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "vulnerable-lib".to_string(),
                ranges: vec![],
            }],
            severity: None,
            summary: None,
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        }];

        let matches = match_vulnerabilities(&components, &vulnerabilities);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_match_vulnerabilities_with_match() {
        let components = vec![Component {
            id: ComponentId::new("test:log4j:2.14.0"),
            name: "log4j".to_string(),
            version: "2.14.0".to_string(),
            purl: Some("pkg:maven/test/log4j@2.14.0".to_string()),
            license: None,
            scope: None,
            hash: None,
        }];

        let vulnerabilities = vec![Vulnerability {
            id: "CVE-2021-44228".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "log4j".to_string(),
                ranges: vec![],
            }],
            severity: Some(Severity {
                cvss_v3: Some(10.0),
                cvss_v4: None,
                level: SeverityLevel::Critical,
            }),
            summary: Some("Log4Shell vulnerability".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        }];

        let matches = match_vulnerabilities(&components, &vulnerabilities);
        assert_eq!(matches.len(), 1);
        assert!(matches.contains_key("log4j:2.14.0"));
        assert_eq!(matches["log4j:2.14.0"].len(), 1);
    }
}
