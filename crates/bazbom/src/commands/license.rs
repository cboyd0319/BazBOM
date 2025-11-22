use anyhow::Result;
use bazbom::cli::LicenseCmd;
use bazbom_formats::{cyclonedx::CycloneDxBom, spdx::SpdxDocument};
use std::fs;
use std::path::Path;

/// Handle the `bazbom license` command
pub fn handle_license(action: LicenseCmd) -> Result<()> {
    match action {
        LicenseCmd::Obligations { sbom_file } => handle_obligations(sbom_file),
        LicenseCmd::Compatibility {
            project_license,
            sbom_file,
        } => handle_compatibility(project_license, sbom_file),
        LicenseCmd::Contamination { sbom_file } => handle_contamination(sbom_file),
    }
}

fn handle_obligations(sbom_file: Option<String>) -> Result<()> {
    println!("[bazbom] generating license obligations report");

    // Default to common SBOM locations
    let sbom_path = sbom_file
        .as_deref()
        .or_else(|| {
            if Path::new("sbom/spdx.json").exists() {
                Some("sbom/spdx.json")
            } else if Path::new("sbom.spdx.json").exists() {
                Some("sbom.spdx.json")
            } else if Path::new("sbom/cyclonedx.json").exists() {
                Some("sbom/cyclonedx.json")
            } else {
                None
            }
        })
        .ok_or_else(|| {
            anyhow::anyhow!("No SBOM file found. Run 'bazbom scan' first or specify --sbom-file")
        })?;

    println!("[bazbom] reading SBOM from: {}", sbom_path);

    // Parse SBOM
    let packages = parse_sbom(sbom_path)?;

    if packages.is_empty() {
        println!("WARN  No packages found in SBOM");
        return Ok(());
    }

    println!("[bazbom] found {} packages in SBOM", packages.len());

    let obligations_db = bazbom_formats::licenses::LicenseObligations::new();

    println!("\n# License Obligations Report\n");
    println!("Source: {}\n", sbom_path);
    println!("Total packages: {}\n", packages.len());

    // Group by license
    let mut license_counts: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for pkg in &packages {
        if let Some(license) = &pkg.license {
            license_counts
                .entry(license.clone())
                .or_insert_with(Vec::new)
                .push(format!("{}@{}", pkg.name, pkg.version));
        }
    }

    // Show obligations for each license found
    for (license, components) in license_counts.iter() {
        println!("## {} ({} packages)\n", license, components.len());

        if let Some(obligations) = obligations_db.get(license) {
            for obligation in obligations {
                println!(
                    "- **{:?}**: {} (Severity: {:?})",
                    obligation.obligation_type, obligation.description, obligation.severity
                );
            }

            // Show first few affected packages
            println!("\nAffected packages:");
            for (i, component) in components.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, component);
            }
            if components.len() > 5 {
                println!("  ... and {} more", components.len() - 5);
            }
        } else {
            println!(
                "WARN  No obligation data available for license: {}",
                license
            );
        }
        println!();
    }

    // Show packages without licenses
    let no_license: Vec<_> = packages.iter().filter(|p| p.license.is_none()).collect();

    if !no_license.is_empty() {
        println!(
            "## WARN  Packages Without License Information ({} packages)\n",
            no_license.len()
        );
        for (i, pkg) in no_license.iter().take(10).enumerate() {
            println!("  {}. {}@{}", i + 1, pkg.name, pkg.version);
        }
        if no_license.len() > 10 {
            println!("  ... and {} more", no_license.len() - 10);
        }
        println!();
    }

    Ok(())
}

fn handle_compatibility(project_license: String, sbom_file: Option<String>) -> Result<()> {
    println!("[bazbom] checking license compatibility");
    println!("Project license: {}", project_license);

    // Default to common SBOM locations
    let sbom_path = sbom_file
        .as_deref()
        .or_else(|| {
            if Path::new("sbom/spdx.json").exists() {
                Some("sbom/spdx.json")
            } else if Path::new("sbom.spdx.json").exists() {
                Some("sbom.spdx.json")
            } else if Path::new("sbom/cyclonedx.json").exists() {
                Some("sbom/cyclonedx.json")
            } else {
                None
            }
        })
        .ok_or_else(|| {
            anyhow::anyhow!("No SBOM file found. Run 'bazbom scan' first or specify --sbom-file")
        })?;

    println!("[bazbom] reading SBOM from: {}", sbom_path);

    // Parse SBOM
    let packages = parse_sbom(sbom_path)?;

    if packages.is_empty() {
        println!("WARN  No packages found in SBOM");
        return Ok(());
    }

    println!("[bazbom] found {} packages in SBOM", packages.len());

    println!("\n# License Compatibility Report\n");
    println!("Project License: {}\n", project_license);

    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;
    let mut safe_count = 0;

    for pkg in &packages {
        if let Some(dep_license) = &pkg.license {
            let risk = bazbom_formats::licenses::LicenseCompatibility::is_compatible(
                &project_license,
                dep_license,
            );

            let (indicator, color) = match risk {
                bazbom_formats::licenses::LicenseRisk::Safe => {
                    safe_count += 1;
                    ("OK", "Safe")
                }
                bazbom_formats::licenses::LicenseRisk::Low => {
                    low_count += 1;
                    ("WARN ", "Low")
                }
                bazbom_formats::licenses::LicenseRisk::Medium => {
                    medium_count += 1;
                    ("WARN ", "Medium")
                }
                bazbom_formats::licenses::LicenseRisk::High => {
                    high_count += 1;
                    ("FAIL", "High")
                }
                bazbom_formats::licenses::LicenseRisk::Critical => {
                    critical_count += 1;
                    ("🔥", "Critical")
                }
            };

            // Only show problematic licenses (Medium and above)
            if matches!(
                risk,
                bazbom_formats::licenses::LicenseRisk::Medium
                    | bazbom_formats::licenses::LicenseRisk::High
                    | bazbom_formats::licenses::LicenseRisk::Critical
            ) {
                println!(
                    "{} {}@{} ({}) - Risk: {}",
                    indicator, pkg.name, pkg.version, dep_license, color
                );
            }
        }
    }

    // Summary
    println!("\n## Summary\n");
    println!("Total packages analyzed: {}", packages.len());
    println!("  OK Safe: {}", safe_count);
    if low_count > 0 {
        println!("  WARN  Low risk: {}", low_count);
    }
    if medium_count > 0 {
        println!("  WARN  Medium risk: {}", medium_count);
    }
    if high_count > 0 {
        println!("  FAIL High risk: {}", high_count);
    }
    if critical_count > 0 {
        println!("  🔥 Critical risk: {}", critical_count);
    }

    if critical_count > 0 || high_count > 0 {
        println!("\nFAIL License compatibility issues found!");
        std::process::exit(1);
    } else if medium_count > 0 {
        println!("\nWARN  Some license compatibility warnings present");
    } else {
        println!("\nOK All licenses compatible!");
    }

    Ok(())
}

fn handle_contamination(sbom_file: Option<String>) -> Result<()> {
    println!("[bazbom] detecting copyleft contamination");

    // Default to common SBOM locations
    let sbom_path = sbom_file
        .as_deref()
        .or_else(|| {
            if Path::new("sbom/spdx.json").exists() {
                Some("sbom/spdx.json")
            } else if Path::new("sbom.spdx.json").exists() {
                Some("sbom.spdx.json")
            } else if Path::new("sbom/cyclonedx.json").exists() {
                Some("sbom/cyclonedx.json")
            } else {
                None
            }
        })
        .ok_or_else(|| {
            anyhow::anyhow!("No SBOM file found. Run 'bazbom scan' first or specify --sbom-file")
        })?;

    println!("[bazbom] reading SBOM from: {}", sbom_path);

    // Parse SBOM
    let packages = parse_sbom(sbom_path)?;

    if packages.is_empty() {
        println!("WARN  No packages found in SBOM");
        return Ok(());
    }

    println!("[bazbom] found {} packages in SBOM", packages.len());

    // Convert to Dependency format for contamination check
    let dependencies: Vec<bazbom_formats::licenses::Dependency> = packages
        .iter()
        .filter_map(|pkg| {
            pkg.license
                .as_ref()
                .map(|license| bazbom_formats::licenses::Dependency {
                    name: format!("{}:{}", pkg.name, pkg.version),
                    license: license.clone(),
                })
        })
        .collect();

    if dependencies.is_empty() {
        println!("WARN  No packages with license information found");
        return Ok(());
    }

    let warnings =
        bazbom_formats::licenses::LicenseCompatibility::check_contamination(&dependencies);

    println!("\n# Copyleft Contamination Report\n");
    println!("Source: {}\n", sbom_path);
    println!("Total packages analyzed: {}\n", dependencies.len());

    if warnings.is_empty() {
        println!("OK No copyleft contamination detected");
    } else {
        for warning in &warnings {
            let risk_indicator = match warning.risk {
                bazbom_formats::licenses::LicenseRisk::Critical => "[XX] CRITICAL",
                bazbom_formats::licenses::LicenseRisk::High => "[X] HIGH",
                bazbom_formats::licenses::LicenseRisk::Medium => "[!] MEDIUM",
                _ => "[i] INFO",
            };

            println!("{}: {}", risk_indicator, warning.message);
            println!(
                "Affected licenses: {}",
                warning.affected_licenses.join(", ")
            );
            println!();
        }

        // Show affected packages
        println!("\n## Affected Packages\n");
        for dep in &dependencies {
            for warning in &warnings {
                if warning.affected_licenses.contains(&dep.license) {
                    println!("  - {} ({})", dep.name, dep.license);
                }
            }
        }
    }

    Ok(())
}
/// Parsed package information from SBOM
#[derive(Debug, Clone)]
struct PackageInfo {
    name: String,
    version: String,
    license: Option<String>,
}

/// Parse SBOM file and extract packages with license information
fn parse_sbom(sbom_path: &str) -> Result<Vec<PackageInfo>> {
    let path = Path::new(sbom_path);

    if !path.exists() {
        anyhow::bail!("SBOM file not found: {}", sbom_path);
    }

    let content = fs::read_to_string(path)?;

    // Try to detect format from content
    if content.contains("spdxVersion") {
        parse_spdx_sbom(&content)
    } else if content.contains("bomFormat") {
        parse_cyclonedx_sbom(&content)
    } else {
        anyhow::bail!("Unknown SBOM format. Expected SPDX or CycloneDX.")
    }
}

/// Parse SPDX SBOM
fn parse_spdx_sbom(content: &str) -> Result<Vec<PackageInfo>> {
    let doc: SpdxDocument = serde_json::from_str(content)?;

    let packages = doc
        .packages
        .into_iter()
        .map(|pkg| PackageInfo {
            name: pkg.name.clone(),
            version: pkg.version_info.unwrap_or_else(|| "unknown".to_string()),
            // Prefer license_concluded over license_declared
            license: pkg
                .license_concluded
                .or(pkg.license_declared)
                .and_then(|l| {
                    // Filter out NOASSERTION and NONE
                    if l == "NOASSERTION" || l == "NONE" {
                        None
                    } else {
                        Some(l)
                    }
                }),
        })
        .collect();

    Ok(packages)
}

/// Parse CycloneDX SBOM
fn parse_cyclonedx_sbom(content: &str) -> Result<Vec<PackageInfo>> {
    let bom: CycloneDxBom = serde_json::from_str(content)?;

    let packages = bom
        .components
        .into_iter()
        .map(|component| {
            // Extract first license if available
            let license = component
                .licenses
                .and_then(|licenses| licenses.first().cloned())
                .map(|lic| match lic.license {
                    bazbom_formats::cyclonedx::LicenseChoice::Id { id } => id,
                    bazbom_formats::cyclonedx::LicenseChoice::Name { name } => name,
                });

            PackageInfo {
                name: component.name.clone(),
                version: component.version.unwrap_or_else(|| "unknown".to_string()),
                license,
            }
        })
        .collect();

    Ok(packages)
}
