use anyhow::{Context, Result};
use bazbom::cli::PolicyCmd;
use bazbom::{advisory, policy_integration};
use std::fs;
use std::path::PathBuf;

/// Handle the `bazbom policy` command
pub fn handle_policy(action: PolicyCmd) -> Result<()> {
    match action {
        PolicyCmd::Check {} => handle_policy_check(),
        PolicyCmd::Init {
            list,
            template,
            output,
        } => handle_policy_init(list, template, output),
        PolicyCmd::Validate { policy_file } => handle_policy_validate(policy_file),
    }
}

fn handle_policy_check() -> Result<()> {
    println!("[bazbom] policy check");

    // Load policy configuration
    let policy_path = PathBuf::from("bazbom.yml");
    let policy = policy_integration::load_policy_config(&policy_path)
        .context("failed to load policy configuration")?;
    println!(
        "[bazbom] loaded policy config (threshold={:?})",
        policy.severity_threshold
    );

    // Load advisories from cache
    let cache_dir = PathBuf::from(".bazbom/cache");
    let vulnerabilities = load_advisories_from_cache(&cache_dir)?;

    // Check vulnerabilities against policy
    let result = policy_integration::check_policy(&vulnerabilities, &policy);

    // Write policy result to JSON
    write_policy_result(&result)?;

    // Write policy violations to SARIF
    write_policy_violations_sarif(&result)?;

    // Print summary and exit with appropriate code
    print_policy_summary(&result)
}

fn load_advisories_from_cache(
    cache_dir: &PathBuf,
) -> Result<Vec<bazbom_advisories::Vulnerability>> {
    if cache_dir.exists() {
        match advisory::load_advisories(cache_dir) {
            Ok(vulns) => {
                println!("[bazbom] loaded {} vulnerabilities from cache", vulns.len());
                Ok(vulns)
            }
            Err(e) => {
                eprintln!("[bazbom] warning: failed to load advisories: {}", e);
                Ok(Vec::new())
            }
        }
    } else {
        eprintln!(
            "[bazbom] warning: advisory cache not found at {:?}, run 'bazbom db sync' first",
            cache_dir
        );
        Ok(Vec::new())
    }
}

fn write_policy_result(result: &bazbom_policy::PolicyResult) -> Result<()> {
    let policy_output = PathBuf::from("policy_result.json");
    let json_data = serde_json::to_vec_pretty(&result)
        .context("failed to serialize policy result to JSON")?;
    fs::write(&policy_output, json_data)
        .with_context(|| format!("failed writing {:?}", policy_output))?;
    println!("[bazbom] wrote {:?}", policy_output);
    Ok(())
}

fn write_policy_violations_sarif(result: &bazbom_policy::PolicyResult) -> Result<()> {
    let sarif_path = PathBuf::from("policy_violations.sarif");
    let mut sarif = bazbom_formats::sarif::SarifReport::new("bazbom-policy", bazbom_core::VERSION);

    for violation in &result.violations {
        let level = determine_violation_level(violation);
        let rule_id = format!("policy/{}", violation.rule);
        let result_item = bazbom_formats::sarif::Result::new(&rule_id, level, &violation.message);
        sarif.add_result(result_item);
    }

    let json_data = serde_json::to_vec_pretty(&sarif)
        .context("failed to serialize SARIF report to JSON")?;
    fs::write(&sarif_path, json_data)
        .with_context(|| format!("failed writing {:?}", sarif_path))?;
    println!(
        "[bazbom] wrote {:?} ({} violations)",
        sarif_path,
        result.violations.len()
    );
    Ok(())
}

fn determine_violation_level(violation: &bazbom_policy::PolicyViolation) -> &'static str {
    if violation.rule == "kev_gate" {
        "error"
    } else if let Some(vuln) = &violation.vulnerability {
        match vuln.severity {
            bazbom_policy::SeverityLevel::Critical => "error",
            bazbom_policy::SeverityLevel::High => "error",
            bazbom_policy::SeverityLevel::Medium => "warning",
            _ => "note",
        }
    } else {
        "warning"
    }
}

fn print_policy_summary(result: &bazbom_policy::PolicyResult) -> Result<()> {
    if result.passed {
        println!("[bazbom] [+] policy check passed (no violations)");
        Ok(())
    } else {
        println!(
            "[bazbom] [X] policy check failed ({} violations)",
            result.violations.len()
        );
        for violation in &result.violations {
            println!("  - {}: {}", violation.rule, violation.message);
        }
        std::process::exit(1);
    }
}

fn handle_policy_init(list: bool, template: Option<String>, output: String) -> Result<()> {
    if list {
        list_policy_templates();
        Ok(())
    } else if let Some(template_id) = template {
        initialize_policy_template(&template_id, &output)
    } else {
        eprintln!("Error: Either --list or --template <template-id> must be specified");
        eprintln!("Run 'bazbom policy init --list' to see available templates");
        std::process::exit(1);
    }
}

fn list_policy_templates() {
    println!("[bazbom] Available policy templates:\n");
    let templates = bazbom_policy::PolicyTemplateLibrary::list_templates();

    let mut by_category: std::collections::HashMap<String, Vec<_>> =
        std::collections::HashMap::new();
    for template in templates {
        by_category
            .entry(template.category.clone())
            .or_insert_with(Vec::new)
            .push(template);
    }

    for (category, templates) in by_category {
        println!("{}:", category);
        for template in templates {
            println!("  {} - {}", template.id, template.name);
            println!("    {}", template.description);
        }
        println!();
    }

    println!("Usage: bazbom policy init --template <template-id>");
}

fn initialize_policy_template(template_id: &str, output: &str) -> Result<()> {
    let output_path = PathBuf::from(output);
    match bazbom_policy::PolicyTemplateLibrary::initialize_template(template_id, &output_path) {
        Ok(msg) => {
            println!("{}", msg);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_policy_validate(policy_file: String) -> Result<()> {
    println!("[bazbom] validating policy file: {}", policy_file);

    let policy_path = PathBuf::from(&policy_file);
    match policy_integration::load_policy_config(&policy_path) {
        Ok(policy) => {
            print_policy_details(&policy);
            Ok(())
        }
        Err(e) => {
            eprintln!("[X] Policy file validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_policy_details(policy: &bazbom_policy::PolicyConfig) {
    println!("[+] Policy file is valid");
    println!("\nPolicy Configuration:");
    println!("  Severity threshold: {:?}", policy.severity_threshold);
    println!("  KEV gate: {}", policy.kev_gate);
    println!("  EPSS threshold: {:?}", policy.epss_threshold);
    println!("  Reachability required: {}", policy.reachability_required);
    println!("  VEX auto-apply: {}", policy.vex_auto_apply);

    if let Some(allowlist) = &policy.license_allowlist {
        println!("  License allowlist: {} licenses", allowlist.len());
    }
    if let Some(denylist) = &policy.license_denylist {
        println!("  License denylist: {} licenses", denylist.len());
    }
}
