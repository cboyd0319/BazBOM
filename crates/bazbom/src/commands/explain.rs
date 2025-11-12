use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Handle the `bazbom explain` command
///
/// Provides detailed information about a specific vulnerability including:
/// - Severity and CVSS score
/// - Reachability status
/// - Call chain (if reachable)
/// - Remediation guidance
/// - References and links
pub fn handle_explain(cve_id: String, findings_path: Option<String>, verbose: bool) -> Result<()> {
    println!();
    println!("{}", format!("🔍 Explaining {}", cve_id).bold().cyan());
    println!();

    let findings_file = findings_path.unwrap_or_else(|| "bazbom-findings.json".to_string());

    if !Path::new(&findings_file).exists() {
        println!("{}", "Error: Findings file not found".red().bold());
        println!();
        println!("Please run a scan first:");
        println!("  {} {}", "bazbom scan --reachability".green(), "[path]".dimmed());
        println!();
        println!("Or specify a findings file:");
        println!("  {} {} {}", "bazbom explain".green(), cve_id.yellow(), "--findings=/path/to/findings.json".dimmed());
        println!();
        return Ok(());
    }

    // Load and parse findings
    let findings_content = std::fs::read_to_string(&findings_file)?;
    let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

    println!("{} {}", "Findings file:".dimmed(), findings_file);
    println!();

    // Search for CVE in findings
    let mut found = false;
    if let Some(runs) = findings.get("runs").and_then(|r| r.as_array()) {
        for run in runs {
            if let Some(results) = run.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    if let Some(rule_id) = result.get("ruleId").and_then(|r| r.as_str()) {
                        if rule_id == cve_id {
                            found = true;
                            display_vulnerability(result, &cve_id, verbose)?;
                            break;
                        }
                    }
                }
            }
        }
    }

    if !found {
        println!("{}", format!("No information found for {}", cve_id).yellow());
        println!();
        println!("{}", "This CVE may not affect your project, or the findings file may be outdated.".dimmed());
    }

    println!();
    println!("{}", "📚 References:".bold());
    println!("  • NVD: https://nvd.nist.gov/vuln/detail/{}", cve_id);
    println!();

    Ok(())
}

/// Display vulnerability details from SARIF result
fn display_vulnerability(result: &serde_json::Value, cve_id: &str, verbose: bool) -> Result<()> {
    println!("{}", "📦 Affected Package:".bold());
    
    if let Some(message) = result.get("message").and_then(|m| m.get("text")).and_then(|t| t.as_str()) {
        println!("  {}", message);
    }
    println!();

    println!("{}", "⚠️  Severity Information:".bold());
    if let Some(level) = result.get("level").and_then(|l| l.as_str()) {
        let level_colored = match level {
            "error" => "CRITICAL".red().bold(),
            "warning" => "HIGH".yellow().bold(),
            _ => "MEDIUM".cyan().bold(),
        };
        println!("  Severity: {}", level_colored);
    }
    
    if let Some(properties) = result.get("properties") {
        if let Some(cvss) = properties.get("cvss").and_then(|c| c.as_f64()) {
            println!("  CVSS Score: {:.1}", cvss);
        }
    }
    println!();

    println!("{}", "🎯 Reachability Analysis:".bold());
    if let Some(properties) = result.get("properties") {
        if let Some(reachable) = properties.get("reachable").and_then(|r| r.as_bool()) {
            if reachable {
                println!("  Status: {} {}", "REACHABLE".red().bold(), "(actively exploitable)".red());
            } else {
                println!("  Status: {} {}", "UNREACHABLE".green().bold(), "(not called by your code)".green());
            }
        } else {
            println!("  Status: {}", "UNKNOWN (reachability not analyzed)".yellow());
        }
        
        if verbose {
            if let Some(call_chain) = properties.get("callChain").and_then(|c| c.as_array()) {
                println!();
                println!("{}", "📍 Call Chain:".bold());
                for (i, call) in call_chain.iter().enumerate() {
                    if let Some(call_str) = call.as_str() {
                        println!("  {}→ {}", "  ".repeat(i), call_str);
                    }
                }
            }
        }
    }
    println!();

    println!("{}", "🔧 Remediation:".bold());
    if let Some(properties) = result.get("properties") {
        if let Some(fix) = properties.get("fix").and_then(|f| f.as_str()) {
            println!("  {}", fix);
        } else {
            println!("  No automated fix available. Check vendor advisories.");
        }
    }
    println!();

    Ok(())
}
