/// PDF report generation module
///
/// Provides PDF generation for BazBOM reports using printpdf.
/// Generates executive summaries, compliance reports, and detailed vulnerability reports.
use anyhow::{Context, Result};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::ReportGenerator;

/// Generate a PDF report from a ReportGenerator
///
/// Creates a professional PDF report with:
/// - Executive summary header
/// - Security score and metrics
/// - Vulnerability breakdown
/// - SBOM statistics
/// - Policy compliance status
pub fn generate_pdf(generator: &ReportGenerator, output_path: &Path) -> Result<()> {
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "BazBOM Security Report",
        Mm(210.0), // A4 width
        Mm(297.0), // A4 height
        "Layer 1",
    );

    // Get the current layer
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Use built-in fonts (Helvetica)
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .context("Failed to load font")?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .context("Failed to load bold font")?;

    // Title page
    let mut y_position = Mm(270.0); // Start from top
    y_position = add_title_page(&current_layer, &font, &font_bold, generator, y_position)?;

    // Executive summary
    y_position = add_section_header(&current_layer, &font_bold, "Executive Summary", y_position)?;
    y_position = add_executive_summary(&current_layer, &font, &font_bold, generator, y_position)?;

    // Vulnerability details
    if y_position < Mm(50.0) {
        // Add new page if running out of space
        let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Page 2");
        let current_layer = doc.get_page(page2).get_layer(layer2);
        y_position = Mm(270.0);
        y_position =
            add_vulnerability_section(&current_layer, &font, &font_bold, generator, y_position)?;
    } else {
        y_position =
            add_vulnerability_section(&current_layer, &font, &font_bold, generator, y_position)?;
    }

    // SBOM and Policy sections
    if y_position < Mm(50.0) {
        let (page3, layer3) = doc.add_page(Mm(210.0), Mm(297.0), "Page 3");
        let current_layer = doc.get_page(page3).get_layer(layer3);
        y_position = Mm(270.0);
        y_position = add_sbom_section(&current_layer, &font, &font_bold, generator, y_position)?;
        add_policy_section(&current_layer, &font, &font_bold, generator, y_position)?;
    } else {
        y_position = add_sbom_section(&current_layer, &font, &font_bold, generator, y_position)?;
        add_policy_section(&current_layer, &font, &font_bold, generator, y_position)?;
    }

    // Save the PDF
    let file = File::create(output_path).context("Failed to create PDF file")?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)
        .context("Failed to save PDF document")?;

    Ok(())
}

/// Add title page content
fn add_title_page(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    generator: &ReportGenerator,
    mut y_pos: Mm,
) -> Result<Mm> {
    let sbom = generator.sbom();

    // Main title
    add_centered_text(layer, font_bold, 24.0, "BazBOM Security Report", y_pos);
    y_pos -= Mm(15.0);

    // Project info
    add_centered_text(
        layer,
        font,
        14.0,
        &format!("Project: {}", sbom.project_name),
        y_pos,
    );
    y_pos -= Mm(8.0);

    add_centered_text(
        layer,
        font,
        14.0,
        &format!("Version: {}", sbom.project_version),
        y_pos,
    );
    y_pos -= Mm(8.0);

    add_centered_text(
        layer,
        font,
        12.0,
        &format!(
            "Scan Date: {}",
            sbom.scan_timestamp.format("%Y-%m-%d %H:%M UTC")
        ),
        y_pos,
    );
    y_pos -= Mm(15.0);

    // Security score
    let security_score = generator.vulnerabilities().security_score();
    add_centered_text(
        layer,
        font_bold,
        18.0,
        &format!("Security Score: {}/100", security_score),
        y_pos,
    );
    y_pos -= Mm(20.0);

    Ok(y_pos)
}

/// Add section header
fn add_section_header(
    layer: &PdfLayerReference,
    font_bold: &IndirectFontRef,
    title: &str,
    mut y_pos: Mm,
) -> Result<Mm> {
    y_pos -= Mm(10.0);
    add_text(layer, font_bold, 18.0, title, Mm(20.0), y_pos);
    y_pos -= Mm(8.0);
    Ok(y_pos)
}

/// Add executive summary
fn add_executive_summary(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    generator: &ReportGenerator,
    mut y_pos: Mm,
) -> Result<Mm> {
    let vulnerabilities = generator.vulnerabilities();
    let sbom = generator.sbom();

    // Key findings
    add_text(layer, font_bold, 12.0, "Key Findings:", Mm(20.0), y_pos);
    y_pos -= Mm(6.0);

    add_text(
        layer,
        font,
        10.0,
        &format!(
            "• Total Dependencies: {} ({} direct, {} transitive)",
            sbom.total_dependencies, sbom.direct_dependencies, sbom.transitive_dependencies
        ),
        Mm(25.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("• Total Vulnerabilities: {}", vulnerabilities.total_count()),
        Mm(25.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("  - Critical: {}", vulnerabilities.critical.len()),
        Mm(30.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("  - High: {}", vulnerabilities.high.len()),
        Mm(30.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("  - Medium: {}", vulnerabilities.medium.len()),
        Mm(30.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("  - Low: {}", vulnerabilities.low.len()),
        Mm(30.0),
        y_pos,
    );
    y_pos -= Mm(8.0);

    // Risk assessment
    add_text(layer, font_bold, 12.0, "Risk Assessment:", Mm(20.0), y_pos);
    y_pos -= Mm(6.0);

    let security_score = vulnerabilities.security_score();
    let risk_level = if security_score >= 90 {
        "LOW - Project has minimal security issues"
    } else if security_score >= 70 {
        "MODERATE - Some vulnerabilities require attention"
    } else if security_score >= 50 {
        "HIGH - Multiple critical vulnerabilities detected"
    } else {
        "CRITICAL - Immediate remediation required"
    };

    add_text(
        layer,
        font,
        10.0,
        &format!("• {}", risk_level),
        Mm(25.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    if !vulnerabilities.critical.is_empty() {
        add_text(
            layer,
            font,
            10.0,
            &format!(
                "• {} CRITICAL vulnerabilities require immediate attention",
                vulnerabilities.critical.len()
            ),
            Mm(25.0),
            y_pos,
        );
        y_pos -= Mm(5.0);
    }

    Ok(y_pos)
}

/// Add vulnerability section
fn add_vulnerability_section(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    generator: &ReportGenerator,
    mut y_pos: Mm,
) -> Result<Mm> {
    let vulnerabilities = generator.vulnerabilities();

    y_pos = add_section_header(layer, font_bold, "Vulnerability Details", y_pos)?;

    // Critical vulnerabilities
    if !vulnerabilities.critical.is_empty() {
        add_text(
            layer,
            font_bold,
            14.0,
            "Critical Vulnerabilities",
            Mm(20.0),
            y_pos,
        );
        y_pos -= Mm(6.0);

        for (i, vuln) in vulnerabilities.critical.iter().take(3).enumerate() {
            if y_pos < Mm(30.0) {
                break; // Stop if running out of space
            }
            y_pos = add_vulnerability_entry(layer, font, font_bold, vuln, y_pos)?;
            if i < 2 {
                y_pos -= Mm(3.0);
            }
        }

        if vulnerabilities.critical.len() > 3 {
            add_text(
                layer,
                font,
                10.0,
                &format!(
                    "... and {} more critical vulnerabilities",
                    vulnerabilities.critical.len() - 3
                ),
                Mm(25.0),
                y_pos,
            );
            y_pos -= Mm(5.0);
        }
    }

    // High vulnerabilities summary
    if !vulnerabilities.high.is_empty() {
        add_text(
            layer,
            font,
            10.0,
            &format!("High Vulnerabilities: {}", vulnerabilities.high.len()),
            Mm(20.0),
            y_pos,
        );
        y_pos -= Mm(5.0);
    }

    // Medium and low summaries
    if !vulnerabilities.medium.is_empty() {
        add_text(
            layer,
            font,
            10.0,
            &format!("Medium Vulnerabilities: {}", vulnerabilities.medium.len()),
            Mm(20.0),
            y_pos,
        );
        y_pos -= Mm(5.0);
    }

    if !vulnerabilities.low.is_empty() {
        add_text(
            layer,
            font,
            10.0,
            &format!("Low Vulnerabilities: {}", vulnerabilities.low.len()),
            Mm(20.0),
            y_pos,
        );
        y_pos -= Mm(5.0);
    }

    Ok(y_pos)
}

/// Add a single vulnerability entry
fn add_vulnerability_entry(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    vuln: &crate::VulnerabilityDetail,
    mut y_pos: Mm,
) -> Result<Mm> {
    // CVE ID and package
    add_text(
        layer,
        font_bold,
        10.0,
        &format!(
            "{} - {} {}",
            vuln.cve, vuln.package_name, vuln.package_version
        ),
        Mm(25.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    // CVSS score
    add_text(
        layer,
        font,
        9.0,
        &format!("CVSS Score: {:.1}", vuln.cvss_score),
        Mm(25.0),
        y_pos,
    );
    y_pos -= Mm(4.0);

    // Description (truncated)
    let desc = if vuln.description.len() > 80 {
        format!("{}...", &vuln.description[..80])
    } else {
        vuln.description.clone()
    };
    add_text(layer, font, 9.0, &desc, Mm(25.0), y_pos);
    y_pos -= Mm(4.0);

    // Fixed version
    if let Some(ref fixed) = vuln.fixed_version {
        add_text(
            layer,
            font,
            9.0,
            &format!("Fixed in: {}", fixed),
            Mm(25.0),
            y_pos,
        );
        y_pos -= Mm(4.0);
    }

    // Flags
    let mut flags = Vec::new();
    if vuln.is_kev {
        flags.push("KEV");
    }
    if vuln.is_reachable {
        flags.push("REACHABLE");
    }
    if !flags.is_empty() {
        add_text(
            layer,
            font,
            9.0,
            &format!("Flags: {}", flags.join(", ")),
            Mm(25.0),
            y_pos,
        );
        y_pos -= Mm(4.0);
    }

    Ok(y_pos)
}

/// Add SBOM section
fn add_sbom_section(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    generator: &ReportGenerator,
    mut y_pos: Mm,
) -> Result<Mm> {
    let sbom = generator.sbom();

    y_pos = add_section_header(layer, font_bold, "Software Bill of Materials (SBOM)", y_pos)?;

    add_text(
        layer,
        font,
        10.0,
        &format!("Total Dependencies: {}", sbom.total_dependencies),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("Direct Dependencies: {}", sbom.direct_dependencies),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("Transitive Dependencies: {}", sbom.transitive_dependencies),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(8.0);

    Ok(y_pos)
}

/// Add policy section
fn add_policy_section(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    generator: &ReportGenerator,
    mut y_pos: Mm,
) -> Result<Mm> {
    let policy = generator.policy();

    y_pos = add_section_header(layer, font_bold, "Policy Compliance", y_pos)?;

    let compliance_status = if policy.policy_violations == 0
        && policy.license_issues == 0
        && policy.blocked_packages == 0
    {
        "COMPLIANT - No policy violations detected"
    } else {
        "NON-COMPLIANT - Policy violations found"
    };

    add_text(
        layer,
        font_bold,
        10.0,
        &format!("Status: {}", compliance_status),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(6.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("Policy Violations: {}", policy.policy_violations),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("License Issues: {}", policy.license_issues),
        Mm(20.0),
        y_pos,
    );
    y_pos -= Mm(5.0);

    add_text(
        layer,
        font,
        10.0,
        &format!("Blocked Packages: {}", policy.blocked_packages),
        Mm(20.0),
        y_pos,
    );

    Ok(y_pos)
}

/// Helper function to add text at a specific position
fn add_text(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    size: f32,
    text: &str,
    x: Mm,
    y: Mm,
) {
    layer.use_text(text, size, x, y, font);
}

/// Helper function to add centered text
fn add_centered_text(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    size: f32,
    text: &str,
    y: Mm,
) {
    // Approximate text width (very rough estimate)
    let char_width = size * 0.5;
    let text_width = (text.len() as f32) * char_width * 0.6;
    let page_width = 210.0; // A4 width in mm
    let x = Mm((page_width - text_width) / 2.0);

    layer.use_text(text, size, x, y, font);
}

/// Check if PDF generation is available
pub fn is_pdf_available() -> bool {
    true // Fully implemented with printpdf!
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[test]
    fn test_pdf_availability() {
        assert!(is_pdf_available());
    }

    #[test]
    fn test_pdf_generation() {
        let temp = TempDir::new().unwrap();
        let pdf_path = temp.path().join("test-report.pdf");

        // Create test data
        let sbom = crate::SbomData {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            scan_timestamp: Utc::now(),
            total_dependencies: 100,
            direct_dependencies: 20,
            transitive_dependencies: 80,
        };

        let vulnerabilities = crate::VulnerabilityFindings {
            critical: vec![crate::VulnerabilityDetail {
                cve: "CVE-2021-44228".to_string(),
                package_name: "log4j-core".to_string(),
                package_version: "2.14.1".to_string(),
                severity: "CRITICAL".to_string(),
                cvss_score: 10.0,
                description: "Log4Shell RCE vulnerability".to_string(),
                fixed_version: Some("2.21.1".to_string()),
                is_reachable: true,
                is_kev: true,
                epss_score: Some(0.975),
            }],
            high: vec![],
            medium: vec![],
            low: vec![],
        };

        let policy = crate::PolicyStatus {
            policy_violations: 1,
            license_issues: 0,
            blocked_packages: 0,
        };

        let generator = crate::ReportGenerator::new(sbom, vulnerabilities, policy);

        // Generate PDF
        let result = generate_pdf(&generator, &pdf_path);

        // Should succeed
        assert!(result.is_ok(), "PDF generation failed: {:?}", result.err());

        // File should exist
        assert!(pdf_path.exists(), "PDF file was not created");

        // File should have content
        let metadata = std::fs::metadata(&pdf_path).unwrap();
        assert!(metadata.len() > 0, "PDF file is empty");
    }
}
