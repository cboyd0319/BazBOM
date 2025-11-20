//! Developer report generation
//!
//! Generates detailed reports for developers with remediation instructions

use crate::{ReportGenerator, VulnerabilityDetail};
use anyhow::Result;
use std::path::Path;

/// Generate a developer-focused HTML report with detailed vulnerability information
pub fn generate_developer_report(generator: &ReportGenerator, output_path: &Path) -> Result<()> {
    let html = build_developer_html(generator);
    crate::write_html_file(output_path, &html)
}

/// Build HTML content for developer report
fn build_developer_html(generator: &ReportGenerator) -> String {
    let sbom = generator.sbom();
    let vulns = generator.vulnerabilities();
    let policy = generator.policy();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Developer Security Report - {}</title>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}
        h1 {{ color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; margin-bottom: 30px; }}
        h2 {{ color: #3498db; margin-top: 40px; border-bottom: 2px solid #ecf0f1; padding-bottom: 8px; }}
        h3 {{ color: #34495e; margin-top: 25px; }}
        .summary {{ background: #ecf0f1; padding: 20px; border-radius: 5px; margin-bottom: 30px; }}
        .metric {{ display: inline-block; margin: 10px 25px 10px 0; padding: 10px; }}
        .metric strong {{ color: #2c3e50; }}
        .vulnerability {{ background: #fff; border-left: 5px solid #e74c3c; padding: 20px; margin: 20px 0; border-radius: 5px; box-shadow: 0 2px 4px rgba(0,0,0,0.08); }}
        .vulnerability.critical {{ border-left-color: #e74c3c; background: #fdf2f2; }}
        .vulnerability.high {{ border-left-color: #f39c12; background: #fef9f2; }}
        .vulnerability.medium {{ border-left-color: #f1c40f; background: #fffef2; }}
        .vulnerability.low {{ border-left-color: #95a5a6; background: #f9f9f9; }}
        .vuln-header {{ display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 15px; }}
        .vuln-title {{ font-size: 1.2em; font-weight: bold; color: #2c3e50; }}
        .severity-badge {{ padding: 5px 12px; border-radius: 3px; color: white; font-weight: bold; font-size: 0.85em; }}
        .severity-badge.critical {{ background: #e74c3c; }}
        .severity-badge.high {{ background: #f39c12; }}
        .severity-badge.medium {{ background: #f1c40f; color: #333; }}
        .severity-badge.low {{ background: #95a5a6; }}
        .vuln-meta {{ display: flex; gap: 20px; margin: 10px 0; flex-wrap: wrap; }}
        .meta-item {{ display: inline-flex; align-items: center; gap: 5px; color: #7f8c8d; }}
        .badge {{ padding: 3px 8px; border-radius: 3px; font-size: 0.8em; font-weight: bold; }}
        .badge.kev {{ background: #e74c3c; color: white; }}
        .badge.reachable {{ background: #e67e22; color: white; }}
        .code-block {{ background: #2c3e50; color: #ecf0f1; padding: 15px; border-radius: 5px; margin: 15px 0; overflow-x: auto; font-family: 'Courier New', monospace; font-size: 0.9em; }}
        .fix-instruction {{ background: #e8f5e9; border-left: 4px solid #4caf50; padding: 15px; margin: 15px 0; border-radius: 3px; }}
        .fix-instruction strong {{ color: #2e7d32; }}
        .warning-box {{ background: #fff3cd; border-left: 4px solid #ffc107; padding: 15px; margin: 15px 0; border-radius: 3px; }}
        .info-box {{ background: #e3f2fd; border-left: 4px solid #2196f3; padding: 15px; margin: 15px 0; border-radius: 3px; }}
        .no-vulns {{ text-align: center; padding: 40px; color: #27ae60; }}
        .no-vulns-icon {{ font-size: 4em; }}
        ul {{ line-height: 1.8; }}
        .footer {{ margin-top: 50px; padding-top: 20px; border-top: 2px solid #ecf0f1; color: #7f8c8d; text-align: center; }}

        /* Interactive controls */
        .controls {{ background: #34495e; padding: 15px 20px; border-radius: 5px; margin-bottom: 30px; display: flex; flex-wrap: wrap; gap: 15px; align-items: center; }}
        .controls label {{ color: #ecf0f1; font-weight: bold; margin-right: 8px; }}
        .controls select, .controls input {{ padding: 8px 12px; border-radius: 4px; border: none; font-size: 0.95em; }}
        .controls input[type="text"] {{ width: 250px; }}
        .filter-btn {{ padding: 8px 15px; border-radius: 4px; border: none; cursor: pointer; font-weight: bold; transition: background 0.2s; }}
        .filter-btn.active {{ background: #3498db; color: white; }}
        .filter-btn:not(.active) {{ background: #bdc3c7; color: #2c3e50; }}
        .filter-btn:hover {{ opacity: 0.9; }}

        /* Collapsible */
        .vuln-details {{ max-height: 0; overflow: hidden; transition: max-height 0.3s ease-out; }}
        .vulnerability.expanded .vuln-details {{ max-height: 1000px; }}
        .expand-toggle {{ cursor: pointer; color: #3498db; font-size: 0.9em; margin-top: 10px; user-select: none; }}
        .expand-toggle:hover {{ text-decoration: underline; }}

        /* Hidden state */
        .hidden {{ display: none !important; }}

        /* Stats bar */
        .stats-bar {{ background: #2c3e50; color: #ecf0f1; padding: 10px 20px; border-radius: 5px; margin-bottom: 20px; display: flex; justify-content: space-between; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Security: Developer Security Report</h1>
        
        <div class="summary">
            <h2>Project Summary</h2>
            <div class="metric">
                <strong>Project:</strong> {} v{}
            </div>
            <div class="metric">
                <strong>Scan Date:</strong> {}
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
        </div>

        <div class="controls">
            <div>
                <label>Filter:</label>
                <button class="filter-btn active" onclick="filterBySeverity('all')">All</button>
                <button class="filter-btn" onclick="filterBySeverity('critical')">Critical</button>
                <button class="filter-btn" onclick="filterBySeverity('high')">High</button>
                <button class="filter-btn" onclick="filterBySeverity('medium')">Medium</button>
                <button class="filter-btn" onclick="filterBySeverity('low')">Low</button>
            </div>
            <div>
                <button class="filter-btn" onclick="filterByFixable(true)">Fixable Only</button>
                <button class="filter-btn" onclick="filterByFixable(false)">No Fix</button>
            </div>
            <div>
                <label>Search:</label>
                <input type="text" id="searchInput" placeholder="Search CVE, package..." oninput="searchVulns(this.value)">
            </div>
            <div>
                <button class="filter-btn" onclick="toggleAll(true)">Expand All</button>
                <button class="filter-btn" onclick="toggleAll(false)">Collapse All</button>
            </div>
        </div>

        <div class="stats-bar">
            <span id="visibleCount">Showing all vulnerabilities</span>
            <span id="filterStatus"></span>
        </div>

        <div id="vulnContainer">
        {}
        </div>

        <div class="footer">
            <p>Generated by BazBOM | Scan Date: {}</p>
            <p>For support and documentation: <a href="https://github.com/cboyd0319/BazBOM">github.com/cboyd0319/BazBOM</a></p>
        </div>
    </div>

    <script>
        let currentSeverityFilter = 'all';
        let currentFixableFilter = null;
        let currentSearch = '';

        function filterBySeverity(severity) {{
            currentSeverityFilter = severity;
            // Update button states
            document.querySelectorAll('.controls > div:first-child .filter-btn').forEach(btn => {{
                btn.classList.toggle('active', btn.textContent.toLowerCase() === severity);
            }});
            applyFilters();
        }}

        function filterByFixable(hasfix) {{
            // Toggle - if already selected, clear it
            if (currentFixableFilter === hasfix) {{
                currentFixableFilter = null;
                document.querySelectorAll('.controls > div:nth-child(2) .filter-btn').forEach(btn => {{
                    btn.classList.remove('active');
                }});
            }} else {{
                currentFixableFilter = hasfix;
                document.querySelectorAll('.controls > div:nth-child(2) .filter-btn').forEach(btn => {{
                    btn.classList.toggle('active', (btn.textContent === 'Fixable Only') === hasfix);
                }});
            }}
            applyFilters();
        }}

        function searchVulns(query) {{
            currentSearch = query.toLowerCase();
            applyFilters();
        }}

        function applyFilters() {{
            const vulns = document.querySelectorAll('.vulnerability');
            let visible = 0;
            let total = vulns.length;

            vulns.forEach(vuln => {{
                let show = true;

                // Severity filter
                if (currentSeverityFilter !== 'all') {{
                    show = show && vuln.classList.contains(currentSeverityFilter);
                }}

                // Fixable filter
                if (currentFixableFilter !== null) {{
                    const hasFix = vuln.querySelector('.fix-instruction') !== null;
                    show = show && (hasFix === currentFixableFilter);
                }}

                // Search filter
                if (currentSearch) {{
                    const text = vuln.textContent.toLowerCase();
                    show = show && text.includes(currentSearch);
                }}

                vuln.classList.toggle('hidden', !show);
                if (show) visible++;
            }});

            // Update stats
            document.getElementById('visibleCount').textContent =
                visible === total ? `Showing all ${{total}} vulnerabilities` :
                `Showing ${{visible}} of ${{total}} vulnerabilities`;

            // Update filter status
            let filters = [];
            if (currentSeverityFilter !== 'all') filters.push(currentSeverityFilter.toUpperCase());
            if (currentFixableFilter === true) filters.push('Fixable');
            if (currentFixableFilter === false) filters.push('No Fix');
            if (currentSearch) filters.push(`Search: "${{currentSearch}}"`);
            document.getElementById('filterStatus').textContent = filters.length ? `Filters: ${{filters.join(', ')}}` : '';
        }}

        function toggleAll(expand) {{
            document.querySelectorAll('.vulnerability').forEach(vuln => {{
                vuln.classList.toggle('expanded', expand);
                const toggle = vuln.querySelector('.expand-toggle');
                if (toggle) toggle.textContent = expand ? '[-] Collapse' : '[+] Expand';
            }});
        }}

        function toggleVuln(element) {{
            const vuln = element.closest('.vulnerability');
            vuln.classList.toggle('expanded');
            element.textContent = vuln.classList.contains('expanded') ? '[-] Collapse' : '[+] Expand';
        }}

        // Keyboard shortcuts
        document.addEventListener('keydown', function(e) {{
            if (e.key === '/' && document.activeElement.tagName !== 'INPUT') {{
                e.preventDefault();
                document.getElementById('searchInput').focus();
            }}
            if (e.key === 'Escape') {{
                document.getElementById('searchInput').value = '';
                currentSearch = '';
                applyFilters();
            }}
        }});
    </script>
</body>
</html>"#,
        sbom.project_name,
        sbom.project_name,
        sbom.project_version,
        sbom.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
        sbom.total_dependencies,
        vulns.total_count(),
        policy.policy_violations,
        build_vulnerabilities_html(vulns),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
    )
}

/// Build HTML for all vulnerabilities
fn build_vulnerabilities_html(vulns: &crate::VulnerabilityFindings) -> String {
    if vulns.total_count() == 0 {
        return r#"<div class="no-vulns">
            <div class="no-vulns-icon">[+]</div>
            <h2>No Vulnerabilities Detected!</h2>
            <p>Your project is free of known vulnerabilities. Great job!</p>
            <p>Continue to scan regularly to maintain security.</p>
        </div>"#
            .to_string();
    }

    let mut sections = Vec::new();

    if !vulns.critical.is_empty() {
        sections.push(format!(
            "<h2>[!!] Critical Vulnerabilities ({}) - IMMEDIATE ACTION REQUIRED</h2>\n{}",
            vulns.critical.len(),
            vulns
                .critical
                .iter()
                .map(|v| build_vulnerability_html(v, "critical"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    if !vulns.high.is_empty() {
        sections.push(format!(
            "<h2>[!] High Severity Vulnerabilities ({}) - FIX WITHIN 30 DAYS</h2>\n{}",
            vulns.high.len(),
            vulns
                .high
                .iter()
                .map(|v| build_vulnerability_html(v, "high"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    if !vulns.medium.is_empty() {
        sections.push(format!(
            "<h2>[*] Medium Severity Vulnerabilities ({})</h2>\n{}",
            vulns.medium.len(),
            vulns
                .medium
                .iter()
                .map(|v| build_vulnerability_html(v, "medium"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    if !vulns.low.is_empty() {
        sections.push(format!(
            "<h2>[i] Low Severity Vulnerabilities ({})</h2>\n{}",
            vulns.low.len(),
            vulns
                .low
                .iter()
                .map(|v| build_vulnerability_html(v, "low"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }

    sections.join("\n")
}

/// Build HTML for a single vulnerability
fn build_vulnerability_html(vuln: &VulnerabilityDetail, severity_class: &str) -> String {
    let kev_badge = if vuln.is_kev {
        r#"<span class="badge kev">CISA KEV</span>"#
    } else {
        ""
    };
    let reachable_badge = if vuln.is_reachable {
        r#"<span class="badge reachable">REACHABLE</span>"#
    } else {
        ""
    };
    let epss_text = vuln
        .epss_score
        .map(|score| format!("EPSS: {:.1}%", score * 100.0))
        .unwrap_or_default();

    let fix_section = if let Some(ref fixed_version) = vuln.fixed_version {
        format!(
            r#"<div class="fix-instruction">
            <strong>[+] Fix Available:</strong> Upgrade to version <code>{}</code>
            <div class="code-block">
# Maven (pom.xml)
&lt;dependency&gt;
    &lt;groupId&gt;[group]&lt;/groupId&gt;
    &lt;artifactId&gt;{}&lt;/artifactId&gt;
    &lt;version&gt;{}&lt;/version&gt;
&lt;/dependency&gt;

# Gradle (build.gradle)
implementation '{}:{}'

# Or use BazBOM auto-fix:
bazbom fix --apply
            </div>
        </div>"#,
            fixed_version, vuln.package_name, fixed_version, vuln.package_name, fixed_version
        )
    } else {
        r#"<div class="warning-box">
            <strong>[!] No Fix Available Yet</strong>
            <p>Consider:</p>
            <ul>
                <li>Using an alternative package</li>
                <li>Implementing mitigating controls</li>
                <li>Monitoring for updates</li>
            </ul>
        </div>"#
            .to_string()
    };

    format!(
        r#"<div class="vulnerability {}" data-fixable="{}">
        <div class="vuln-header">
            <div>
                <div class="vuln-title">{} - {}</div>
                <div class="vuln-meta">
                    <span class="meta-item">[Pkg] {}@{}</span>
                    <span class="meta-item">[Score] CVSS: {}</span>
                    <span class="meta-item">{}</span>
                    {} {}
                </div>
            </div>
            <span class="severity-badge {}">{}</span>
        </div>

        <div class="expand-toggle" onclick="toggleVuln(this)">[+] Expand</div>

        <div class="vuln-details">
            <p>{}</p>

            {}

            <div class="info-box">
                <strong>[Refs] References:</strong>
                <ul>
                    <li><a href="https://nvd.nist.gov/vuln/detail/{}" target="_blank">NVD Entry</a></li>
                    <li><a href="https://osv.dev/vulnerability/{}" target="_blank">OSV Database</a></li>
                </ul>
            </div>
        </div>
    </div>"#,
        severity_class,
        vuln.fixed_version.is_some(),
        vuln.cve,
        vuln.package_name,
        vuln.package_name,
        vuln.package_version,
        vuln.cvss_score,
        epss_text,
        kev_badge,
        reachable_badge,
        severity_class,
        vuln.severity,
        vuln.description,
        fix_section,
        vuln.cve,
        vuln.cve
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PolicyStatus, SbomData, VulnerabilityFindings};
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_developer_report_stub() {
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

        let output = PathBuf::from("/tmp/test_developer.html");
        let result = generate_developer_report(&generator, &output);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(output);
    }
}
