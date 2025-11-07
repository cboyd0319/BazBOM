use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use bazbom_reports::{
    ComplianceFramework, PolicyStatus, ReportGenerator, ReportType, SbomData,
    VulnerabilityDetail, VulnerabilityFindings,
};
use chrono::Utc;
use bazbom::cli::{ComplianceFrameworkArg, ReportCmd};

/// Handle the `bazbom report` command
pub fn handle_report(action: ReportCmd) -> Result<()> {
    match action {
        ReportCmd::Executive {
            sbom,
            findings,
            output,
        } => generate_executive_report(sbom, findings, output),
        ReportCmd::Compliance {
            framework,
            sbom,
            findings,
            output,
        } => generate_compliance_report(framework, sbom, findings, output),
        ReportCmd::Developer {
            sbom,
            findings,
            output,
        } => generate_developer_report(sbom, findings, output),
        ReportCmd::Trend {
            sbom,
            findings,
            output,
        } => generate_trend_report(sbom, findings, output),
        ReportCmd::All {
            sbom,
            findings,
            output_dir,
        } => generate_all_reports(sbom, findings, output_dir),
    }
}

fn generate_executive_report(
    sbom: Option<String>,
    findings: Option<String>,
    output: String,
) -> Result<()> {
    println!("[*] Generating executive summary report...");

    let sbom_data = load_sbom_data(sbom.as_deref())?;
    let vulnerabilities = load_vulnerabilities(findings.as_deref())?;
    let policy = create_default_policy();

    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
    generator.generate(ReportType::Executive, Path::new(&output))?;

    println!("[+] Executive report generated: {}", output);
    Ok(())
}

fn generate_compliance_report(
    framework: ComplianceFrameworkArg,
    sbom: Option<String>,
    findings: Option<String>,
    output: String,
) -> Result<()> {
    let framework_name = convert_framework(framework);
    println!(
        "[*] Generating compliance report for {}...",
        framework_name.name()
    );

    let sbom_data = load_sbom_data(sbom.as_deref())?;
    let vulnerabilities = load_vulnerabilities(findings.as_deref())?;
    let policy = create_default_policy();

    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
    generator.generate(ReportType::Compliance(framework_name), Path::new(&output))?;

    println!("[+] Compliance report generated: {}", output);
    Ok(())
}

fn generate_developer_report(
    sbom: Option<String>,
    findings: Option<String>,
    output: String,
) -> Result<()> {
    println!("[*] Generating developer report...");

    let sbom_data = load_sbom_data(sbom.as_deref())?;
    let vulnerabilities = load_vulnerabilities(findings.as_deref())?;
    let policy = create_default_policy();

    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
    generator.generate(ReportType::Developer, Path::new(&output))?;

    println!("[+] Developer report generated: {}", output);
    Ok(())
}

fn generate_trend_report(
    sbom: Option<String>,
    findings: Option<String>,
    output: String,
) -> Result<()> {
    println!("[*] Generating trend report...");

    let sbom_data = load_sbom_data(sbom.as_deref())?;
    let vulnerabilities = load_vulnerabilities(findings.as_deref())?;
    let policy = create_default_policy();

    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
    generator.generate(ReportType::Trend, Path::new(&output))?;

    println!("[+] Trend report generated: {}", output);
    Ok(())
}

fn generate_all_reports(
    sbom: Option<String>,
    findings: Option<String>,
    output_dir: String,
) -> Result<()> {
    println!("[*] Generating all reports...");

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    let sbom_data = load_sbom_data(sbom.as_deref())?;
    let vulnerabilities = load_vulnerabilities(findings.as_deref())?;
    let policy = create_default_policy();

    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);

    // Generate all report types
    let reports = vec![
        (
            ReportType::Executive,
            format!("{}/executive-report.html", output_dir),
        ),
        (
            ReportType::Developer,
            format!("{}/developer-report.html", output_dir),
        ),
        (
            ReportType::Trend,
            format!("{}/trend-report.html", output_dir),
        ),
        (
            ReportType::Compliance(ComplianceFramework::PciDss),
            format!("{}/compliance-pci-dss.html", output_dir),
        ),
        (
            ReportType::Compliance(ComplianceFramework::Hipaa),
            format!("{}/compliance-hipaa.html", output_dir),
        ),
        (
            ReportType::Compliance(ComplianceFramework::Soc2),
            format!("{}/compliance-soc2.html", output_dir),
        ),
    ];

    for (report_type, output_path) in reports {
        generator.generate(report_type, Path::new(&output_path))?;
        println!("  [+] {}", output_path);
    }

    println!("\n[+] All reports generated in: {}", output_dir);
    Ok(())
}

// Helper functions

fn load_vulnerabilities(findings_path: Option<&str>) -> Result<VulnerabilityFindings> {
    if let Some(path) = findings_path {
        load_findings_from_file(path)
    } else {
        Ok(VulnerabilityFindings {
            critical: vec![],
            high: vec![],
            medium: vec![],
            low: vec![],
        })
    }
}

fn create_default_policy() -> PolicyStatus {
    PolicyStatus {
        policy_violations: 0,
        license_issues: 0,
        blocked_packages: 0,
    }
}

fn load_findings_from_file(path: &str) -> Result<VulnerabilityFindings> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read findings file: {}", path))?;
    let findings: serde_json::Value = serde_json::from_str(&content)?;

    // Parse vulnerabilities by severity
    let critical = extract_vulnerabilities(&findings, "CRITICAL");
    let high = extract_vulnerabilities(&findings, "HIGH");
    let medium = extract_vulnerabilities(&findings, "MEDIUM");
    let low = extract_vulnerabilities(&findings, "LOW");

    Ok(VulnerabilityFindings {
        critical,
        high,
        medium,
        low,
    })
}

fn extract_vulnerabilities(
    findings: &serde_json::Value,
    severity: &str,
) -> Vec<VulnerabilityDetail> {
    findings
        .get("vulnerabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if v.get("severity")?.as_str()? == severity {
                        Some(VulnerabilityDetail {
                            cve: v
                                .get("id")
                                .or_else(|| v.get("cve"))
                                .and_then(|id| id.as_str())
                                .unwrap_or("UNKNOWN")
                                .to_string(),
                            package_name: v
                                .get("package")
                                .or_else(|| v.get("package_name"))
                                .and_then(|p| p.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            package_version: v
                                .get("version")
                                .or_else(|| v.get("package_version"))
                                .and_then(|ver| ver.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            severity: severity.to_string(),
                            cvss_score: v
                                .get("cvss")
                                .or_else(|| v.get("cvss_score"))
                                .and_then(|s| s.as_f64())
                                .unwrap_or(0.0),
                            description: v
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("No description")
                                .to_string(),
                            fixed_version: v
                                .get("fixed_version")
                                .and_then(|f| f.as_str())
                                .map(|s| s.to_string()),
                            is_reachable: v
                                .get("reachable")
                                .or_else(|| v.get("is_reachable"))
                                .and_then(|r| r.as_bool())
                                .unwrap_or(false),
                            is_kev: v
                                .get("kev")
                                .or_else(|| v.get("is_kev"))
                                .and_then(|k| k.as_bool())
                                .unwrap_or(false),
                            epss_score: v
                                .get("epss")
                                .or_else(|| v.get("epss_score"))
                                .and_then(|e| e.as_f64()),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn load_sbom_data(sbom_path: Option<&str>) -> Result<SbomData> {
    if let Some(path) = sbom_path {
        // Try to parse SBOM file
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read SBOM file: {}", path))?;
        let sbom: serde_json::Value = serde_json::from_str(&content)?;

        Ok(SbomData {
            project_name: sbom
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown Project")
                .to_string(),
            project_version: sbom
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            scan_timestamp: Utc::now(),
            total_dependencies: sbom
                .get("packages")
                .and_then(|p| p.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0),
            direct_dependencies: 0, // Would need graph analysis
            transitive_dependencies: 0,
        })
    } else {
        // Return default data
        Ok(SbomData {
            project_name: "Unknown Project".to_string(),
            project_version: "0.0.0".to_string(),
            scan_timestamp: Utc::now(),
            total_dependencies: 0,
            direct_dependencies: 0,
            transitive_dependencies: 0,
        })
    }
}

fn convert_framework(arg: ComplianceFrameworkArg) -> ComplianceFramework {
    match arg {
        ComplianceFrameworkArg::PciDss => ComplianceFramework::PciDss,
        ComplianceFrameworkArg::Hipaa => ComplianceFramework::Hipaa,
        ComplianceFrameworkArg::FedRampModerate => ComplianceFramework::FedRampModerate,
        ComplianceFrameworkArg::Soc2 => ComplianceFramework::Soc2,
        ComplianceFrameworkArg::Gdpr => ComplianceFramework::Gdpr,
        ComplianceFrameworkArg::Iso27001 => ComplianceFramework::Iso27001,
        ComplianceFrameworkArg::NistCsf => ComplianceFramework::NistCsf,
    }
}
