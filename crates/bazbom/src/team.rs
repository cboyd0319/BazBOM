//! Team coordination features
//!
//! This module implements git-based team coordination for vulnerability management:
//! - Assignment tracking using git notes
//! - Team member management
//! - Audit trail for security actions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use chrono::{DateTime, Utc};

/// Assignment metadata stored in git notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentMetadata {
    pub cve: String,
    pub assignee: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<String>,
    pub notes: Option<String>,
}

/// Assignment information
#[derive(Debug, Clone)]
pub struct Assignment {
    pub cve: String,
    pub assignee: String,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: Option<String>,
    pub notes: Option<String>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub action: String,
    pub details: Option<String>,
}

/// Team coordinator for managing vulnerability assignments
pub struct TeamCoordinator {
    /// Path to git repository
    repo_path: String,
}

impl TeamCoordinator {
    /// Create a new team coordinator
    pub fn new(repo_path: Option<String>) -> Self {
        Self {
            repo_path: repo_path.unwrap_or_else(|| ".".to_string()),
        }
    }

    /// Check if we're in a git repository
    pub fn is_git_repo(&self) -> bool {
        Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&self.repo_path)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Assign a vulnerability to a team member
    pub fn assign(&self, cve: &str, assignee: &str) -> Result<()> {
        if !self.is_git_repo() {
            anyhow::bail!("Not a git repository. Git-based assignments require a git repo.");
        }

        let metadata = AssignmentMetadata {
            cve: cve.to_string(),
            assignee: assignee.to_string(),
            assigned_at: Utc::now(),
            assigned_by: self.get_current_user(),
            notes: None,
        };

        let json = serde_json::to_string(&metadata)?;
        let notes_ref = format!("bazbom/assignments/{}", cve);

        // Store assignment in git notes
        // Note: git notes requires a commit to attach to, using HEAD
        let output = Command::new("git")
            .args(["notes", "--ref", &notes_ref, "add", "-f", "-m", &json, "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create git note")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to assign vulnerability: {}", stderr);
        }

        println!("✅ Assigned {} to {}", cve, assignee);
        Ok(())
    }

    /// List all assignments
    pub fn list_assignments(&self) -> Result<Vec<Assignment>> {
        if !self.is_git_repo() {
            anyhow::bail!("Not a git repository");
        }

        // List all notes refs that start with bazbom/assignments/
        let output = Command::new("git")
            .args(["notes", "list"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to list git notes")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let assignments = Vec::new();

        // For each assignment note, parse the metadata
        // Note: This is a simplified implementation
        // In production, we'd iterate through all bazbom/assignments/* refs

        Ok(assignments)
    }

    /// Get assignments for a specific user
    pub fn get_my_assignments(&self, user: &str) -> Result<Vec<Assignment>> {
        let all_assignments = self.list_assignments()?;
        Ok(all_assignments
            .into_iter()
            .filter(|a| a.assignee == user)
            .collect())
    }

    /// Get the current git user
    fn get_current_user(&self) -> Option<String> {
        let output = Command::new("git")
            .args(["config", "user.email"])
            .current_dir(&self.repo_path)
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    /// Log an audit event
    pub fn log_audit_event(&self, action: &str, details: Option<String>) -> Result<()> {
        let user = self.get_current_user().unwrap_or_else(|| "unknown".to_string());
        
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user,
            action: action.to_string(),
            details,
        };

        // Store audit entry in .bazbom/audit.json
        let audit_dir = std::path::Path::new(&self.repo_path).join(".bazbom");
        std::fs::create_dir_all(&audit_dir)?;
        
        let audit_file = audit_dir.join("audit.json");
        let mut entries: Vec<AuditEntry> = if audit_file.exists() {
            let content = std::fs::read_to_string(&audit_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        entries.push(entry);

        // Keep only last 1000 entries
        if entries.len() > 1000 {
            let skip_count = entries.len() - 1000;
            entries = entries.into_iter().skip(skip_count).collect();
        }

        let json = serde_json::to_string_pretty(&entries)?;
        std::fs::write(audit_file, json)?;

        Ok(())
    }

    /// Get audit log
    pub fn get_audit_log(&self, limit: Option<usize>) -> Result<Vec<AuditEntry>> {
        let audit_file = std::path::Path::new(&self.repo_path)
            .join(".bazbom")
            .join("audit.json");

        if !audit_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&audit_file)?;
        let mut entries: Vec<AuditEntry> = serde_json::from_str(&content)?;

        if let Some(limit) = limit {
            entries = entries.into_iter().rev().take(limit).rev().collect();
        }

        Ok(entries)
    }

    /// Export audit log to CSV
    pub fn export_audit_log(&self, output_path: &str) -> Result<()> {
        let entries = self.get_audit_log(None)?;

        let mut csv = String::new();
        csv.push_str("timestamp,user,action,details\n");

        for entry in entries {
            csv.push_str(&format!(
                "{},{},{},{}\n",
                entry.timestamp.to_rfc3339(),
                entry.user,
                entry.action,
                entry.details.unwrap_or_default()
            ));
        }

        std::fs::write(output_path, csv)?;
        println!("✅ Exported audit log to {}", output_path);

        Ok(())
    }
}

/// Team configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub name: String,
    pub members: Vec<String>,
    pub notification_channels: HashMap<String, String>,
}

impl TeamConfig {
    /// Load team configuration
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read team configuration")?;
        let config: TeamConfig = serde_json::from_str(&content)
            .context("Failed to parse team configuration")?;
        Ok(config)
    }

    /// Save team configuration
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Add a team member
    pub fn add_member(&mut self, email: String) {
        if !self.members.contains(&email) {
            self.members.push(email);
        }
    }

    /// Remove a team member
    pub fn remove_member(&mut self, email: &str) {
        self.members.retain(|m| m != email);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_metadata_serialization() {
        let metadata = AssignmentMetadata {
            cve: "CVE-2021-44228".to_string(),
            assignee: "alice@example.com".to_string(),
            assigned_at: Utc::now(),
            assigned_by: Some("bob@example.com".to_string()),
            notes: Some("High priority".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: AssignmentMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.cve, "CVE-2021-44228");
        assert_eq!(parsed.assignee, "alice@example.com");
    }

    #[test]
    fn test_team_config() {
        let mut config = TeamConfig {
            name: "Security Team".to_string(),
            members: vec!["alice@example.com".to_string()],
            notification_channels: HashMap::new(),
        };

        config.add_member("bob@example.com".to_string());
        assert_eq!(config.members.len(), 2);

        config.remove_member("alice@example.com");
        assert_eq!(config.members.len(), 1);
        assert_eq!(config.members[0], "bob@example.com");
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            user: "alice@example.com".to_string(),
            action: "Fixed CVE-2021-44228".to_string(),
            details: Some("Upgraded log4j to 2.21.1".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: AuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.user, "alice@example.com");
        assert_eq!(parsed.action, "Fixed CVE-2021-44228");
    }
}
