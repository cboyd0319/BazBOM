//! Security audit logging
//!
//! Provides structured logging for security-relevant events such as:
//! - Authentication attempts
//! - Authorization failures
//! - Configuration changes
//! - Vulnerability fixes
//! - Policy violations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Security audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// Authentication event
    Authentication,
    /// Authorization event
    Authorization,
    /// Configuration change
    ConfigChange,
    /// Vulnerability detected
    VulnerabilityDetected,
    /// Vulnerability fixed
    VulnerabilityFixed,
    /// Policy violation
    PolicyViolation,
    /// Policy enforcement
    PolicyEnforcement,
    /// Scan started
    ScanStarted,
    /// Scan completed
    ScanCompleted,
    /// Data access
    DataAccess,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// User or system that triggered the event
    pub actor: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: Option<String>,
    /// Result (success/failure)
    pub result: AuditResult,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// IP address (for network events)
    pub ip_address: Option<String>,
}

/// Result of an audited action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        actor: String,
        action: String,
        result: AuditResult,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            event_type,
            actor,
            action,
            resource: None,
            result,
            metadata: serde_json::json!({}),
            ip_address: None,
        }
    }

    /// Set resource for this entry
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set metadata for this entry
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set IP address for this entry
    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }
}

/// Audit logger
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    /// Get default audit log path
    pub fn default_path() -> PathBuf {
        PathBuf::from(".bazbom/audit.jsonl")
    }

    /// Log an audit event
    pub fn log(&self, entry: &AuditLogEntry) -> anyhow::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        // Write as JSON Lines format (one JSON object per line)
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;

        file.sync_all()?;

        Ok(())
    }

    /// Read all audit log entries
    pub fn read_all(&self) -> anyhow::Result<Vec<AuditLogEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.log_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<AuditLogEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse audit log line: {}", e),
            }
        }

        Ok(entries)
    }

    /// Query audit log by event type
    pub fn query_by_type(&self, event_type: AuditEventType) -> anyhow::Result<Vec<AuditLogEntry>> {
        let all_entries = self.read_all()?;
        Ok(all_entries
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect())
    }

    /// Query audit log by actor
    pub fn query_by_actor(&self, actor: &str) -> anyhow::Result<Vec<AuditLogEntry>> {
        let all_entries = self.read_all()?;
        Ok(all_entries
            .into_iter()
            .filter(|e| e.actor == actor)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_log_entry_creation() {
        let entry = AuditLogEntry::new(
            AuditEventType::Authentication,
            "user@example.com".to_string(),
            "login".to_string(),
            AuditResult::Success,
        )
        .with_resource("dashboard".to_string())
        .with_ip("127.0.0.1".to_string());

        assert_eq!(entry.event_type, AuditEventType::Authentication);
        assert_eq!(entry.actor, "user@example.com");
        assert_eq!(entry.result, AuditResult::Success);
        assert_eq!(entry.resource, Some("dashboard".to_string()));
        assert_eq!(entry.ip_address, Some("127.0.0.1".to_string()));
    }

    #[test]
    fn test_audit_logger_write_read() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(log_path);

        // Log some events
        let entry1 = AuditLogEntry::new(
            AuditEventType::VulnerabilityDetected,
            "bazbom-scanner".to_string(),
            "detected CVE-2021-44228".to_string(),
            AuditResult::Success,
        );

        let entry2 = AuditLogEntry::new(
            AuditEventType::VulnerabilityFixed,
            "alice@example.com".to_string(),
            "upgraded log4j to 2.21.1".to_string(),
            AuditResult::Success,
        );

        logger.log(&entry1).unwrap();
        logger.log(&entry2).unwrap();

        // Read back
        let entries = logger.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].event_type, AuditEventType::VulnerabilityDetected);
        assert_eq!(entries[1].event_type, AuditEventType::VulnerabilityFixed);
    }

    #[test]
    fn test_query_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(log_path);

        logger
            .log(&AuditLogEntry::new(
                AuditEventType::Authentication,
                "user1".to_string(),
                "login".to_string(),
                AuditResult::Success,
            ))
            .unwrap();

        logger
            .log(&AuditLogEntry::new(
                AuditEventType::VulnerabilityDetected,
                "scanner".to_string(),
                "detected".to_string(),
                AuditResult::Success,
            ))
            .unwrap();

        let auth_events = logger
            .query_by_type(AuditEventType::Authentication)
            .unwrap();
        assert_eq!(auth_events.len(), 1);
        assert_eq!(auth_events[0].actor, "user1");
    }
}
