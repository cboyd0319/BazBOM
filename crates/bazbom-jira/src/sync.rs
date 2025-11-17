use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Bidirectional sync engine for Jira ↔ BazBOM
pub struct SyncEngine {
    /// Sync configuration
    enabled: bool,

    /// Sync state storage
    state: Arc<RwLock<SyncState>>,
}

/// Sync state tracking
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    /// Map of CVE ID -> Jira issue key
    cve_to_jira: HashMap<String, String>,

    /// Map of Jira issue key -> CVE ID
    jira_to_cve: HashMap<String, String>,

    /// Last sync timestamp per issue
    last_sync: HashMap<String, i64>,
}

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Sync from Jira to BazBOM
    FromJira,

    /// Sync from BazBOM to Jira
    ToJira,

    /// Bidirectional sync
    Bidirectional,
}

/// Sync event from Jira webhook
#[derive(Debug, Clone)]
pub struct JiraSyncEvent {
    /// Issue key
    pub issue_key: String,

    /// Event type
    pub event_type: JiraEventType,

    /// New status (if status changed)
    pub new_status: Option<String>,

    /// Comment added (if comment event)
    pub comment: Option<String>,

    /// Timestamp
    pub timestamp: i64,
}

/// Jira webhook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JiraEventType {
    /// Issue created
    IssueCreated,

    /// Issue updated
    IssueUpdated,

    /// Issue deleted
    IssueDeleted,

    /// Comment added
    CommentAdded,

    /// Status changed
    StatusChanged,

    /// Assignment changed
    AssignmentChanged,
}

/// BazBOM sync event
#[derive(Debug, Clone)]
pub struct BazBomSyncEvent {
    /// CVE ID
    pub cve_id: String,

    /// Event type
    pub event_type: BazBomEventType,

    /// New status
    pub status: VulnerabilityStatus,

    /// Additional context
    pub context: Option<String>,
}

/// BazBOM event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BazBomEventType {
    /// Vulnerability discovered
    VulnerabilityDiscovered,

    /// Vulnerability fixed
    VulnerabilityFixed,

    /// Fix verified
    FixVerified,

    /// Severity changed
    SeverityChanged,
}

/// Vulnerability status in BazBOM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VulnerabilityStatus {
    /// Open/active
    Open,

    /// In progress (fix being applied)
    InProgress,

    /// Fixed (patch applied)
    Fixed,

    /// Verified (fix confirmed)
    Verified,

    /// Accepted risk (VEX)
    AcceptedRisk,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            state: Arc::new(RwLock::new(SyncState::default())),
        }
    }

    /// Create sync engine with initial state
    pub fn with_state(enabled: bool, state: SyncState) -> Self {
        Self {
            enabled,
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Register a CVE -> Jira mapping
    pub async fn register_mapping(&self, cve_id: String, jira_key: String) {
        let mut state = self.state.write().await;
        state.cve_to_jira.insert(cve_id.clone(), jira_key.clone());
        state.jira_to_cve.insert(jira_key, cve_id);
    }

    /// Get Jira key for CVE
    pub async fn get_jira_key(&self, cve_id: &str) -> Option<String> {
        let state = self.state.read().await;
        state.cve_to_jira.get(cve_id).cloned()
    }

    /// Get CVE ID for Jira key
    pub async fn get_cve_id(&self, jira_key: &str) -> Option<String> {
        let state = self.state.read().await;
        state.jira_to_cve.get(jira_key).cloned()
    }

    /// Process Jira webhook event and sync to BazBOM
    pub async fn process_jira_event(&self, event: JiraSyncEvent) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Get CVE ID from issue key
        let cve_id = self.get_cve_id(&event.issue_key).await;

        match event.event_type {
            JiraEventType::StatusChanged => {
                if let Some(new_status) = &event.new_status {
                    self.handle_status_change(&event.issue_key, new_status, cve_id.as_deref())
                        .await?;
                }
            }
            JiraEventType::IssueDeleted => {
                self.handle_issue_deleted(&event.issue_key, cve_id.as_deref())
                    .await?;
            }
            JiraEventType::CommentAdded => {
                if let Some(comment) = &event.comment {
                    self.handle_comment_added(&event.issue_key, comment, cve_id.as_deref())
                        .await?;
                }
            }
            JiraEventType::IssueCreated | JiraEventType::IssueUpdated | JiraEventType::AssignmentChanged => {
                // These events can be handled for tracking but don't require immediate action
            }
        }

        // Update last sync timestamp
        let mut state = self.state.write().await;
        state.last_sync.insert(event.issue_key, event.timestamp);

        Ok(())
    }

    /// Handle Jira status change
    async fn handle_status_change(
        &self,
        issue_key: &str,
        new_status: &str,
        cve_id: Option<&str>,
    ) -> Result<()> {
        // Map Jira status to BazBOM vulnerability status
        let bazbom_status = match new_status.to_lowercase().as_str() {
            "to do" | "open" | "backlog" => VulnerabilityStatus::Open,
            "in progress" | "in review" => VulnerabilityStatus::InProgress,
            "done" | "closed" | "resolved" => VulnerabilityStatus::Fixed,
            "verified" => VulnerabilityStatus::Verified,
            "won't fix" | "accepted risk" => VulnerabilityStatus::AcceptedRisk,
            _ => VulnerabilityStatus::Open,
        };

        if let Some(cve) = cve_id {
            // In a real implementation, this would update BazBOM's internal database
            tracing::info!(
                "Syncing Jira status change: {} -> {:?} for CVE {}",
                issue_key,
                bazbom_status,
                cve
            );
        }

        Ok(())
    }

    /// Handle Jira issue deletion (treat as accepted risk/VEX)
    async fn handle_issue_deleted(&self, issue_key: &str, cve_id: Option<&str>) -> Result<()> {
        if let Some(cve) = cve_id {
            // In a real implementation, this would mark the vulnerability as accepted risk
            tracing::info!(
                "Jira issue deleted: {} - marking CVE {} as accepted risk",
                issue_key,
                cve
            );
        }

        // Remove from state
        let mut state = self.state.write().await;
        if let Some(cve) = cve_id {
            state.cve_to_jira.remove(cve);
        }
        state.jira_to_cve.remove(issue_key);
        state.last_sync.remove(issue_key);

        Ok(())
    }

    /// Handle comment added to Jira issue
    async fn handle_comment_added(
        &self,
        issue_key: &str,
        comment: &str,
        cve_id: Option<&str>,
    ) -> Result<()> {
        if let Some(cve) = cve_id {
            // In a real implementation, this would store remediation notes
            tracing::info!(
                "Comment added to {}: {} (CVE: {})",
                issue_key,
                comment,
                cve
            );
        }

        Ok(())
    }

    /// Sync BazBOM event to Jira
    pub async fn process_bazbom_event(&self, event: BazBomSyncEvent) -> Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        match event.event_type {
            BazBomEventType::VulnerabilityFixed => {
                self.handle_vulnerability_fixed(&event.cve_id).await?;
            }
            BazBomEventType::FixVerified => {
                self.handle_fix_verified(&event.cve_id).await?;
            }
            BazBomEventType::SeverityChanged => {
                self.handle_severity_changed(&event.cve_id, event.context.as_deref())
                    .await?;
            }
            BazBomEventType::VulnerabilityDiscovered => {
                // This would trigger ticket creation, handled separately
            }
        }

        Ok(self.get_jira_key(&event.cve_id).await)
    }

    /// Handle vulnerability fixed in BazBOM
    async fn handle_vulnerability_fixed(&self, cve_id: &str) -> Result<()> {
        if let Some(jira_key) = self.get_jira_key(cve_id).await {
            tracing::info!(
                "Vulnerability {} fixed - should update Jira {} to 'Done'",
                cve_id,
                jira_key
            );
            // In real implementation: transition Jira issue to "Done" status
            // Add comment with fix details (PR link, commit hash, etc.)
        }

        Ok(())
    }

    /// Handle fix verification in BazBOM
    async fn handle_fix_verified(&self, cve_id: &str) -> Result<()> {
        if let Some(jira_key) = self.get_jira_key(cve_id).await {
            tracing::info!(
                "Fix verified for {} - should update Jira {} to 'Verified'",
                cve_id,
                jira_key
            );
            // In real implementation: transition to "Verified" status
            // Add comment with verification details
        }

        Ok(())
    }

    /// Handle severity change in BazBOM
    async fn handle_severity_changed(&self, cve_id: &str, new_severity: Option<&str>) -> Result<()> {
        if let Some(jira_key) = self.get_jira_key(cve_id).await {
            tracing::info!(
                "Severity changed for {} - should update Jira {} priority",
                cve_id,
                jira_key
            );
            // In real implementation: update Jira priority field
            // Add comment explaining severity change
            if let Some(severity) = new_severity {
                tracing::info!("New severity: {}", severity);
            }
        }

        Ok(())
    }

    /// Get sync statistics
    pub async fn get_stats(&self) -> SyncStats {
        let state = self.state.read().await;
        SyncStats {
            total_mappings: state.cve_to_jira.len(),
            enabled: self.enabled,
        }
    }
}

/// Sync statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    /// Total CVE -> Jira mappings
    pub total_mappings: usize,

    /// Sync enabled
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_engine_creation() {
        let engine = SyncEngine::new(true);
        let stats = engine.get_stats().await;
        assert!(stats.enabled);
        assert_eq!(stats.total_mappings, 0);
    }

    #[tokio::test]
    async fn test_register_mapping() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let jira_key = engine.get_jira_key("CVE-2024-1234").await;
        assert_eq!(jira_key, Some("SEC-567".to_string()));

        let cve_id = engine.get_cve_id("SEC-567").await;
        assert_eq!(cve_id, Some("CVE-2024-1234".to_string()));

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_mappings, 1);
    }

    #[tokio::test]
    async fn test_process_jira_status_change() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let event = JiraSyncEvent {
            issue_key: "SEC-567".to_string(),
            event_type: JiraEventType::StatusChanged,
            new_status: Some("Done".to_string()),
            comment: None,
            timestamp: 1234567890,
        };

        let result = engine.process_jira_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_jira_issue_deleted() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let event = JiraSyncEvent {
            issue_key: "SEC-567".to_string(),
            event_type: JiraEventType::IssueDeleted,
            new_status: None,
            comment: None,
            timestamp: 1234567890,
        };

        engine.process_jira_event(event).await.unwrap();

        // Mapping should be removed
        let jira_key = engine.get_jira_key("CVE-2024-1234").await;
        assert_eq!(jira_key, None);

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_mappings, 0);
    }

    #[tokio::test]
    async fn test_process_bazbom_vulnerability_fixed() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let event = BazBomSyncEvent {
            cve_id: "CVE-2024-1234".to_string(),
            event_type: BazBomEventType::VulnerabilityFixed,
            status: VulnerabilityStatus::Fixed,
            context: None,
        };

        let result = engine.process_bazbom_event(event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("SEC-567".to_string()));
    }

    #[tokio::test]
    async fn test_disabled_sync_engine() {
        let engine = SyncEngine::new(false);

        let event = BazBomSyncEvent {
            cve_id: "CVE-2024-1234".to_string(),
            event_type: BazBomEventType::VulnerabilityFixed,
            status: VulnerabilityStatus::Fixed,
            context: None,
        };

        let result = engine.process_bazbom_event(event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_multiple_mappings() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;
        engine
            .register_mapping("CVE-2024-5678".to_string(), "SEC-568".to_string())
            .await;
        engine
            .register_mapping("CVE-2024-9012".to_string(), "SEC-569".to_string())
            .await;

        let stats = engine.get_stats().await;
        assert_eq!(stats.total_mappings, 3);
    }

    #[tokio::test]
    async fn test_comment_event() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let event = JiraSyncEvent {
            issue_key: "SEC-567".to_string(),
            event_type: JiraEventType::CommentAdded,
            new_status: None,
            comment: Some("Manual remediation applied".to_string()),
            timestamp: 1234567890,
        };

        let result = engine.process_jira_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_severity_changed_event() {
        let engine = SyncEngine::new(true);

        engine
            .register_mapping("CVE-2024-1234".to_string(), "SEC-567".to_string())
            .await;

        let event = BazBomSyncEvent {
            cve_id: "CVE-2024-1234".to_string(),
            event_type: BazBomEventType::SeverityChanged,
            status: VulnerabilityStatus::Open,
            context: Some("HIGH".to_string()),
        };

        let result = engine.process_bazbom_event(event).await;
        assert!(result.is_ok());
    }
}

