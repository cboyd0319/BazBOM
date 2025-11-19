use crate::merge::EpssScore;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load EPSS scores from a CSV file
/// Format: cve,epss,percentile
/// Example: CVE-2024-1234,0.00123,0.45678
pub fn load_epss_scores<P: AsRef<Path>>(path: P) -> Result<HashMap<String, EpssScore>> {
    let file = File::open(path.as_ref()).context("Failed to open EPSS scores file")?;

    let reader = BufReader::new(file);
    let mut epss_map = HashMap::new();

    // Skip comment lines and header
    let mut lines = reader.lines();
    let mut found_header = false;

    while let Some(Ok(line)) = lines.next() {
        // Skip comment lines
        if line.trim().starts_with('#') {
            continue;
        }
        // Found the header line (should contain "cve")
        if !found_header {
            if !line.to_lowercase().contains("cve") {
                return Err(anyhow::anyhow!("Invalid EPSS CSV header: {}", line));
            }
            #[allow(unused_assignments)]
            {
                found_header = true;
            }
            break;
        }
    }

    for line in lines {
        let line = line.context("Failed to read line from EPSS file")?;

        // Skip empty lines and comments
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            // Skip malformed lines
            continue;
        }

        let cve_id = parts[0].trim().to_string();
        let score = parts[1].trim().parse::<f64>().ok();
        let percentile = parts[2].trim().parse::<f64>().ok();

        if let (Some(score), Some(percentile)) = (score, percentile) {
            epss_map.insert(cve_id, EpssScore { score, percentile });
        }
    }

    Ok(epss_map)
}

/// Try to find EPSS score for a vulnerability by checking ID and aliases
pub fn find_epss_score(
    vuln_id: &str,
    aliases: &[String],
    epss_map: &HashMap<String, EpssScore>,
) -> Option<EpssScore> {
    // Check the main ID first
    if let Some(score) = epss_map.get(vuln_id) {
        return Some(score.clone());
    }

    // Check all aliases
    for alias in aliases {
        if let Some(score) = epss_map.get(alias) {
            return Some(score.clone());
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
    fn test_load_epss_scores_basic() {
        let epss_csv = "cve,epss,percentile\n\
                       CVE-2024-1234,0.00123,0.45678\n\
                       CVE-2024-5678,0.95432,0.99876\n\
                       CVE-2024-9999,0.00001,0.00001\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(epss_csv.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let epss_map = load_epss_scores(temp_file.path()).unwrap();
        assert_eq!(epss_map.len(), 3);

        let score = epss_map.get("CVE-2024-1234").unwrap();
        assert!((score.score - 0.00123).abs() < 0.00001);
        assert!((score.percentile - 0.45678).abs() < 0.00001);
    }

    #[test]
    fn test_load_epss_scores_empty() {
        let epss_csv = "cve,epss,percentile\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(epss_csv.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let epss_map = load_epss_scores(temp_file.path()).unwrap();
        assert_eq!(epss_map.len(), 0);
    }

    #[test]
    fn test_load_epss_scores_with_comments() {
        let epss_csv = "cve,epss,percentile\n\
                       # This is a comment\n\
                       CVE-2024-1234,0.00123,0.45678\n\
                       \n\
                       CVE-2024-5678,0.95432,0.99876\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(epss_csv.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let epss_map = load_epss_scores(temp_file.path()).unwrap();
        assert_eq!(epss_map.len(), 2);
    }

    #[test]
    fn test_load_epss_scores_malformed_lines() {
        let epss_csv = "cve,epss,percentile\n\
                       CVE-2024-1234,0.00123,0.45678\n\
                       CVE-2024-INVALID,not_a_number,0.5\n\
                       CVE-2024-5678,0.95432,0.99876\n";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(epss_csv.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let epss_map = load_epss_scores(temp_file.path()).unwrap();
        // Should skip the malformed line
        assert_eq!(epss_map.len(), 2);
        assert!(epss_map.contains_key("CVE-2024-1234"));
        assert!(epss_map.contains_key("CVE-2024-5678"));
        assert!(!epss_map.contains_key("CVE-2024-INVALID"));
    }

    #[test]
    fn test_find_epss_score_by_id() {
        let mut epss_map = HashMap::new();
        epss_map.insert(
            "CVE-2024-1234".to_string(),
            EpssScore {
                score: 0.5,
                percentile: 0.8,
            },
        );

        let score = find_epss_score("CVE-2024-1234", &[], &epss_map);
        assert!(score.is_some());
        let score = score.unwrap();
        assert_eq!(score.score, 0.5);
        assert_eq!(score.percentile, 0.8);
    }

    #[test]
    fn test_find_epss_score_by_alias() {
        let mut epss_map = HashMap::new();
        epss_map.insert(
            "CVE-2024-1234".to_string(),
            EpssScore {
                score: 0.5,
                percentile: 0.8,
            },
        );

        let aliases = vec![
            "CVE-2024-1234".to_string(),
            "GHSA-xxxx-yyyy-zzzz".to_string(),
        ];
        let score = find_epss_score("GHSA-xxxx-yyyy-zzzz", &aliases, &epss_map);
        assert!(score.is_some());
        let score = score.unwrap();
        assert_eq!(score.score, 0.5);
    }

    #[test]
    fn test_find_epss_score_not_found() {
        let epss_map = HashMap::new();
        let score = find_epss_score("CVE-2024-9999", &[], &epss_map);
        assert!(score.is_none());
    }

    #[test]
    fn test_epss_score_high_risk() {
        let mut epss_map = HashMap::new();
        epss_map.insert(
            "CVE-2024-HIGH".to_string(),
            EpssScore {
                score: 0.95,
                percentile: 0.99,
            },
        );

        let score = find_epss_score("CVE-2024-HIGH", &[], &epss_map).unwrap();
        assert!(score.score > 0.9);
        assert!(score.percentile > 0.95);
    }

    #[test]
    fn test_epss_score_low_risk() {
        let mut epss_map = HashMap::new();
        epss_map.insert(
            "CVE-2024-LOW".to_string(),
            EpssScore {
                score: 0.00001,
                percentile: 0.00001,
            },
        );

        let score = find_epss_score("CVE-2024-LOW", &[], &epss_map).unwrap();
        assert!(score.score < 0.01);
        assert!(score.percentile < 0.01);
    }
}
