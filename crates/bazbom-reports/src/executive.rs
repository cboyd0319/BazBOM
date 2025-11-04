//! Executive summary report generation
//!
//! Generates 1-page HTML/PDF summaries suitable for executives and CISOs.

use crate::{write_html_file, ReportGenerator};
use anyhow::Result;
use std::path::Path;

/// Generate an executive summary HTML report (can be converted to PDF)
pub fn generate_executive_report(generator: &ReportGenerator, output_path: &Path) -> Result<()> {
    let score = generator.vulnerabilities().security_score();
    let score_label = match score {
        90..=100 => "EXCELLENT",
        75..=89 => "GOOD",
        60..=74 => "FAIR",
        40..=59 => "POOR",
        _ => "CRITICAL",
    };

    let score_color = match score {
        90..=100 => "#10b981", // green
        75..=89 => "#3b82f6", // blue
        60..=74 => "#f59e0b", // yellow
        40..=59 => "#ef4444", // orange
        _ => "#dc2626", // red
    };

    let vulns = generator.vulnerabilities();
    let policy = generator.policy();
    let recommendations = generate_recommendations(generator);

    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BazBOM Executive Security Summary</title>
    <style>
        @media print {{
            @page {{ margin: 1.5cm; }}
            body {{ margin: 0; }}
        }}
        
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            padding: 40px;
            max-width: 210mm;
            margin: 0 auto;
            background: #f9fafb;
        }}

        .report {{
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }}

        h1 {{
            font-size: 32px;
            font-weight: 700;
            color: #111827;
            margin-bottom: 24px;
        }}

        .metadata {{
            color: #6b7280;
            font-size: 14px;
            margin-bottom: 32px;
            padding-bottom: 16px;
            border-bottom: 2px solid #e5e7eb;
        }}

        .metadata p {{
            margin-bottom: 4px;
        }}

        .score-card {{
            background: linear-gradient(135deg, {score_color} 0%, {score_color}dd 100%);
            color: white;
            padding: 32px;
            border-radius: 8px;
            margin: 24px 0;
            text-align: center;
        }}

        .score-value {{
            font-size: 64px;
            font-weight: 700;
            line-height: 1;
        }}

        .score-label {{
            font-size: 24px;
            font-weight: 600;
            margin-top: 8px;
            opacity: 0.9;
        }}

        .section {{
            margin: 32px 0;
        }}

        .section-title {{
            font-size: 20px;
            font-weight: 700;
            color: #111827;
            margin-bottom: 16px;
            padding-bottom: 8px;
            border-bottom: 2px solid #e5e7eb;
        }}

        .stats {{
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 16px;
            margin: 16px 0;
        }}

        .stat-item {{
            background: #f9fafb;
            padding: 16px;
            border-radius: 6px;
            border-left: 4px solid #3b82f6;
        }}

        .stat-label {{
            color: #6b7280;
            font-size: 12px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 4px;
        }}

        .stat-value {{
            font-size: 24px;
            font-weight: 700;
            color: #111827;
        }}

        .vulnerability-item {{
            background: #f9fafb;
            padding: 12px 16px;
            margin: 8px 0;
            border-radius: 6px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .severity-critical {{ border-left: 4px solid #dc2626; }}
        .severity-high {{ border-left: 4px solid #ef4444; }}
        .severity-medium {{ border-left: 4px solid #f59e0b; }}
        .severity-low {{ border-left: 4px solid #64748b; }}

        .risk-list {{
            list-style: none;
        }}

        .risk-list li {{
            background: #fef2f2;
            padding: 12px;
            margin: 8px 0;
            border-radius: 6px;
            border-left: 4px solid #dc2626;
            font-size: 14px;
        }}

        .recommendations {{
            list-style: decimal;
            padding-left: 24px;
        }}

        .recommendations li {{
            margin: 12px 0;
            color: #374151;
        }}

        .footer {{
            margin-top: 48px;
            padding-top: 16px;
            border-top: 2px solid #e5e7eb;
            text-align: center;
            color: #6b7280;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="report">
        <h1>Security Summary Report</h1>
        
        <div class="metadata">
            <p><strong>Project:</strong> {}</p>
            <p><strong>Version:</strong> {}</p>
            <p><strong>Scan Date:</strong> {}</p>
        </div>

        <div class="score-card">
            <div class="score-value">{}/100</div>
            <div class="score-label">{}</div>
        </div>

        <div class="section">
            <h2 class="section-title">Vulnerability Summary</h2>
            <div class="stats">
                <div class="stat-item severity-critical">
                    <div class="stat-label">Critical</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item severity-high">
                    <div class="stat-label">High</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item severity-medium">
                    <div class="stat-label">Medium</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item severity-low">
                    <div class="stat-label">Low</div>
                    <div class="stat-value">{}</div>
                </div>
            </div>
        </div>

        <div class="section">
            <h2 class="section-title">Dependencies</h2>
            <div class="stats">
                <div class="stat-item">
                    <div class="stat-label">Total</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Direct</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Transitive</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Policy Violations</div>
                    <div class="stat-value">{}</div>
                </div>
            </div>
        </div>

        {}

        <div class="section">
            <h2 class="section-title">Recommended Actions</h2>
            <ol class="recommendations">
                {}
            </ol>
        </div>

        <div class="footer">
            Generated by BazBOM | https://github.com/cboyd0319/BazBOM
        </div>
    </div>
</body>
</html>"#,
        generator.sbom().project_name,
        generator.sbom().project_version,
        generator.sbom().scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        score,
        score_label,
        vulns.critical.len(),
        vulns.high.len(),
        vulns.medium.len(),
        vulns.low.len(),
        generator.sbom().total_dependencies,
        generator.sbom().direct_dependencies,
        generator.sbom().transitive_dependencies,
        policy.policy_violations,
        generate_top_risks_html(vulns),
        recommendations.iter().map(|r| format!("<li>{}</li>", r)).collect::<Vec<_>>().join("\n                ")
    );

    write_html_file(output_path, &html)?;
    Ok(())
}

/// Generate HTML for top risks section
fn generate_top_risks_html(vulns: &crate::VulnerabilityFindings) -> String {
    if vulns.critical.is_empty() && vulns.high.is_empty() {
        return String::new();
    }

    let mut html = String::from(r#"<div class="section">
            <h2 class="section-title">Top Risks Requiring Immediate Attention</h2>
            <ul class="risk-list">
"#);

    let mut count = 0;
    for vuln in vulns.critical.iter().take(3) {
        html.push_str(&format!(
            "                <li><strong>{}</strong> - {} ({})</li>\n",
            vuln.cve, vuln.package_name, vuln.severity
        ));
        count += 1;
    }

    for vuln in vulns.high.iter().take(5 - count) {
        html.push_str(&format!(
            "                <li><strong>{}</strong> - {} ({})</li>\n",
            vuln.cve, vuln.package_name, vuln.severity
        ));
    }

    html.push_str("            </ul>\n        </div>\n        ");
    html
}

/// Generate prioritized recommendations based on findings
fn generate_recommendations(generator: &ReportGenerator) -> Vec<String> {
    let mut recs = Vec::new();
    let vulns = generator.vulnerabilities();
    let policy = generator.policy();

    // Critical vulnerabilities
    if !vulns.critical.is_empty() {
        recs.push(format!(
            "Fix {} CRITICAL vulnerabilities immediately (potential active exploitation)",
            vulns.critical.len()
        ));
    }

    // High vulnerabilities
    if !vulns.high.is_empty() {
        recs.push(format!(
            "Address {} HIGH severity vulnerabilities within 7 days",
            vulns.high.len()
        ));
    }

    // Policy violations
    if policy.policy_violations > 0 {
        recs.push(format!(
            "Resolve {} policy violations to maintain compliance",
            policy.policy_violations
        ));
    }

    // License issues
    if policy.license_issues > 0 {
        recs.push(format!(
            "Review {} license compliance issues",
            policy.license_issues
        ));
    }

    // Medium vulnerabilities
    if !vulns.medium.is_empty() {
        recs.push(format!(
            "Schedule remediation for {} MEDIUM severity vulnerabilities",
            vulns.medium.len()
        ));
    }

    // If everything is good
    if recs.is_empty() {
        recs.push("Maintain current security posture with regular scans".to_string());
        recs.push("Consider implementing automated dependency updates".to_string());
        recs.push("Review and update security policies quarterly".to_string());
    }

    recs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyStatus, SbomData, VulnerabilityDetail, VulnerabilityFindings};
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_generator() -> ReportGenerator {
        let sbom = SbomData {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            scan_timestamp: Utc::now(),
            total_dependencies: 100,
            direct_dependencies: 20,
            transitive_dependencies: 80,
        };

        let vulnerabilities = VulnerabilityFindings {
            critical: vec![VulnerabilityDetail {
                cve: "CVE-2021-44228".to_string(),
                package_name: "log4j-core".to_string(),
                package_version: "2.14.1".to_string(),
                severity: "CRITICAL".to_string(),
                cvss_score: 10.0,
                description: "Log4Shell".to_string(),
                fixed_version: Some("2.21.1".to_string()),
                is_reachable: true,
                is_kev: true,
                epss_score: Some(0.975),
            }],
            high: vec![],
            medium: vec![],
            low: vec![],
        };

        let policy = PolicyStatus {
            policy_violations: 1,
            license_issues: 0,
            blocked_packages: 0,
        };

        ReportGenerator::new(sbom, vulnerabilities, policy)
    }

    #[test]
    fn test_recommendations_generation() {
        let generator = create_test_generator();
        let recs = generate_recommendations(&generator);

        assert!(!recs.is_empty());
        assert!(recs.iter().any(|r| r.contains("CRITICAL")));
    }

    #[test]
    fn test_executive_report_generation() {
        let generator = create_test_generator();
        let output_path = PathBuf::from("/tmp/test_executive_report.html");

        let result = generate_executive_report(&generator, &output_path);
        // Should succeed in creating the HTML
        assert!(result.is_ok());

        // Verify HTML content
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Security Summary Report"));
        assert!(content.contains("test-project"));
        assert!(content.contains("CVE-2021-44228"));

        // Clean up
        let _ = std::fs::remove_file(output_path);
    }
}
