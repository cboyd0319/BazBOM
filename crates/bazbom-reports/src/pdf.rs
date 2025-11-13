/// PDF report generation module
///
/// Generates PDF versions of security and compliance reports

use anyhow::{Context, Result};
use genpdf::elements::{Paragraph, Text};
use genpdf::fonts;
use genpdf::style::Style;
use genpdf::{Alignment, Element};
use std::path::Path;

use crate::compliance::ComplianceFramework;
use crate::{ReportData, ReportType};

/// Generate a PDF report from report data
pub fn generate_pdf_report(
    report_data: &ReportData,
    report_type: ReportType,
    output_path: &Path,
) -> Result<()> {
    // Load font from system or embedded
    let font_family = load_font_family()?;

    // Create document
    let mut doc = genpdf::Document::new(font_family);

    // Set document metadata
    doc.set_title(&format!("{:?} Report", report_type));
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);

    // Generate report content based on type
    match report_type {
        ReportType::Executive => generate_executive_pdf(&mut doc, report_data)?,
        ReportType::Compliance(framework) => {
            generate_compliance_pdf(&mut doc, report_data, framework)?
        }
        ReportType::Developer => generate_developer_pdf(&mut doc, report_data)?,
        ReportType::Trend => generate_trend_pdf(&mut doc, report_data)?,
    }

    // Render to file
    doc.render_to_file(output_path)
        .context("Failed to render PDF")?;

    Ok(())
}

/// Load font family for PDF generation
fn load_font_family() -> Result<genpdf::fonts::FontFamily<fonts::Font>> {
    // Use built-in Liberation font family (similar to Times New Roman)
    let font_data = genpdf::fonts::from_files("./fonts", "LiberationSerif", None)
        .or_else(|_| {
            // Fallback: try to load from system fonts
            genpdf::fonts::from_files("/usr/share/fonts/truetype/liberation", "LiberationSerif", None)
        })
        .or_else(|_| {
            // Second fallback: use embedded font
            Ok(genpdf::fonts::FontFamily {
                regular: fonts::Font::builtin(fonts::BuiltinFont::Helvetica),
                bold: fonts::Font::builtin(fonts::BuiltinFont::HelveticaBold),
                italic: fonts::Font::builtin(fonts::BuiltinFont::HelveticaOblique),
                bold_italic: fonts::Font::builtin(fonts::BuiltinFont::HelveticaBoldOblique),
            })
        })?;

    Ok(font_data)
}

/// Generate executive summary PDF
fn generate_executive_pdf(doc: &mut genpdf::Document, data: &ReportData) -> Result<()> {
    // Title
    doc.push(
        Paragraph::new("Executive Security Summary")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(20)),
    );

    doc.push(genpdf::elements::Break::new(1.0));

    // Project info
    doc.push(
        Paragraph::new(format!("Project: {}", data.project_name))
            .styled(Style::new().with_font_size(12)),
    );
    doc.push(
        Paragraph::new(format!("Scan Date: {}", data.timestamp))
            .styled(Style::new().with_font_size(10)),
    );

    doc.push(genpdf::elements::Break::new(1.5));

    // Security score
    let score_color = if data.security_score >= 80.0 {
        "Good"
    } else if data.security_score >= 60.0 {
        "Moderate"
    } else {
        "Needs Improvement"
    };

    doc.push(
        Paragraph::new(format!(
            "Security Score: {:.0}/100 ({})",
            data.security_score, score_color
        ))
        .styled(Style::new().bold().with_font_size(14)),
    );

    doc.push(genpdf::elements::Break::new(1.0));

    // Vulnerability summary
    doc.push(
        Paragraph::new("Vulnerability Summary").styled(Style::new().bold().with_font_size(14)),
    );

    doc.push(Paragraph::new(format!(
        "• Critical: {}",
        data.critical_vulns
    )));
    doc.push(Paragraph::new(format!("• High: {}", data.high_vulns)));
    doc.push(Paragraph::new(format!("• Medium: {}", data.medium_vulns)));
    doc.push(Paragraph::new(format!("• Low: {}", data.low_vulns)));

    doc.push(genpdf::elements::Break::new(1.0));

    // Dependencies
    doc.push(
        Paragraph::new("Dependency Analysis").styled(Style::new().bold().with_font_size(14)),
    );
    doc.push(Paragraph::new(format!(
        "Total Dependencies: {}",
        data.total_dependencies
    )));
    doc.push(Paragraph::new(format!(
        "Vulnerable Dependencies: {}",
        data.vulnerable_dependencies
    )));

    // Top findings
    if !data.top_findings.is_empty() {
        doc.push(genpdf::elements::Break::new(1.5));
        doc.push(
            Paragraph::new("Top Security Findings").styled(Style::new().bold().with_font_size(14)),
        );

        for (i, finding) in data.top_findings.iter().take(5).enumerate() {
            doc.push(Paragraph::new(format!("{}. {} - {}", i + 1, finding.cve_id, finding.severity))
                .styled(Style::new().with_font_size(10)));
            doc.push(Paragraph::new(format!("   Package: {}", finding.package))
                .styled(Style::new().with_font_size(9)));
        }
    }

    Ok(())
}

/// Generate compliance report PDF
fn generate_compliance_pdf(
    doc: &mut genpdf::Document,
    data: &ReportData,
    framework: ComplianceFramework,
) -> Result<()> {
    // Title
    doc.push(
        Paragraph::new(format!("{:?} Compliance Report", framework))
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(20)),
    );

    doc.push(genpdf::elements::Break::new(1.0));

    // Project info
    doc.push(Paragraph::new(format!("Project: {}", data.project_name)));
    doc.push(Paragraph::new(format!("Date: {}", data.timestamp)));

    doc.push(genpdf::elements::Break::new(1.5));

    // Compliance status
    doc.push(
        Paragraph::new("Compliance Status").styled(Style::new().bold().with_font_size(14)),
    );

    let status = if data.policy_violations == 0 {
        "COMPLIANT"
    } else {
        "NON-COMPLIANT"
    };

    doc.push(
        Paragraph::new(format!("Status: {}", status))
            .styled(Style::new().bold().with_font_size(12)),
    );
    doc.push(Paragraph::new(format!(
        "Policy Violations: {}",
        data.policy_violations
    )));

    // Framework-specific requirements
    doc.push(genpdf::elements::Break::new(1.0));
    doc.push(
        Paragraph::new("Requirements").styled(Style::new().bold().with_font_size(14)),
    );

    let requirements = get_framework_requirements(framework);
    for req in requirements {
        doc.push(Paragraph::new(format!("• {}", req)).styled(Style::new().with_font_size(10)));
    }

    Ok(())
}

/// Generate developer-focused PDF
fn generate_developer_pdf(doc: &mut genpdf::Document, data: &ReportData) -> Result<()> {
    doc.push(
        Paragraph::new("Developer Security Report")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(18)),
    );

    doc.push(genpdf::elements::Break::new(1.0));

    doc.push(Paragraph::new(format!("Project: {}", data.project_name)));
    doc.push(Paragraph::new(format!("Scan Date: {}", data.timestamp)));

    doc.push(genpdf::elements::Break::new(1.5));

    // Detailed findings
    doc.push(
        Paragraph::new("Vulnerability Details").styled(Style::new().bold().with_font_size(14)),
    );

    for finding in &data.top_findings {
        doc.push(genpdf::elements::Break::new(0.5));
        doc.push(
            Paragraph::new(format!("{} - {}", finding.cve_id, finding.severity))
                .styled(Style::new().bold()),
        );
        doc.push(Paragraph::new(format!("Package: {}", finding.package)));
        doc.push(Paragraph::new(format!("CVSS Score: {}", finding.cvss_score)));
        if let Some(fix) = &finding.fix_available {
            doc.push(Paragraph::new(format!("Fix: Upgrade to {}", fix)));
        }
    }

    Ok(())
}

/// Generate trend analysis PDF
fn generate_trend_pdf(doc: &mut genpdf::Document, data: &ReportData) -> Result<()> {
    doc.push(
        Paragraph::new("Security Trend Analysis")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(18)),
    );

    doc.push(genpdf::elements::Break::new(1.0));

    doc.push(Paragraph::new(format!("Project: {}", data.project_name)));
    doc.push(Paragraph::new("Historical security metrics and trends"));

    // TODO: Add trend charts when chart support is added
    doc.push(genpdf::elements::Break::new(1.0));
    doc.push(Paragraph::new(
        "Note: Graphical trends are available in the HTML report.",
    ));

    Ok(())
}

/// Get compliance framework requirements
fn get_framework_requirements(framework: ComplianceFramework) -> Vec<&'static str> {
    match framework {
        ComplianceFramework::PciDss => vec![
            "Regular vulnerability scanning required",
            "All critical and high vulnerabilities must be addressed",
            "Maintain inventory of system components",
            "Use only secure protocols and encryption",
        ],
        ComplianceFramework::Hipaa => vec![
            "Access controls for sensitive data",
            "Audit controls and monitoring",
            "Data integrity and availability",
            "Regular security assessments",
        ],
        ComplianceFramework::FedRamp => vec![
            "Continuous monitoring required",
            "Incident response procedures",
            "Configuration management",
            "Security assessment and authorization",
        ],
        ComplianceFramework::Soc2 => vec![
            "Security policies and procedures",
            "Change management controls",
            "Risk assessment program",
            "Monitoring and incident response",
        ],
        ComplianceFramework::Gdpr => vec![
            "Data protection by design",
            "Security of processing",
            "Data breach notification procedures",
            "Privacy impact assessments",
        ],
        ComplianceFramework::Iso27001 => vec![
            "Information security management system",
            "Risk assessment and treatment",
            "Security controls implementation",
            "Continuous improvement process",
        ],
        ComplianceFramework::Nist => vec![
            "Identify assets and risks",
            "Protect critical infrastructure",
            "Detect security events",
            "Respond to incidents",
            "Recover from disruptions",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pdf_generation() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test_report.pdf");

        let data = ReportData {
            project_name: "Test Project".to_string(),
            timestamp: "2025-01-01".to_string(),
            security_score: 85.0,
            critical_vulns: 0,
            high_vulns: 2,
            medium_vulns: 5,
            low_vulns: 10,
            total_dependencies: 150,
            vulnerable_dependencies: 17,
            policy_violations: 0,
            top_findings: vec![],
        };

        let result = generate_pdf_report(&data, ReportType::Executive, &pdf_path);
        assert!(result.is_ok());
        assert!(pdf_path.exists());
    }
}
