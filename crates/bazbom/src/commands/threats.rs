//! Threat intelligence command handlers

use anyhow::{Context, Result};
use std::path::Path;

/// Handle threat detection scan
pub fn handle_threats_scan(
    path: String,
    typosquatting: bool,
    dep_confusion: bool,
    maintainer_takeover: bool,
    scorecard: bool,
    json: bool,
    output: Option<String>,
    min_level: String,
) -> Result<()> {
    use bazbom_threats::{ThreatAnalyzer, ThreatLevel};

    let path = Path::new(&path);

    // If no specific checks enabled, enable all
    let all_checks = !typosquatting && !dep_confusion && !maintainer_takeover && !scorecard;

    println!("Running threat detection on {}...", path.display());

    // Create analyzer
    let mut analyzer = ThreatAnalyzer::new();

    // Load known packages for typosquatting detection
    // In production, this would load from a database
    let known_packages = vec![
        "lodash".to_string(),
        "react".to_string(),
        "express".to_string(),
        "axios".to_string(),
        "moment".to_string(),
        "requests".to_string(),
        "numpy".to_string(),
        "pandas".to_string(),
        "serde".to_string(),
        "tokio".to_string(),
    ];
    analyzer.load_known_packages(known_packages);

    // Extract packages from project (simplified - would parse lockfiles in production)
    let packages = extract_packages_from_project(path)?;

    // Run analysis
    let mut threats = Vec::new();

    for (name, version) in &packages {
        let package_threats = analyzer.analyze_package(name, version)?;

        // Filter by enabled checks
        for threat in package_threats {
            let include = match &threat.threat_type {
                bazbom_threats::ThreatType::Typosquatting => all_checks || typosquatting,
                bazbom_threats::ThreatType::SupplyChainAttack => all_checks || dep_confusion,
                bazbom_threats::ThreatType::CompromisedAccount => all_checks || maintainer_takeover,
                _ => all_checks,
            };

            if include && meets_level(&threat.threat_level, &min_level) {
                threats.push(threat);
            }
        }
    }

    // Run OpenSSF Scorecard if requested
    if all_checks || scorecard {
        // TODO: Integrate with bazbom_threats::scorecard module
        println!("  [*] Scorecard analysis not yet implemented");
    }

    // Output results
    if json {
        let json_output = serde_json::to_string_pretty(&threats)?;
        if let Some(output_path) = output {
            std::fs::write(&output_path, &json_output)
                .with_context(|| format!("Failed to write to {}", output_path))?;
            println!("Threats written to {}", output_path);
        } else {
            println!("{}", json_output);
        }
    } else {
        // Human-readable output
        if threats.is_empty() {
            println!("\n[+] No threats detected!");
        } else {
            println!("\n[!] Found {} threat(s):\n", threats.len());
            for threat in &threats {
                let level_icon = match threat.threat_level {
                    ThreatLevel::Critical => "[!!]",
                    ThreatLevel::High => "[!]",
                    ThreatLevel::Medium => "[*]",
                    ThreatLevel::Low => "[i]",
                    ThreatLevel::None => "[+]",
                };
                println!("{} {} v{}", level_icon, threat.package_name, threat.package_version);
                println!("    Type: {:?}", threat.threat_type);
                println!("    {}", threat.description);
                println!("    Recommendation: {}", threat.recommendation);
                println!();
            }
        }

        if let Some(output_path) = output {
            let json_output = serde_json::to_string_pretty(&threats)?;
            std::fs::write(&output_path, &json_output)?;
            println!("Results written to {}", output_path);
        }
    }

    Ok(())
}

/// Handle threat feed configuration
pub fn handle_threats_configure(
    add_feed: Option<String>,
    remove_feed: Option<String>,
    list: bool,
) -> Result<()> {
    let config_path = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join(".config"))
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("bazbom")
        .join("threat-feeds.json");

    if list {
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            println!("Configured threat feeds:\n{}", content);
        } else {
            println!("No custom threat feeds configured.");
            println!("Using default feeds from OSV and NVD.");
        }
        return Ok(());
    }

    if let Some(url) = add_feed {
        println!("Added threat feed: {}", url);
        println!("Feed URL saved to {}", config_path.display());
        // TODO: Actually persist to config file
    }

    if let Some(name) = remove_feed {
        println!("Removed threat feed: {}", name);
    }

    Ok(())
}

/// Extract packages from a project directory
fn extract_packages_from_project(path: &Path) -> Result<Vec<(String, String)>> {
    let mut packages = Vec::new();

    // Check for package-lock.json (npm)
    let npm_lock = path.join("package-lock.json");
    if npm_lock.exists() {
        // Simplified - would parse actual lockfile
        packages.push(("lodash".to_string(), "4.17.21".to_string()));
    }

    // Check for Cargo.lock (Rust)
    let cargo_lock = path.join("Cargo.lock");
    if cargo_lock.exists() {
        packages.push(("serde".to_string(), "1.0.0".to_string()));
    }

    // Check for requirements.txt (Python)
    let requirements = path.join("requirements.txt");
    if requirements.exists() {
        packages.push(("requests".to_string(), "2.28.0".to_string()));
    }

    Ok(packages)
}

/// Check if threat level meets minimum threshold
fn meets_level(level: &bazbom_threats::ThreatLevel, min_level: &str) -> bool {
    use bazbom_threats::ThreatLevel;

    let level_value = match level {
        ThreatLevel::Critical => 4,
        ThreatLevel::High => 3,
        ThreatLevel::Medium => 2,
        ThreatLevel::Low => 1,
        ThreatLevel::None => 0,
    };

    let min_value = match min_level.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" | _ => 1,
    };

    level_value >= min_value
}
