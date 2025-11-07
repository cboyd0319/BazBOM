use anyhow::Result;
use bazbom::cli::TeamCmd;
use bazbom::team::{TeamConfig, TeamCoordinator};

/// Handle the `bazbom team` command
pub fn handle_team(action: TeamCmd) -> Result<()> {
    let coordinator = TeamCoordinator::new(None);

    match action {
        TeamCmd::Assign { cve, to } => handle_assign(&coordinator, &cve, &to),
        TeamCmd::List {} => handle_list(&coordinator),
        TeamCmd::Mine {} => handle_mine(&coordinator),
        TeamCmd::AuditLog { format, output } => handle_audit_log(&coordinator, &format, output),
        TeamCmd::Config {
            name,
            add_member,
            remove_member,
        } => handle_config(name, add_member, remove_member),
    }
}

fn handle_assign(coordinator: &TeamCoordinator, cve: &str, to: &str) -> Result<()> {
    coordinator.assign(cve, to)?;
    coordinator.log_audit_event(
        &format!("Assigned {}", cve),
        Some(format!("Assigned to {}", to)),
    )?;
    Ok(())
}

fn handle_list(coordinator: &TeamCoordinator) -> Result<()> {
    let assignments = coordinator.list_assignments()?;
    if assignments.is_empty() {
        println!("No assignments found.");
    } else {
        println!("Vulnerability Assignments:");
        for assignment in assignments {
            println!(
                "  {} → {} (assigned {})",
                assignment.cve,
                assignment.assignee,
                assignment.assigned_at.format("%Y-%m-%d %H:%M")
            );
        }
    }
    Ok(())
}

fn handle_mine(coordinator: &TeamCoordinator) -> Result<()> {
    // Get current user
    let user = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let assignments = coordinator.get_my_assignments(&user)?;
    if assignments.is_empty() {
        println!("No assignments for {}", user);
    } else {
        println!("{} vulnerabilities assigned to you:", assignments.len());
        for assignment in assignments {
            println!(
                "  {} (assigned {})",
                assignment.cve,
                assignment.assigned_at.format("%Y-%m-%d %H:%M")
            );
        }
    }
    Ok(())
}

fn handle_audit_log(
    coordinator: &TeamCoordinator,
    format: &str,
    output: Option<String>,
) -> Result<()> {
    if format == "csv" {
        let output_path = output.unwrap_or_else(|| "audit.csv".to_string());
        coordinator.export_audit_log(&output_path)?;
    } else {
        let entries = coordinator.get_audit_log(Some(50))?;
        if entries.is_empty() {
            println!("No audit entries found.");
        } else {
            println!("Recent Audit Log:");
            for entry in entries {
                println!(
                    "  {} | {} | {}",
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.user,
                    entry.action
                );
                if let Some(details) = entry.details {
                    println!("    {}", details);
                }
            }
        }
    }
    Ok(())
}

fn handle_config(
    name: Option<String>,
    add_member: Option<String>,
    remove_member: Option<String>,
) -> Result<()> {
    let config_path = ".bazbom/team-config.json";
    let mut config = TeamConfig::load(config_path).unwrap_or_else(|_| TeamConfig {
        name: "Security Team".to_string(),
        members: Vec::new(),
        notification_channels: std::collections::HashMap::new(),
    });

    if let Some(team_name) = name {
        config.name = team_name;
        println!("[+] Set team name to: {}", config.name);
    }

    if let Some(email) = add_member {
        config.add_member(email.clone());
        println!("[+] Added team member: {}", email);
    }

    if let Some(email) = remove_member {
        config.remove_member(&email);
        println!("[+] Removed team member: {}", email);
    }

    // Create .bazbom directory if it doesn't exist
    std::fs::create_dir_all(".bazbom")?;
    config.save(config_path)?;
    println!("[+] Team configuration saved to {}", config_path);
    Ok(())
}
