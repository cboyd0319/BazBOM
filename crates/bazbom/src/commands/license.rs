use anyhow::Result;
use bazbom::cli::LicenseCmd;

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

    let sbom_path = sbom_file.as_deref().unwrap_or("sbom.spdx.json");

    if sbom_file.is_some() {
        println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
    }

    let obligations_db = bazbom_formats::licenses::LicenseObligations::new();

    println!("\n# License Obligations Report\n");
    println!("Example output for: {}\n", sbom_path);

    let example_licenses = vec![
        ("MIT", "example-mit-lib:1.0.0"),
        ("Apache-2.0", "example-apache-lib:2.0.0"),
        ("GPL-3.0-only", "example-gpl-lib:3.0.0"),
    ];

    for (license, component) in example_licenses {
        if let Some(obligations) = obligations_db.get(license) {
            println!("## {} ({})\n", component, license);
            for obligation in obligations {
                println!(
                    "- **{:?}**: {} (Severity: {:?})",
                    obligation.obligation_type, obligation.description, obligation.severity
                );
            }
            println!();
        }
    }

    println!("Note: This is a demonstration. Full SBOM parsing integration coming soon.");
    Ok(())
}

fn handle_compatibility(project_license: String, sbom_file: Option<String>) -> Result<()> {
    println!("[bazbom] checking license compatibility");
    println!("Project license: {}", project_license);

    if let Some(sbom) = &sbom_file {
        println!("SBOM file: {}", sbom);
        println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
    }

    let test_dependencies = vec![
        ("MIT", "example-mit-lib"),
        ("Apache-2.0", "example-apache-lib"),
        ("GPL-3.0-only", "example-gpl-lib"),
        ("AGPL-3.0-only", "example-agpl-lib"),
    ];

    println!("\n# License Compatibility Report\n");

    for (dep_license, dep_name) in test_dependencies {
        let risk = bazbom_formats::licenses::LicenseCompatibility::is_compatible(
            &project_license,
            dep_license,
        );

        let risk_str = format!("{:?}", risk);
        let indicator = match risk {
            bazbom_formats::licenses::LicenseRisk::Safe => "[+]",
            bazbom_formats::licenses::LicenseRisk::Low => "[!]",
            bazbom_formats::licenses::LicenseRisk::Medium => "[!]",
            bazbom_formats::licenses::LicenseRisk::High => "[X]",
            bazbom_formats::licenses::LicenseRisk::Critical => "[XX]",
        };

        println!(
            "{} {} ({}) - Risk: {}",
            indicator, dep_name, dep_license, risk_str
        );
    }

    println!("\nNote: This is a demonstration. Full SBOM parsing integration coming soon.");
    Ok(())
}

fn handle_contamination(sbom_file: Option<String>) -> Result<()> {
    println!("[bazbom] detecting copyleft contamination");

    if let Some(sbom) = &sbom_file {
        println!("SBOM file: {}", sbom);
        println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
    }

    let test_dependencies = vec![
        bazbom_formats::licenses::Dependency {
            name: "example-mit-lib:1.0.0".to_string(),
            license: "MIT".to_string(),
        },
        bazbom_formats::licenses::Dependency {
            name: "example-gpl-lib:2.0.0".to_string(),
            license: "GPL-3.0-only".to_string(),
        },
        bazbom_formats::licenses::Dependency {
            name: "example-agpl-lib:3.0.0".to_string(),
            license: "AGPL-3.0-only".to_string(),
        },
    ];

    let warnings =
        bazbom_formats::licenses::LicenseCompatibility::check_contamination(&test_dependencies);

    println!("\n# Copyleft Contamination Report\n");

    if warnings.is_empty() {
        println!("[+] No copyleft contamination detected");
    } else {
        for warning in warnings {
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
    }

    println!("Note: This is a demonstration. Full SBOM parsing integration coming soon.");
    Ok(())
}
