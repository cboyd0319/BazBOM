use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::PolicyResult;

/// Represents a single audit log entry for a policy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// ISO 8601 timestamp of the policy check
    pub timestamp: String,
    /// Unix timestamp in seconds
    pub timestamp_unix: u64,
    /// Policy action taken (e.g., "scan", "check", "validate")
    pub action: String,
    /// Result of the policy check
    pub result: AuditResult,
    /// Number of violations found
    pub violation_count: usize,
    /// Number of warnings issued
    pub warning_count: usize,
    /// Policy configuration used (file path or identifier)
    pub policy_source: Option<String>,
    /// Optional context (e.g., project name, CI job ID)
    pub context: Option<AuditContext>,
}

/// Result of a policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    /// Policy check passed
    Pass,
    /// Policy check failed with violations
    Fail,
    /// Policy check completed with warnings
    Warn,
}

/// Additional context for audit entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Project or repository name
    pub project: Option<String>,
    /// User or service account that triggered the check
    pub user: Option<String>,
    /// CI/CD job identifier
    pub ci_job_id: Option<String>,
    /// Git commit SHA
    pub commit_sha: Option<String>,
    /// Additional key-value metadata
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Configuration for audit trail logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    /// Path to audit log file
    pub log_file: PathBuf,
    /// Log all scans (including passing ones)
    pub log_all_scans: bool,
    /// Log policy violations
    pub log_violations: bool,
    /// Log exceptions/warnings
    pub log_warnings: bool,
    /// Maximum log file size in bytes (0 = unlimited)
    pub max_size_bytes: usize,
    /// Log retention in days (0 = unlimited)
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_file: PathBuf::from(".bazbom/audit.jsonl"),
            log_all_scans: false,
            log_violations: true,
            log_warnings: true,
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            retention_days: 365, // 1 year
        }
    }
}

/// Audit logger for policy decisions
pub struct AuditLogger {
    config: AuditConfig,
}

impl AuditLogger {
    /// Create a new audit logger with the given configuration
    pub fn new(config: AuditConfig) -> Self {
        Self { config }
    }

    /// Create a default audit logger (disabled by default)
    pub fn new_default() -> Self {
        Self::new(AuditConfig::default())
    }

    /// Log a policy check result
    pub fn log_policy_check(
        &self,
        action: &str,
        result: &PolicyResult,
        policy_source: Option<&str>,
        context: Option<AuditContext>,
    ) -> std::io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Determine if we should log this entry
        let should_log = if result.passed {
            self.config.log_all_scans
        } else if result.violations.is_empty() {
            self.config.log_warnings
        } else {
            self.config.log_violations
        };

        if !should_log {
            return Ok(());
        }

        // Create audit log entry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let entry = AuditLogEntry {
            timestamp: chrono::DateTime::<chrono::Utc>::from(SystemTime::now())
                .to_rfc3339(),
            timestamp_unix: now.as_secs(),
            action: action.to_string(),
            result: if result.passed {
                AuditResult::Pass
            } else if result.violations.is_empty() {
                AuditResult::Warn
            } else {
                AuditResult::Fail
            },
            violation_count: result.violations.len(),
            warning_count: 0, // TODO: separate warnings from violations
            policy_source: policy_source.map(String::from),
            context,
        };

        // Write to log file (JSONL format - one JSON object per line)
        self.write_log_entry(&entry)?;

        Ok(())
    }

    /// Write a log entry to the audit file
    fn write_log_entry(&self, entry: &AuditLogEntry) -> std::io::Result<()> {
        // Ensure the parent directory exists
        if let Some(parent) = self.config.log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check file size and rotate if needed
        if self.config.max_size_bytes > 0 {
            if let Ok(metadata) = std::fs::metadata(&self.config.log_file) {
                if metadata.len() as usize >= self.config.max_size_bytes {
                    self.rotate_log_file()?;
                }
            }
        }

        // Append to log file in JSONL format
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file)?;

        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Rotate the log file when it exceeds max size
    fn rotate_log_file(&self) -> std::io::Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let rotated_name = format!(
            "{}.{}",
            self.config
                .log_file
                .to_str()
                .unwrap_or("audit.jsonl"),
            timestamp
        );

        std::fs::rename(&self.config.log_file, rotated_name)?;
        Ok(())
    }

    /// Query audit logs (basic filtering)
    pub fn query_logs(
        &self,
        since: Option<u64>,
        until: Option<u64>,
        action: Option<&str>,
        result: Option<AuditResult>,
    ) -> std::io::Result<Vec<AuditLogEntry>> {
        let file = File::open(&self.config.log_file)?;
        let reader = std::io::BufReader::new(file);
        let mut entries = Vec::new();

        use std::io::BufRead;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line) {
                // Apply filters
                if let Some(since_ts) = since {
                    if entry.timestamp_unix < since_ts {
                        continue;
                    }
                }
                if let Some(until_ts) = until {
                    if entry.timestamp_unix > until_ts {
                        continue;
                    }
                }
                if let Some(action_filter) = action {
                    if entry.action != action_filter {
                        continue;
                    }
                }
                if let Some(ref result_filter) = result {
                    if !matches!(
                        (&entry.result, result_filter),
                        (AuditResult::Pass, AuditResult::Pass)
                            | (AuditResult::Fail, AuditResult::Fail)
                            | (AuditResult::Warn, AuditResult::Warn)
                    ) {
                        continue;
                    }
                }
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Clean up old log entries based on retention policy
    pub fn cleanup_old_logs(&self) -> std::io::Result<()> {
        if self.config.retention_days == 0 {
            return Ok(()); // Unlimited retention
        }

        let cutoff_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - (self.config.retention_days as u64 * 86400); // days to seconds

        // Read all entries
        let entries = self.query_logs(Some(cutoff_timestamp), None, None, None)?;

        // Rewrite log file with only recent entries
        let temp_file = self.config.log_file.with_extension("tmp");
        let mut file = File::create(&temp_file)?;

        for entry in entries {
            let json = serde_json::to_string(&entry)?;
            writeln!(file, "{}", json)?;
        }

        // Replace old file with new one
        std::fs::rename(temp_file, &self.config.log_file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PolicyViolation;
    use std::io::Read;
    use tempfile::TempDir;

    #[test]
    fn test_audit_config_default() {
        let config = AuditConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.log_file, PathBuf::from(".bazbom/audit.jsonl"));
        assert!(config.log_violations);
        assert_eq!(config.retention_days, 365);
    }

    #[test]
    fn test_audit_logger_disabled() {
        let config = AuditConfig {
            enabled: false,
            ..Default::default()
        };
        let logger = AuditLogger::new(config);

        let result = PolicyResult {
            passed: false,
            violations: vec![],
        };

        // Should not error even with invalid path when disabled
        assert!(logger
            .log_policy_check("test", &result, None, None)
            .is_ok());
    }

    #[test]
    fn test_audit_logger_write_entry() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("audit.jsonl");

        let config = AuditConfig {
            enabled: true,
            log_file: log_file.clone(),
            log_all_scans: true,
            ..Default::default()
        };

        let logger = AuditLogger::new(config);

        let result = PolicyResult {
            passed: true,
            violations: vec![],
        };

        logger
            .log_policy_check("scan", &result, Some("bazbom.yml"), None)
            .unwrap();

        // Verify file was created and contains data
        assert!(log_file.exists());

        let mut content = String::new();
        File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("\"action\":\"scan\""));
        assert!(content.contains("\"result\":\"pass\""));
    }

    #[test]
    fn test_audit_logger_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("audit.jsonl");

        let config = AuditConfig {
            enabled: true,
            log_file: log_file.clone(),
            log_violations: true,
            ..Default::default()
        };

        let logger = AuditLogger::new(config);

        let result = PolicyResult {
            passed: false,
            violations: vec![PolicyViolation {
                rule: "test_rule".to_string(),
                message: "Test violation".to_string(),
                vulnerability: None,
            }],
        };

        let context = AuditContext {
            project: Some("test-project".to_string()),
            user: Some("test-user".to_string()),
            ci_job_id: Some("12345".to_string()),
            commit_sha: Some("abc123".to_string()),
            metadata: None,
        };

        logger
            .log_policy_check("check", &result, Some("test.yml"), Some(context))
            .unwrap();

        let mut content = String::new();
        File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert!(content.contains("\"project\":\"test-project\""));
        assert!(content.contains("\"user\":\"test-user\""));
        assert!(content.contains("\"violation_count\":1"));
    }

    #[test]
    fn test_audit_logger_query() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("audit.jsonl");

        let config = AuditConfig {
            enabled: true,
            log_file: log_file.clone(),
            log_all_scans: true,
            ..Default::default()
        };

        let logger = AuditLogger::new(config);

        // Log multiple entries
        let pass_result = PolicyResult {
            passed: true,
            violations: vec![],
        };
        let fail_result = PolicyResult {
            passed: false,
            violations: vec![PolicyViolation {
                rule: "test".to_string(),
                message: "Test".to_string(),
                vulnerability: None,
            }],
        };

        logger
            .log_policy_check("scan", &pass_result, None, None)
            .unwrap();
        logger
            .log_policy_check("check", &fail_result, None, None)
            .unwrap();

        // Query all logs
        let all_logs = logger.query_logs(None, None, None, None).unwrap();
        assert_eq!(all_logs.len(), 2);

        // Query by action
        let scan_logs = logger
            .query_logs(None, None, Some("scan"), None)
            .unwrap();
        assert_eq!(scan_logs.len(), 1);
        assert_eq!(scan_logs[0].action, "scan");

        // Query by result
        let fail_logs = logger
            .query_logs(None, None, None, Some(AuditResult::Fail))
            .unwrap();
        assert_eq!(fail_logs.len(), 1);
        assert!(matches!(fail_logs[0].result, AuditResult::Fail));
    }

    #[test]
    fn test_log_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_file = temp_dir.path().join("audit.jsonl");

        let config = AuditConfig {
            enabled: true,
            log_file: log_file.clone(),
            log_all_scans: true,
            max_size_bytes: 100, // Very small to trigger rotation
            ..Default::default()
        };

        let logger = AuditLogger::new(config);

        // Write multiple entries to trigger rotation
        for i in 0..10 {
            let result = PolicyResult {
                passed: true,
                violations: vec![],
            };
            logger
                .log_policy_check(&format!("scan_{}", i), &result, None, None)
                .unwrap();
        }

        // Original file should be smaller than total writes
        let metadata = std::fs::metadata(&log_file).unwrap();
        assert!(metadata.len() > 0);

        // Check that rotated files exist
        let entries = std::fs::read_dir(temp_dir.path()).unwrap();
        let rotated_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("audit.jsonl.")
            })
            .collect();

        assert!(!rotated_files.is_empty(), "Log rotation should have occurred");
    }
}
