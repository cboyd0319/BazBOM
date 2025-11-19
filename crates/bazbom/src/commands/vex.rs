//! VEX (Vulnerability Exploitability eXchange) command handlers

use anyhow::{Context, Result};
use bazbom_vulnerabilities::{
    VexDocument, VexFilter, VexJustification, VexStatement, VexStatus,
};
use colored::Colorize;
use std::path::Path;

/// Handle VEX create command
pub fn handle_vex_create(
    cve: String,
    status: String,
    justification: Option<String>,
    impact: Option<String>,
    package: Option<String>,
    author: String,
    output: Option<String>,
) -> Result<()> {
    // Parse status
    let vex_status = match status.to_lowercase().as_str() {
        "not_affected" | "notaffected" => VexStatus::NotAffected,
        "affected" => VexStatus::Affected,
        "fixed" => VexStatus::Fixed,
        "under_investigation" | "underinvestigation" => VexStatus::UnderInvestigation,
        _ => {
            anyhow::bail!(
                "Invalid status '{}'. Valid values: not_affected, affected, fixed, under_investigation",
                status
            );
        }
    };

    // Parse justification
    let vex_justification = if let Some(j) = justification {
        Some(match j.to_lowercase().as_str() {
            "component_not_present" => VexJustification::ComponentNotPresent,
            "vulnerable_code_not_present" => VexJustification::VulnerableCodeNotPresent,
            "vulnerable_code_not_in_execute_path" => {
                VexJustification::VulnerableCodeNotInExecutePath
            }
            "vulnerable_code_cannot_be_controlled_by_adversary" => {
                VexJustification::VulnerableCodeCannotBeControlledByAdversary
            }
            "inline_mitigations_already_exist" => VexJustification::InlineMitigationsAlreadyExist,
            _ => {
                anyhow::bail!(
                    "Invalid justification '{}'. Valid values: component_not_present, vulnerable_code_not_present, vulnerable_code_not_in_execute_path, vulnerable_code_cannot_be_controlled_by_adversary, inline_mitigations_already_exist",
                    j
                );
            }
        })
    } else {
        None
    };

    // Create document
    let doc_id = format!(
        "https://example.com/vex/{}-{}",
        cve,
        chrono::Utc::now().format("%Y%m%d")
    );
    let mut doc = VexDocument::new(&doc_id, &author);

    // Create statement
    let mut statement = VexStatement::new(&cve, vex_status);

    if let Some(j) = vex_justification {
        statement = statement.with_justification(j);
    }

    if let Some(i) = impact {
        statement = statement.with_impact_statement(&i);
    }

    if let Some(p) = package {
        statement = statement.with_product(&p);
    }

    doc.add_statement(statement);

    // Determine output path
    let output_path = output.unwrap_or_else(|| format!("{}.json", cve));
    let path = Path::new(&output_path);

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Save document
    doc.save(path)?;

    println!();
    println!("{}", "✅ VEX statement created".green().bold());
    println!();
    println!("  CVE:           {}", cve.cyan());
    println!("  Status:        {}", format!("{}", vex_status).yellow());
    if let Some(j) = vex_justification {
        println!("  Justification: {}", format!("{}", j));
    }
    println!("  Output:        {}", output_path.dimmed());
    println!();

    Ok(())
}

/// Handle VEX apply command
pub fn handle_vex_apply(
    vex_dir: String,
    findings: String,
    output: Option<String>,
) -> Result<()> {
    let vex_path = Path::new(&vex_dir);
    let findings_path = Path::new(&findings);

    // Load VEX filter
    println!();
    println!("{}", "🔄 Applying VEX statements...".bold());
    println!();

    let filter = VexFilter::load(vex_path).context("Failed to load VEX statements")?;

    // Load findings
    let findings_content =
        std::fs::read_to_string(findings_path).context("Failed to read findings file")?;
    let mut findings_json: serde_json::Value =
        serde_json::from_str(&findings_content).context("Failed to parse findings JSON")?;

    // Apply filter to vulnerabilities
    let mut suppressed_count = 0;
    let mut kept_count = 0;
    let mut suppressed_list = Vec::new();

    if let Some(vulns) = findings_json
        .get_mut("vulnerabilities")
        .and_then(|v| v.as_array_mut())
    {
        let original_count = vulns.len();

        vulns.retain(|v| {
            let cve = v
                .get("cve_id")
                .or_else(|| v.get("cve"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let purl = v.get("package_url").and_then(|p| p.as_str());

            if filter.should_suppress(cve, purl) {
                suppressed_count += 1;
                suppressed_list.push(cve.to_string());
                false
            } else {
                kept_count += 1;
                true
            }
        });

        println!("  Original findings:  {}", original_count);
        println!("  Suppressed by VEX:  {}", suppressed_count.to_string().green());
        println!("  Remaining:          {}", kept_count);
    }

    // Add suppressed list to output
    findings_json["vex_suppressed"] = serde_json::json!(suppressed_list);

    // Write output
    let output_path = output.unwrap_or_else(|| {
        let stem = findings_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("findings");
        format!("{}_filtered.json", stem)
    });

    let output_content = serde_json::to_string_pretty(&findings_json)?;
    std::fs::write(&output_path, output_content).context("Failed to write filtered findings")?;

    println!();
    println!("  Output: {}", output_path.dimmed());
    println!();

    if suppressed_count > 0 {
        println!("{}", "  Suppressed CVEs:".dimmed());
        for cve in &suppressed_list {
            println!("    {} {}", "-".green(), cve.green());
        }
        println!();
    }

    Ok(())
}

/// Handle VEX list command
pub fn handle_vex_list(vex_dir: String) -> Result<()> {
    let vex_path = Path::new(&vex_dir);

    if !vex_path.exists() {
        println!();
        println!(
            "{}",
            format!("No VEX directory found: {}", vex_dir).yellow()
        );
        println!();
        println!("Create a VEX statement:");
        println!(
            "  {} {} {} {}",
            "bazbom vex create".green(),
            "CVE-2023-12345".cyan(),
            "--status not_affected",
            "--output vex/statements/CVE-2023-12345.json"
        );
        println!();
        return Ok(());
    }

    let documents = VexDocument::load_all(vex_path)?;

    if documents.is_empty() {
        println!();
        println!("{}", "No VEX statements found.".yellow());
        println!();
        return Ok(());
    }

    println!();
    println!("{}", "📋 VEX Statements".bold());
    println!();

    let mut total_statements = 0;
    let mut by_status: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for doc in &documents {
        for statement in &doc.statements {
            total_statements += 1;
            *by_status.entry(statement.status.to_string()).or_insert(0) += 1;

            let status_colored = match statement.status {
                VexStatus::NotAffected => statement.status.to_string().green(),
                VexStatus::Affected => statement.status.to_string().red(),
                VexStatus::Fixed => statement.status.to_string().blue(),
                VexStatus::UnderInvestigation => statement.status.to_string().yellow(),
            };

            println!(
                "  {} {} [{}]",
                statement.vulnerability.id.cyan(),
                status_colored,
                doc.author.dimmed()
            );

            if let Some(ref impact) = statement.impact_statement {
                let truncated = if impact.len() > 60 {
                    format!("{}...", &impact[..57])
                } else {
                    impact.clone()
                };
                println!("    {}", truncated.dimmed());
            }
        }
    }

    println!();
    println!("{}", "Summary:".bold());
    println!("  Total statements: {}", total_statements);
    for (status, count) in &by_status {
        println!("  {}: {}", status, count);
    }
    println!();

    Ok(())
}
