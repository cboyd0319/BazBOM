//! Audit Logging
//!
//! Comprehensive, tamper-evident audit logging for security events.
//!
//! # Security Features
//!
//! - HMAC signatures for tamper detection
//! - Structured JSON logging
//! - Automatic log rotation
//! - Immutable append-only logs
//! - SIEM integration ready

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use time::OffsetDateTime;

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Authentication events
    Authentication,
    /// Authorization checks
    Authorization,
    /// Data access
    DataAccess,
    /// Data modification
    DataModification,
    /// Configuration changes
    ConfigurationChange,
    /// Security events (failed auth, rate limit, etc.)
    SecurityEvent,
    /// System errors
    Error,
}

/// Audit event result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    /// Operation succeeded
    Success,
    /// Operation failed with reason
    Failure(String),
    /// Operation partially succeeded
    PartialSuccess,
}

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Timestamp (ISO 8601)
    pub timestamp: OffsetDateTime,
    /// Event type
    pub event_type: AuditEventType,
    /// Actor (user/service that performed the action)
    pub actor: String,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource: String,
    /// Result of the operation
    pub result: AuditResult,
    /// Additional metadata (JSON object)
    pub metadata: serde_json::Value,
    /// HMAC signature for tamper-evidence
    pub signature: String,
    /// Source IP address (if applicable)
    pub source_ip: Option<String>,
    /// User agent (if applicable)
    pub user_agent: Option<String>,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(
        event_type: AuditEventType,
        actor: &str,
        action: &str,
        resource: &str,
        result: AuditResult,
    ) -> Self {
        Self {
            timestamp: OffsetDateTime::now_utc(),
            event_type,
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            result,
            metadata: serde_json::json!({}),
            signature: String::new(), // Will be computed when logging
            source_ip: None,
            user_agent: None,
        }
    }

    /// Add metadata to event
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add source IP
    pub fn with_source_ip(mut self, ip: &str) -> Self {
        self.source_ip = Some(ip.to_string());
        self
    }

    /// Add user agent
    pub fn with_user_agent(mut self, ua: &str) -> Self {
        self.user_agent = Some(ua.to_string());
        self
    }

    /// Compute HMAC signature for tamper-evidence
    fn sign(&mut self, secret_key: &[u8]) {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // Serialize event without signature
        let mut data = serde_json::to_string(self).unwrap();

        // Remove the signature field from JSON for consistent hashing
        let value: serde_json::Value = serde_json::from_str(&data).unwrap();
        if let serde_json::Value::Object(mut map) = value {
            map.remove("signature");
            data = serde_json::to_string(&map).unwrap();
        }

        let mut mac = HmacSha256::new_from_slice(secret_key).unwrap();
        mac.update(data.as_bytes());

        self.signature = hex::encode(mac.finalize().into_bytes());
    }

    /// Verify HMAC signature
    pub fn verify(&self, secret_key: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // Serialize event without signature
        let mut data = serde_json::to_string(self).unwrap();
        let value: serde_json::Value = serde_json::from_str(&data).unwrap();
        if let serde_json::Value::Object(mut map) = value {
            map.remove("signature");
            data = serde_json::to_string(&map).unwrap();
        }

        let mut mac = HmacSha256::new_from_slice(secret_key).unwrap();
        mac.update(data.as_bytes());

        let expected = hex::encode(mac.finalize().into_bytes());

        // Constant-time comparison
        use subtle::ConstantTimeEq;
        expected.as_bytes().ct_eq(self.signature.as_bytes()).into()
    }
}

/// Audit logger configuration
pub struct AuditLoggerConfig {
    /// Directory for audit logs
    pub log_dir: PathBuf,
    /// Secret key for HMAC signatures
    pub secret_key: Vec<u8>,
    /// Enable log rotation (daily)
    pub rotate_daily: bool,
    /// Retention period in days
    pub retention_days: u32,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from(".bazbom/audit"),
            secret_key: vec![],
            rotate_daily: true,
            retention_days: 90,
        }
    }
}

/// Audit logger
pub struct AuditLogger {
    config: AuditLoggerConfig,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(config: AuditLoggerConfig) -> anyhow::Result<Self> {
        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&config.log_dir)?;

        Ok(Self { config })
    }

    /// Log an audit event
    pub fn log(&self, mut event: AuditEvent) -> anyhow::Result<()> {
        // Sign the event
        event.sign(&self.config.secret_key);

        // Determine log file path
        let log_file = self.get_log_file()?;

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;

        // Write event as JSON line
        let json = serde_json::to_string(&event)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Get current log file path (with rotation)
    fn get_log_file(&self) -> anyhow::Result<PathBuf> {
        if self.config.rotate_daily {
            let date = OffsetDateTime::now_utc().date();
            Ok(self.config.log_dir.join(format!("{}.log", date)))
        } else {
            Ok(self.config.log_dir.join("audit.log"))
        }
    }

    /// Read audit log entries from a file
    pub fn read_log(&self, date: Option<time::Date>) -> anyhow::Result<Vec<AuditEvent>> {
        let log_file = if let Some(d) = date {
            self.config.log_dir.join(format!("{}.log", d))
        } else {
            self.get_log_file()?
        };

        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&log_file)?;
        let mut events = Vec::new();

        for line in content.lines() {
            if line.is_empty() {
                continue;
            }

            let event: AuditEvent = serde_json::from_str(line)?;
            events.push(event);
        }

        Ok(events)
    }

    /// Verify integrity of all audit logs
    pub fn verify_integrity(&self) -> anyhow::Result<Vec<(PathBuf, bool)>> {
        let mut results = Vec::new();

        for entry in std::fs::read_dir(&self.config.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let mut all_valid = true;

            for line in content.lines() {
                if line.is_empty() {
                    continue;
                }

                let event: AuditEvent = serde_json::from_str(line)?;
                if !event.verify(&self.config.secret_key) {
                    all_valid = false;
                    break;
                }
            }

            results.push((path, all_valid));
        }

        Ok(results)
    }

    /// Clean up old log files based on retention policy
    pub fn cleanup_old_logs(&self) -> anyhow::Result<usize> {
        let cutoff_date =
            OffsetDateTime::now_utc().date() - time::Duration::days(self.config.retention_days as i64);

        let mut deleted = 0;

        for entry in std::fs::read_dir(&self.config.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                // Parse date from filename (YYYY-MM-DD format)
                if let Ok(date) = time::Date::parse(
                    filename,
                    &time::format_description::parse("[year]-[month]-[day]").unwrap(),
                ) {
                    if date < cutoff_date {
                        std::fs::remove_file(&path)?;
                        deleted += 1;
                    }
                }
            }
        }

        Ok(deleted)
    }
}

/// Helper macros for common audit events
#[macro_export]
macro_rules! audit_auth_success {
    ($logger:expr, $user:expr) => {
        $logger.log(AuditEvent::new(
            AuditEventType::Authentication,
            $user,
            "login",
            "authentication",
            AuditResult::Success,
        ))
    };
}

#[macro_export]
macro_rules! audit_auth_failure {
    ($logger:expr, $user:expr, $reason:expr) => {
        $logger.log(AuditEvent::new(
            AuditEventType::Authentication,
            $user,
            "login",
            "authentication",
            AuditResult::Failure($reason.to_string()),
        ))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "alice@example.com",
            "login",
            "/api/login",
            AuditResult::Success,
        );

        assert_eq!(event.actor, "alice@example.com");
        assert_eq!(event.action, "login");
        assert_eq!(event.event_type, AuditEventType::Authentication);
    }

    #[test]
    fn test_audit_event_signing() {
        let mut event = AuditEvent::new(
            AuditEventType::Authentication,
            "alice@example.com",
            "login",
            "/api/login",
            AuditResult::Success,
        );

        let secret = b"test-secret-key";
        event.sign(secret);

        assert!(!event.signature.is_empty());
        assert!(event.verify(secret));
    }

    #[test]
    fn test_audit_event_tamper_detection() {
        let mut event = AuditEvent::new(
            AuditEventType::Authentication,
            "alice@example.com",
            "login",
            "/api/login",
            AuditResult::Success,
        );

        let secret = b"test-secret-key";
        event.sign(secret);

        // Tamper with event
        event.actor = "eve@example.com".to_string();

        // Verification should fail
        assert!(!event.verify(secret));
    }

    #[test]
    fn test_audit_logger() {
        let temp_dir = TempDir::new().unwrap();

        let config = AuditLoggerConfig {
            log_dir: temp_dir.path().to_path_buf(),
            secret_key: b"test-secret".to_vec(),
            rotate_daily: false,
            retention_days: 90,
        };

        let logger = AuditLogger::new(config).unwrap();

        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "alice@example.com",
            "login",
            "/api/login",
            AuditResult::Success,
        );

        logger.log(event).unwrap();

        // Read back the event
        let events = logger.read_log(None).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].actor, "alice@example.com");
    }

    #[test]
    fn test_log_integrity_verification() {
        let temp_dir = TempDir::new().unwrap();

        let config = AuditLoggerConfig {
            log_dir: temp_dir.path().to_path_buf(),
            secret_key: b"test-secret".to_vec(),
            rotate_daily: false,
            retention_days: 90,
        };

        let logger = AuditLogger::new(config).unwrap();

        // Log multiple events
        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::Authentication,
                &format!("user{}@example.com", i),
                "login",
                "/api/login",
                AuditResult::Success,
            );
            logger.log(event).unwrap();
        }

        // Verify integrity
        let results = logger.verify_integrity().unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1); // All events should be valid
    }
}
