use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub generated_at: String,
    pub files: Vec<ManifestEntry>,
}

fn write_file<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<ManifestEntry> {
    let path_ref = path.as_ref();
    if let Some(parent) = path_ref.parent() { fs::create_dir_all(parent)?; }
    fs::write(path_ref, content)?;
    let hash = blake3::hash(content).to_hex().to_string();
    Ok(ManifestEntry { path: path_ref.to_string_lossy().into_owned(), bytes: content.len() as u64, blake3: hash })
}

fn fetch_or_placeholder(url: &str, placeholder: &[u8]) -> Vec<u8> {
    match ureq::get(url).call() {
        Ok(resp) if resp.ok() => resp.into_bytes().unwrap_or_else(|_| placeholder.to_vec()),
        _ => placeholder.to_vec(),
    }
}

pub fn db_sync<P: AsRef<Path>>(cache_dir: P, offline: bool) -> Result<Manifest> {
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir)?;
    let mut files = Vec::new();

    // Deterministic placeholders
    let placeholder_json = b"{\n  \"note\": \"offline or fetch failed\"\n}\n";
    let placeholder_csv = b"cve,score\n";

    // OSV (placeholder)
    let osv_bytes = if offline { placeholder_json.to_vec() } else { fetch_or_placeholder("https://api.osv.dev/v1/vulns", placeholder_json) };
    files.push(write_file(cache_dir.join("advisories/osv.json"), &osv_bytes)?);

    // NVD (placeholder)
    let nvd_bytes = if offline { placeholder_json.to_vec() } else { fetch_or_placeholder("https://services.nvd.nist.gov/rest/json/cves/2.0?resultsPerPage=1", placeholder_json) };
    files.push(write_file(cache_dir.join("advisories/nvd.json"), &nvd_bytes)?);

    // GHSA (placeholder)
    let ghsa_bytes = placeholder_json.to_vec(); // GitHub token required for real calls
    files.push(write_file(cache_dir.join("advisories/ghsa.json"), &ghsa_bytes)?);

    // CISA KEV
    let kev_bytes = if offline { placeholder_json.to_vec() } else { fetch_or_placeholder("https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json", placeholder_json) };
    files.push(write_file(cache_dir.join("advisories/kev.json"), &kev_bytes)?);

    // EPSS
    let epss_bytes = if offline { placeholder_csv.to_vec() } else { fetch_or_placeholder("https://epss.cyentia.com/epss_scores-current.csv.gz", placeholder_csv) };
    files.push(write_file(cache_dir.join("advisories/epss.csv"), &epss_bytes)?);

    // Manifest
    let now = OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or_else(|_| "".into());
    let manifest = Manifest { generated_at: now, files };
    let data = serde_json::to_vec_pretty(&manifest)?;
    write_file(cache_dir.join("manifest.json"), &data)?;
    Ok(manifest)
}
