//! Trend report generation
//!
//! Generates historical trend analysis reports

use crate::ReportGenerator;
use anyhow::Result;
use std::path::Path;

/// Generate a trend analysis HTML report
pub fn generate_trend_report(generator: &ReportGenerator, output_path: &Path) -> Result<()> {
    let html = build_trend_html(generator);
    crate::write_html_file(output_path, &html)
}

/// Build HTML content for trend report
fn build_trend_html(generator: &ReportGenerator) -> String {
    let sbom = generator.sbom();
    let vulns = generator.vulnerabilities();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Security Trend Report - {}</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 40px; border-radius: 12px; box-shadow: 0 8px 32px rgba(0,0,0,0.2); }}
        h1 {{ color: #2d3748; margin-bottom: 10px; font-size: 2.5em; }}
        h2 {{ color: #4a5568; margin-top: 40px; padding-bottom: 10px; border-bottom: 3px solid #667eea; }}
        .subtitle {{ color: #718096; font-size: 1.1em; margin-bottom: 30px; }}
        .metrics-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; margin: 30px 0; }}
        .metric-card {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 25px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .metric-card h3 {{ margin: 0 0 10px 0; font-size: 0.9em; text-transform: uppercase; letter-spacing: 1px; opacity: 0.9; }}
        .metric-card .value {{ font-size: 2.5em; font-weight: bold; margin: 10px 0; }}
        .metric-card .change {{ font-size: 0.9em; opacity: 0.9; }}
        .chart-placeholder {{ background: #f7fafc; border: 2px dashed #cbd5e0; border-radius: 8px; padding: 60px 20px; text-align: center; color: #a0aec0; margin: 20px 0; }}
        .insight-box {{ background: #edf2f7; border-left: 4px solid #667eea; padding: 20px; margin: 20px 0; border-radius: 4px; }}
        .insight-box h3 {{ color: #2d3748; margin-top: 0; }}
        .recommendation {{ background: #ffffff; border: 1px solid #e2e8f0; padding: 15px; margin: 10px 0; border-radius: 6px; box-shadow: 0 2px 4px rgba(0,0,0,0.05); }}
        .recommendation::before {{ content: "[i] "; font-size: 1.2em; }}
        .footer {{ margin-top: 60px; padding-top: 20px; border-top: 2px solid #e2e8f0; color: #718096; text-align: center; }}
        .note {{ background: #fef5e7; border-left: 4px solid #f39c12; padding: 15px; margin: 20px 0; border-radius: 4px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Report: Security Trend Report</h1>
        <div class="subtitle">Project: {} v{} | Scan Date: {}</div>

        <div class="note">
            <strong>📌 Note:</strong> This is a snapshot report. Historical trend analysis requires multiple scans over time. 
            Run regular scans to build comprehensive trend data.
        </div>

        <h2>Current Security Metrics</h2>
        <div class="metrics-grid">
            <div class="metric-card">
                <h3>Security Score</h3>
                <div class="value">{}/100</div>
                <div class="change">Current snapshot</div>
            </div>
            <div class="metric-card">
                <h3>Total Vulnerabilities</h3>
                <div class="value">{}</div>
                <div class="change">{} Critical · {} High</div>
            </div>
            <div class="metric-card">
                <h3>Dependencies</h3>
                <div class="value">{}</div>
                <div class="change">{} Direct · {} Transitive</div>
            </div>
            <div class="metric-card">
                <h3>Risk Level</h3>
                <div class="value">{}</div>
                <div class="change">Based on current findings</div>
            </div>
        </div>

        <h2>Vulnerability Trends (Preview)</h2>
        <div class="chart-placeholder">
            <strong>📈 Historical Chart</strong><br>
            <p style="margin-top: 10px;">Vulnerability trends will appear here after multiple scans.</p>
            <p>Run <code>bazbom scan</code> regularly to track changes over time.</p>
        </div>

        <h2>Security Insights</h2>
        <div class="insight-box">
            <h3>Current State Analysis</h3>
            {}
        </div>

        <h2>Recommended Actions</h2>
        {}

        <h2>Future Trend Analysis</h2>
        <div class="insight-box">
            <h3>What to Expect</h3>
            <p>After accumulating scan history, this report will show:</p>
            <ul>
                <li><strong>Vulnerability Introduction Rate:</strong> How many new vulnerabilities appear per scan</li>
                <li><strong>Mean Time to Fix (MTTF):</strong> Average time to remediate vulnerabilities</li>
                <li><strong>Remediation Velocity:</strong> How quickly your team addresses security issues</li>
                <li><strong>Dependency Growth:</strong> How your dependency count changes over time</li>
                <li><strong>Security Score Trends:</strong> Whether your security posture is improving</li>
                <li><strong>Repeat Offenders:</strong> Dependencies that frequently have vulnerabilities</li>
            </ul>
        </div>

        <div class="footer">
            <p>Generated by BazBOM | Scan Date: {}</p>
            <p>Schedule regular scans to build comprehensive trend data for better security insights.</p>
        </div>
    </div>
</body>
</html>"#,
        sbom.project_name,
        sbom.project_name,
        sbom.project_version,
        sbom.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        vulns.security_score(),
        vulns.total_count(),
        vulns.critical.len(),
        vulns.high.len(),
        sbom.total_dependencies,
        sbom.direct_dependencies,
        sbom.transitive_dependencies,
        calculate_risk_level(vulns),
        build_current_state_analysis(vulns),
        build_recommendations_list(vulns),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
    )
}

/// Calculate risk level based on vulnerabilities
fn calculate_risk_level(vulns: &crate::VulnerabilityFindings) -> &'static str {
    if !vulns.critical.is_empty() {
        "CRITICAL"
    } else if !vulns.high.is_empty() {
        "HIGH"
    } else if vulns.medium.len() > 5 {
        "MEDIUM"
    } else {
        "LOW"
    }
}

/// Build current state analysis text
fn build_current_state_analysis(vulns: &crate::VulnerabilityFindings) -> String {
    if vulns.total_count() == 0 {
        return "<p>[+] <strong>Excellent security posture!</strong> No vulnerabilities detected in your dependencies. Continue maintaining this high standard with regular scans.</p>".to_string();
    }

    let mut analysis = Vec::new();

    if !vulns.critical.is_empty() {
        analysis.push(format!(
            "<p>🚨 <strong>Critical Alert:</strong> {} CRITICAL vulnerabilities require immediate attention. These pose severe security risks and should be addressed within 24 hours.</p>",
            vulns.critical.len()
        ));
    }

    if !vulns.high.is_empty() {
        analysis.push(format!(
            "<p>[!] <strong>High Priority:</strong> {} HIGH severity vulnerabilities detected. Industry best practice recommends remediation within 30 days.</p>",
            vulns.high.len()
        ));
    }

    if !vulns.medium.is_empty() {
        analysis.push(format!(
            "<p>[*] <strong>Medium Priority:</strong> {} MEDIUM severity vulnerabilities found. Plan remediation within 90 days as part of regular maintenance.</p>",
            vulns.medium.len()
        ));
    }

    let kev_count = vulns
        .critical
        .iter()
        .chain(vulns.high.iter())
        .chain(vulns.medium.iter())
        .chain(vulns.low.iter())
        .filter(|v| v.is_kev)
        .count();

    if kev_count > 0 {
        analysis.push(format!(
            "<p>[*] <strong>CISA KEV Alert:</strong> {} vulnerabilities are listed in CISA's Known Exploited Vulnerabilities catalog, indicating active exploitation in the wild.</p>",
            kev_count
        ));
    }

    let reachable_count = vulns
        .critical
        .iter()
        .chain(vulns.high.iter())
        .chain(vulns.medium.iter())
        .chain(vulns.low.iter())
        .filter(|v| v.is_reachable)
        .count();

    if reachable_count > 0 {
        analysis.push(format!(
            "<p>[*] <strong>Reachability Analysis:</strong> {} vulnerabilities have reachable code paths in your application, increasing actual risk.</p>",
            reachable_count
        ));
    }

    analysis.join("\n")
}

/// Build recommendations list
fn build_recommendations_list(vulns: &crate::VulnerabilityFindings) -> String {
    let mut recommendations = Vec::new();

    if !vulns.critical.is_empty() {
        recommendations.push(
            r#"<div class="recommendation">
            <strong>Address Critical Vulnerabilities Immediately</strong><br>
            Run <code>bazbom fix --apply</code> to automatically upgrade affected dependencies.
        </div>"#,
        );
    }

    if !vulns.high.is_empty() {
        recommendations.push(r#"<div class="recommendation">
            <strong>Schedule High-Priority Remediation</strong><br>
            Create a remediation plan with <code>bazbom fix --suggest</code> to see detailed fix instructions.
        </div>"#);
    }

    recommendations.push(r#"<div class="recommendation">
        <strong>Enable Pre-Commit Hooks</strong><br>
        Run <code>bazbom install-hooks</code> to catch vulnerabilities before they enter your codebase.
    </div>"#);

    recommendations.push(r#"<div class="recommendation">
        <strong>Schedule Regular Scans</strong><br>
        Set up weekly or monthly automated scans to track trends and catch new vulnerabilities early.
    </div>"#);

    recommendations.push(r#"<div class="recommendation">
        <strong>Integrate with CI/CD</strong><br>
        Add BazBOM to your CI/CD pipeline to automatically scan on every pull request and deployment.
    </div>"#);

    if vulns.total_count() == 0 {
        recommendations.push(
            r#"<div class="recommendation">
            <strong>Maintain Your Security Posture</strong><br>
            Continue regular scanning and stay updated with the latest security advisories.
        </div>"#,
        );
    }

    recommendations.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyStatus, SbomData, VulnerabilityFindings};
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_trend_report_stub() {
        let generator = ReportGenerator::new(
            SbomData {
                project_name: "test".to_string(),
                project_version: "1.0.0".to_string(),
                scan_timestamp: Utc::now(),
                total_dependencies: 10,
                direct_dependencies: 5,
                transitive_dependencies: 5,
            },
            VulnerabilityFindings {
                critical: vec![],
                high: vec![],
                medium: vec![],
                low: vec![],
            },
            PolicyStatus {
                policy_violations: 0,
                license_issues: 0,
                blocked_packages: 0,
            },
        );

        let output = PathBuf::from("/tmp/test_trend.html");
        let result = generate_trend_report(&generator, &output);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(output);
    }
}
