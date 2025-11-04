//! Compliance report generation
//!
//! Generates compliance reports for specific frameworks (PCI-DSS, HIPAA, etc.)

use crate::{ComplianceFramework, ReportGenerator};
use anyhow::Result;
use std::path::Path;

/// Compliance requirement details
struct ComplianceRequirement {
    requirement_id: &'static str,
    title: &'static str,
    description: &'static str,
    controls: Vec<&'static str>,
}

/// Generate a compliance-specific HTML report
pub fn generate_compliance_report(
    generator: &ReportGenerator,
    framework: ComplianceFramework,
    output_path: &Path,
) -> Result<()> {
    let requirements = get_framework_requirements(framework);
    let html = build_compliance_html(generator, framework, &requirements);
    crate::write_html_file(output_path, &html)
}

/// Get compliance requirements for a framework
fn get_framework_requirements(framework: ComplianceFramework) -> Vec<ComplianceRequirement> {
    match framework {
        ComplianceFramework::PciDss => get_pci_dss_requirements(),
        ComplianceFramework::Hipaa => get_hipaa_requirements(),
        ComplianceFramework::FedRampModerate => get_fedramp_requirements(),
        ComplianceFramework::Soc2 => get_soc2_requirements(),
        ComplianceFramework::Gdpr => get_gdpr_requirements(),
        ComplianceFramework::Iso27001 => get_iso27001_requirements(),
        ComplianceFramework::NistCsf => get_nist_csf_requirements(),
    }
}

/// PCI-DSS v4.0 requirements relevant to software dependencies
fn get_pci_dss_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "6.2.4",
            title: "Software Engineering Techniques",
            description:
                "All software components are kept up to date and free of known vulnerabilities",
            controls: vec![
                "Maintain inventory of software components",
                "Monitor for security vulnerabilities",
                "Apply security patches within defined timeframe",
            ],
        },
        ComplianceRequirement {
            requirement_id: "6.3.2",
            title: "Secure Development Practices",
            description: "Review custom code and third-party components for vulnerabilities",
            controls: vec![
                "Use SBOM to track dependencies",
                "Scan for known vulnerabilities",
                "Document remediation plans",
            ],
        },
        ComplianceRequirement {
            requirement_id: "11.3.2",
            title: "Vulnerability Scanning",
            description: "Perform automated vulnerability scans on deployed systems",
            controls: vec![
                "Scan at least quarterly",
                "After significant changes",
                "Address high-risk vulnerabilities",
            ],
        },
    ]
}

/// HIPAA Security Rule requirements
fn get_hipaa_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "164.308(a)(5)(ii)(B)",
            title: "Protection from Malicious Software",
            description: "Procedures for detecting and protecting against malicious software",
            controls: vec![
                "Implement vulnerability scanning",
                "Monitor for malicious packages",
                "Update software regularly",
            ],
        },
        ComplianceRequirement {
            requirement_id: "164.308(a)(8)",
            title: "Evaluation",
            description: "Perform periodic technical and non-technical evaluations",
            controls: vec![
                "Regular security assessments",
                "Dependency vulnerability analysis",
                "Document remediation actions",
            ],
        },
        ComplianceRequirement {
            requirement_id: "164.312(e)(2)(i)",
            title: "Integrity Controls",
            description:
                "Implement electronic mechanisms to corroborate that ePHI has not been altered",
            controls: vec![
                "Verify software integrity (SBOM)",
                "Monitor for unauthorized changes",
                "Maintain audit trails",
            ],
        },
    ]
}

/// FedRAMP Moderate requirements
fn get_fedramp_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "RA-5",
            title: "Vulnerability Scanning",
            description: "Scan for vulnerabilities and remediate legitimate threats",
            controls: vec![
                "Automated vulnerability scanning",
                "Analyze scan reports",
                "Remediate vulnerabilities based on risk",
            ],
        },
        ComplianceRequirement {
            requirement_id: "SI-2",
            title: "Flaw Remediation",
            description: "Identify, report, and correct information system flaws",
            controls: vec![
                "Install security-relevant updates",
                "Test updates before deployment",
                "Track flaw remediation",
            ],
        },
        ComplianceRequirement {
            requirement_id: "SA-10",
            title: "Developer Security Testing",
            description: "Require developers to perform security testing",
            controls: vec![
                "Static code analysis",
                "Dependency vulnerability scanning",
                "Document security testing results",
            ],
        },
    ]
}

/// SOC 2 Type II requirements
fn get_soc2_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "CC7.1",
            title: "System Monitoring",
            description: "Monitor system components for anomalies and vulnerabilities",
            controls: vec![
                "Continuous vulnerability monitoring",
                "Automated alerting",
                "Timely remediation",
            ],
        },
        ComplianceRequirement {
            requirement_id: "CC8.1",
            title: "Change Management",
            description: "Manage changes to system components",
            controls: vec![
                "Track dependency changes",
                "Security review before deployment",
                "Rollback capabilities",
            ],
        },
    ]
}

/// GDPR requirements
fn get_gdpr_requirements() -> Vec<ComplianceRequirement> {
    vec![ComplianceRequirement {
        requirement_id: "Article 32",
        title: "Security of Processing",
        description: "Implement appropriate technical and organizational measures",
        controls: vec![
            "Regular security assessments",
            "Vulnerability management",
            "Incident response procedures",
        ],
    }]
}

/// ISO 27001 requirements
fn get_iso27001_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "A.12.6.1",
            title: "Management of Technical Vulnerabilities",
            description: "Obtain timely information about technical vulnerabilities",
            controls: vec![
                "Vulnerability scanning",
                "Risk assessment",
                "Timely patching",
            ],
        },
        ComplianceRequirement {
            requirement_id: "A.14.2.1",
            title: "Secure Development Policy",
            description: "Rules for the development of software and systems",
            controls: vec![
                "Secure coding practices",
                "Dependency management",
                "Security testing",
            ],
        },
    ]
}

/// NIST Cybersecurity Framework requirements
fn get_nist_csf_requirements() -> Vec<ComplianceRequirement> {
    vec![
        ComplianceRequirement {
            requirement_id: "ID.AM-2",
            title: "Software Platforms and Applications",
            description: "Software platforms and applications are inventoried",
            controls: vec![
                "Maintain SBOM",
                "Track software versions",
                "Identify vulnerabilities",
            ],
        },
        ComplianceRequirement {
            requirement_id: "DE.CM-8",
            title: "Vulnerability Scans",
            description: "Vulnerability scans are performed",
            controls: vec![
                "Regular scanning",
                "Prioritize remediation",
                "Track metrics",
            ],
        },
    ]
}

/// Build HTML content for compliance report
fn build_compliance_html(
    generator: &ReportGenerator,
    framework: ComplianceFramework,
    requirements: &[ComplianceRequirement],
) -> String {
    let sbom = generator.sbom();
    let vulns = generator.vulnerabilities();
    let policy = generator.policy();

    let security_score = vulns.security_score();
    let compliance_status = determine_compliance_status(security_score, vulns.total_count());

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} Compliance Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }}
        h1 {{ color: #333; border-bottom: 3px solid #0066cc; padding-bottom: 10px; }}
        h2 {{ color: #0066cc; margin-top: 30px; }}
        h3 {{ color: #555; }}
        .header {{ background: #f4f4f4; padding: 20px; border-radius: 5px; margin-bottom: 30px; }}
        .status-pass {{ color: #28a745; font-weight: bold; }}
        .status-fail {{ color: #dc3545; font-weight: bold; }}
        .status-warning {{ color: #ffc107; font-weight: bold; }}
        .metric {{ display: inline-block; margin: 10px 20px 10px 0; }}
        .requirement {{ background: #fff; border-left: 4px solid #0066cc; padding: 15px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .control {{ padding-left: 20px; margin: 5px 0; }}
        .vulnerability-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .vulnerability-table th, .vulnerability-table td {{ padding: 12px; text-align: left; border: 1px solid #ddd; }}
        .vulnerability-table th {{ background: #f8f9fa; font-weight: bold; }}
        .critical {{ color: #dc3545; font-weight: bold; }}
        .high {{ color: #fd7e14; font-weight: bold; }}
        .footer {{ margin-top: 50px; padding-top: 20px; border-top: 1px solid #ddd; color: #666; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{} Compliance Report</h1>
        <p><strong>Project:</strong> {} v{}</p>
        <p><strong>Scan Date:</strong> {}</p>
        <p><strong>Framework:</strong> {}</p>
    </div>

    <h2>Compliance Summary</h2>
    <div class="metric">
        <strong>Overall Status:</strong> <span class="{}">{}</span>
    </div>
    <div class="metric">
        <strong>Security Score:</strong> {}/100
    </div>
    <div class="metric">
        <strong>Total Dependencies:</strong> {}
    </div>
    <div class="metric">
        <strong>Vulnerabilities:</strong> {}
    </div>
    <div class="metric">
        <strong>Policy Violations:</strong> {}
    </div>

    <h2>Vulnerability Summary</h2>
    <table class="vulnerability-table">
        <thead>
            <tr>
                <th>Severity</th>
                <th>Count</th>
                <th>Status</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td class="critical">CRITICAL</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            <tr>
                <td class="high">HIGH</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>MEDIUM</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>LOW</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
        </tbody>
    </table>

    <h2>Compliance Requirements</h2>
    <p>This report assesses compliance with {} requirements based on dependency vulnerability analysis.</p>
    {}

    <h2>Recommendations</h2>
    <ul>
        {}
    </ul>

    <div class="footer">
        <p>Generated by BazBOM | Framework: {} | Report Date: {}</p>
        <p>This report should be reviewed by compliance officers and security teams.</p>
    </div>
</body>
</html>"#,
        framework.name(),
        sbom.project_name,
        framework.name(),
        sbom.project_name,
        sbom.project_version,
        sbom.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        framework.name(),
        if compliance_status == "PASS" {
            "status-pass"
        } else if compliance_status == "FAIL" {
            "status-fail"
        } else {
            "status-warning"
        },
        compliance_status,
        security_score,
        sbom.total_dependencies,
        vulns.total_count(),
        policy.policy_violations,
        vulns.critical.len(),
        if vulns.critical.is_empty() {
            "✅ PASS"
        } else {
            "❌ FAIL"
        },
        vulns.high.len(),
        if vulns.high.is_empty() {
            "✅ PASS"
        } else {
            "⚠️ WARNING"
        },
        vulns.medium.len(),
        if vulns.medium.len() <= 5 {
            "✅ PASS"
        } else {
            "⚠️ WARNING"
        },
        vulns.low.len(),
        "✅ PASS",
        framework.name(),
        build_requirements_html(requirements),
        build_recommendations_html(vulns, policy),
        framework.name(),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
    )
}

/// Determine overall compliance status
fn determine_compliance_status(security_score: u32, vuln_count: usize) -> &'static str {
    if security_score >= 90 && vuln_count == 0 {
        "PASS"
    } else if security_score >= 70 {
        "WARNING"
    } else {
        "FAIL"
    }
}

/// Build HTML for requirements section
fn build_requirements_html(requirements: &[ComplianceRequirement]) -> String {
    requirements
        .iter()
        .map(|req| {
            let controls_html = req
                .controls
                .iter()
                .map(|c| format!("<div class=\"control\">• {}</div>", c))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                r#"<div class="requirement">
        <h3>{} - {}</h3>
        <p>{}</p>
        <p><strong>Required Controls:</strong></p>
        {}
    </div>"#,
                req.requirement_id, req.title, req.description, controls_html
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build recommendations based on findings
fn build_recommendations_html(
    vulns: &crate::VulnerabilityFindings,
    policy: &crate::PolicyStatus,
) -> String {
    let mut recommendations = Vec::new();

    if !vulns.critical.is_empty() {
        recommendations.push("<li><strong>URGENT:</strong> Remediate CRITICAL vulnerabilities immediately. These pose significant security risks.</li>");
    }

    if !vulns.high.is_empty() {
        recommendations.push("<li>Address HIGH severity vulnerabilities within 30 days per compliance requirements.</li>");
    }

    if vulns.medium.len() > 5 {
        recommendations.push("<li>Review and remediate MEDIUM severity vulnerabilities to improve security posture.</li>");
    }

    if policy.policy_violations > 0 {
        recommendations.push("<li>Resolve policy violations to maintain compliance.</li>");
    }

    if vulns.total_count() == 0 {
        recommendations.push("<li>✅ Excellent! No vulnerabilities detected. Maintain regular scanning schedule.</li>");
    }

    recommendations
        .push("<li>Schedule regular SBOM scans (at least quarterly) to maintain compliance.</li>");
    recommendations.push("<li>Document all remediation actions for audit purposes.</li>");

    recommendations.join("\n        ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyStatus, SbomData, VulnerabilityFindings};
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_compliance_report_stub() {
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

        let output = PathBuf::from("/tmp/test_compliance.html");
        let result = generate_compliance_report(&generator, ComplianceFramework::PciDss, &output);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(output);
    }
}
