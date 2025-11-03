use crate::merge::KevEntry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// CISA KEV (Known Exploited Vulnerabilities) catalog format
/// Based on https://www.cisa.gov/known-exploited-vulnerabilities-catalog
#[derive(Debug, Deserialize, Serialize)]
pub struct KevCatalog {
    pub title: Option<String>,
    #[serde(rename = "catalogVersion")]
    pub catalog_version: Option<String>,
    #[serde(rename = "dateReleased")]
    pub date_released: Option<String>,
    pub count: Option<u32>,
    pub vulnerabilities: Vec<KevVulnerability>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KevVulnerability {
    #[serde(rename = "cveID")]
    pub cve_id: String,
    #[serde(rename = "vendorProject")]
    pub vendor_project: String,
    pub product: String,
    #[serde(rename = "vulnerabilityName")]
    pub vulnerability_name: String,
    #[serde(rename = "dateAdded")]
    pub date_added: String,
    #[serde(rename = "shortDescription")]
    pub short_description: Option<String>,
    #[serde(rename = "requiredAction")]
    pub required_action: String,
    #[serde(rename = "dueDate")]
    pub due_date: String,
    pub notes: Option<String>,
}

/// Load KEV catalog from a JSON file and return a map of CVE ID to KEV entry
pub fn load_kev_catalog<P: AsRef<Path>>(path: P) -> Result<HashMap<String, KevEntry>> {
    let contents = fs::read_to_string(path.as_ref()).context("Failed to read KEV catalog file")?;

    let catalog: KevCatalog =
        serde_json::from_str(&contents).context("Failed to parse KEV catalog JSON")?;

    let mut kev_map = HashMap::new();

    for vuln in catalog.vulnerabilities {
        let entry = KevEntry {
            cve_id: vuln.cve_id.clone(),
            vendor_project: vuln.vendor_project,
            product: vuln.product,
            vulnerability_name: vuln.vulnerability_name,
            date_added: vuln.date_added,
            required_action: vuln.required_action,
            due_date: vuln.due_date,
        };
        kev_map.insert(vuln.cve_id, entry);
    }

    Ok(kev_map)
}

/// Try to find KEV entry for a vulnerability by checking ID and aliases
pub fn find_kev_entry(
    vuln_id: &str,
    aliases: &[String],
    kev_map: &HashMap<String, KevEntry>,
) -> Option<KevEntry> {
    // Check the main ID first
    if let Some(entry) = kev_map.get(vuln_id) {
        return Some(entry.clone());
    }

    // Check all aliases
    for alias in aliases {
        if let Some(entry) = kev_map.get(alias) {
            return Some(entry.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_kev_catalog_basic() {
        let kev_json = r#"{
            "title": "CISA Catalog of Known Exploited Vulnerabilities",
            "catalogVersion": "2024.01.15",
            "dateReleased": "2024-01-15",
            "count": 2,
            "vulnerabilities": [
                {
                    "cveID": "CVE-2024-1234",
                    "vendorProject": "Example Vendor",
                    "product": "Example Product",
                    "vulnerabilityName": "Example Vulnerability",
                    "dateAdded": "2024-01-10",
                    "shortDescription": "Test vulnerability",
                    "requiredAction": "Apply updates per vendor instructions",
                    "dueDate": "2024-02-10",
                    "notes": ""
                },
                {
                    "cveID": "CVE-2024-5678",
                    "vendorProject": "Another Vendor",
                    "product": "Another Product",
                    "vulnerabilityName": "Another Vulnerability",
                    "dateAdded": "2024-01-12",
                    "requiredAction": "Apply patch",
                    "dueDate": "2024-02-12"
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(kev_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let kev_map = load_kev_catalog(temp_file.path()).unwrap();
        assert_eq!(kev_map.len(), 2);
        assert!(kev_map.contains_key("CVE-2024-1234"));
        assert!(kev_map.contains_key("CVE-2024-5678"));
    }

    #[test]
    fn test_load_kev_catalog_empty() {
        let kev_json = r#"{
            "title": "CISA Catalog of Known Exploited Vulnerabilities",
            "catalogVersion": "2024.01.15",
            "dateReleased": "2024-01-15",
            "count": 0,
            "vulnerabilities": []
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(kev_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let kev_map = load_kev_catalog(temp_file.path()).unwrap();
        assert_eq!(kev_map.len(), 0);
    }

    #[test]
    fn test_find_kev_entry_by_id() {
        let mut kev_map = HashMap::new();
        kev_map.insert(
            "CVE-2024-1234".to_string(),
            KevEntry {
                cve_id: "CVE-2024-1234".to_string(),
                vendor_project: "Vendor".to_string(),
                product: "Product".to_string(),
                vulnerability_name: "Vuln".to_string(),
                date_added: "2024-01-10".to_string(),
                required_action: "Patch".to_string(),
                due_date: "2024-02-10".to_string(),
            },
        );

        let entry = find_kev_entry("CVE-2024-1234", &[], &kev_map);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().cve_id, "CVE-2024-1234");
    }

    #[test]
    fn test_find_kev_entry_by_alias() {
        let mut kev_map = HashMap::new();
        kev_map.insert(
            "CVE-2024-1234".to_string(),
            KevEntry {
                cve_id: "CVE-2024-1234".to_string(),
                vendor_project: "Vendor".to_string(),
                product: "Product".to_string(),
                vulnerability_name: "Vuln".to_string(),
                date_added: "2024-01-10".to_string(),
                required_action: "Patch".to_string(),
                due_date: "2024-02-10".to_string(),
            },
        );

        let aliases = vec![
            "CVE-2024-1234".to_string(),
            "GHSA-xxxx-yyyy-zzzz".to_string(),
        ];
        let entry = find_kev_entry("GHSA-xxxx-yyyy-zzzz", &aliases, &kev_map);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().cve_id, "CVE-2024-1234");
    }

    #[test]
    fn test_find_kev_entry_not_found() {
        let kev_map = HashMap::new();
        let entry = find_kev_entry("CVE-2024-9999", &[], &kev_map);
        assert!(entry.is_none());
    }

    #[test]
    fn test_kev_entry_fields() {
        let entry = KevEntry {
            cve_id: "CVE-2024-1234".to_string(),
            vendor_project: "Test Vendor".to_string(),
            product: "Test Product".to_string(),
            vulnerability_name: "Test Vuln".to_string(),
            date_added: "2024-01-10".to_string(),
            required_action: "Apply patch".to_string(),
            due_date: "2024-02-10".to_string(),
        };

        assert_eq!(entry.cve_id, "CVE-2024-1234");
        assert_eq!(entry.vendor_project, "Test Vendor");
        assert_eq!(entry.product, "Test Product");
    }
}
