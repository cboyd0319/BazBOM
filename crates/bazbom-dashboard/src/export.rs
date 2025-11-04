//! Static HTML export functionality
//!
//! Generates self-contained HTML files that can be shared without running a server.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::models::{DashboardSummary, DependencyGraph};

/// Vulnerability for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve: String,
    pub package_name: String,
    pub package_version: String,
    pub severity: String,
    pub cvss: f32,
    pub description: Option<String>,
    pub fixed_version: Option<String>,
}

/// Export dashboard to a single static HTML file
pub fn export_to_html(
    output_path: &Path,
    summary: &DashboardSummary,
    graph_data: &DependencyGraph,
    vulnerabilities: &[Vulnerability],
) -> Result<()> {
    let html = generate_html(summary, graph_data, vulnerabilities)?;
    
    fs::write(output_path, html)
        .with_context(|| format!("Failed to write HTML export to {:?}", output_path))?;
    
    Ok(())
}

/// Generate complete HTML with embedded data and JavaScript
fn generate_html(
    summary: &DashboardSummary,
    graph_data: &DependencyGraph,
    vulnerabilities: &[Vulnerability],
) -> Result<String> {
    let summary_json = serde_json::to_string_pretty(summary)?;
    let graph_json = serde_json::to_string_pretty(graph_data)?;
    let vulns_json = serde_json::to_string_pretty(vulnerabilities)?;
    
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BazBOM Security Report</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        
        header {{
            text-align: center;
            color: white;
            margin-bottom: 40px;
        }}
        
        h1 {{
            font-size: 3em;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.2);
        }}
        
        .tagline {{
            font-size: 1.2em;
            opacity: 0.9;
        }}
        
        .dashboard {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.1);
            padding: 30px;
        }}
        
        .stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        
        .stat-card {{
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            border-radius: 8px;
            padding: 20px;
            text-align: center;
        }}
        
        .stat-value {{
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
            margin: 10px 0;
        }}
        
        .stat-label {{
            font-size: 0.9em;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .critical {{ color: #e53e3e; }}
        .high {{ color: #ed8936; }}
        .medium {{ color: #ecc94b; }}
        .low {{ color: #48bb78; }}
        
        .section {{
            margin-bottom: 30px;
        }}
        
        .section-title {{
            font-size: 1.5em;
            margin-bottom: 15px;
            color: #667eea;
            border-bottom: 2px solid #e2e8f0;
            padding-bottom: 10px;
        }}
        
        .vuln-list {{
            list-style: none;
        }}
        
        .vuln-item {{
            background: #f7fafc;
            border-left: 4px solid #cbd5e0;
            padding: 15px;
            margin-bottom: 10px;
            border-radius: 4px;
        }}
        
        .vuln-item.critical {{ border-left-color: #e53e3e; }}
        .vuln-item.high {{ border-left-color: #ed8936; }}
        .vuln-item.medium {{ border-left-color: #ecc94b; }}
        .vuln-item.low {{ border-left-color: #48bb78; }}
        
        .vuln-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }}
        
        .vuln-package {{
            font-weight: bold;
            font-size: 1.1em;
        }}
        
        .vuln-badge {{
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.8em;
            font-weight: bold;
            text-transform: uppercase;
        }}
        
        .badge-critical {{
            background: #fee;
            color: #e53e3e;
        }}
        
        .badge-high {{
            background: #fed;
            color: #ed8936;
        }}
        
        .badge-medium {{
            background: #fec;
            color: #ecc94b;
        }}
        
        .badge-low {{
            background: #efe;
            color: #48bb78;
        }}
        
        .vuln-details {{
            color: #666;
            margin-top: 8px;
        }}
        
        .export-info {{
            margin-top: 40px;
            padding: 20px;
            background: #f7fafc;
            border-radius: 8px;
            text-align: center;
            color: #666;
            font-size: 0.9em;
        }}
        
        .export-info a {{
            color: #667eea;
            text-decoration: none;
        }}
        
        .export-info a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>BazBOM Security Report</h1>
            <p class="tagline">JVM SBOM, SCA, and Dependency Analysis</p>
        </header>
        
        <div class="dashboard">
            <div class="stats">
                <div class="stat-card">
                    <div class="stat-label">Security Score</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Total Dependencies</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label critical">Critical</div>
                    <div class="stat-value critical">{}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label high">High</div>
                    <div class="stat-value high">{}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label medium">Medium</div>
                    <div class="stat-value medium">{}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label low">Low</div>
                    <div class="stat-value low">{}</div>
                </div>
            </div>
            
            <div class="section">
                <h2 class="section-title">Vulnerabilities</h2>
                <ul class="vuln-list" id="vulnList">
                    <!-- Populated by JavaScript -->
                </ul>
            </div>
            
            <div class="export-info">
                <p>Generated by <a href="https://github.com/cboyd0319/BazBOM" target="_blank">BazBOM</a> - Open Source JVM SBOM & SCA Tool</p>
                <p>Report generated: {}</p>
            </div>
        </div>
    </div>
    
    <script>
        // Embedded data
        const summary = {};
        const graphData = {};
        const vulnerabilities = {};
        
        // Populate vulnerabilities list
        function populateVulnerabilities() {{
            const vulnList = document.getElementById('vulnList');
            
            if (vulnerabilities.length === 0) {{
                vulnList.innerHTML = '<li style="text-align: center; color: #48bb78; padding: 20px;">No vulnerabilities found!</li>';
                return;
            }}
            
            vulnerabilities.forEach(vuln => {{
                const li = document.createElement('li');
                li.className = `vuln-item ${{vuln.severity.toLowerCase()}}`;
                
                const severityBadge = `<span class="vuln-badge badge-${{vuln.severity.toLowerCase()}}">${{vuln.severity}}</span>`;
                
                li.innerHTML = `
                    <div class="vuln-header">
                        <div class="vuln-package">${{vuln.package_name}}:${{vuln.package_version}}</div>
                        ${{severityBadge}}
                    </div>
                    <div class="vuln-details">
                        <strong>${{vuln.cve}}</strong> - CVSS: ${{vuln.cvss}}<br>
                        ${{vuln.description || 'No description available'}}<br>
                        ${{vuln.fixed_version ? `<strong>Fix:</strong> Upgrade to version ${{vuln.fixed_version}}` : '<em>No fix available</em>'}}
                    </div>
                `;
                
                vulnList.appendChild(li);
            }});
        }}
        
        // Initialize on page load
        document.addEventListener('DOMContentLoaded', () => {{
            populateVulnerabilities();
        }});
    </script>
</body>
</html>"#,
        summary.security_score,
        summary.total_dependencies,
        summary.vulnerabilities.critical,
        summary.vulnerabilities.high,
        summary.vulnerabilities.medium,
        summary.vulnerabilities.low,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        summary_json,
        graph_json,
        vulns_json
    );
    
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::VulnerabilityCounts;
    
    #[test]
    fn test_generate_html() {
        let summary = DashboardSummary {
            security_score: 85,
            total_dependencies: 127,
            vulnerabilities: VulnerabilityCounts {
                critical: 1,
                high: 3,
                medium: 5,
                low: 2,
            },
            license_issues: 0,
            policy_violations: 0,
        };
        
        let graph_data = DependencyGraph {
            nodes: vec![],
            edges: vec![],
        };
        
        let vulnerabilities = vec![];
        
        let html = generate_html(&summary, &graph_data, &vulnerabilities).unwrap();
        
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("BazBOM Security Report"));
        assert!(html.contains("85")); // security score
        assert!(html.contains("127")); // total deps
    }
}
