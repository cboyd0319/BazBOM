//! SQLite database for tracking Jira tickets and GitHub PRs
//!
//! This module provides duplicate detection and tracking for automated remediation.
//! It prevents creating duplicate tickets/PRs and tracks remediation status.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;

/// Remediation tracking database
pub struct RemediationDatabase {
    conn: Connection,
}

impl RemediationDatabase {
    /// Create or open the remediation database
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path).context("Failed to open remediation database")?;

        let mut db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Create an in-memory database (for testing)
    #[cfg(test)]
    fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().context("Failed to open in-memory database")?;

        let mut db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    /// Get the database file path (~/.bazbom/remediation.db)
    fn get_db_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;

        Ok(PathBuf::from(home).join(".bazbom").join("remediation.db"))
    }

    /// Initialize database schema
    fn initialize_schema(&mut self) -> Result<()> {
        // Create jira_issues table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS jira_issues (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cve_id TEXT NOT NULL,
                package TEXT NOT NULL,
                version TEXT NOT NULL,
                jira_key TEXT NOT NULL UNIQUE,
                jira_url TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(cve_id, package, version)
            )",
            [],
        )?;

        // Create github_prs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS github_prs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cve_id TEXT NOT NULL,
                package TEXT NOT NULL,
                version TEXT NOT NULL,
                owner TEXT NOT NULL,
                repo TEXT NOT NULL,
                pr_number INTEGER NOT NULL,
                pr_url TEXT NOT NULL,
                state TEXT NOT NULL,
                jira_key TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(cve_id, package, version, owner, repo)
            )",
            [],
        )?;

        // Create sync_log table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                details TEXT,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_jira_cve ON jira_issues(cve_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_github_cve ON github_prs(cve_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_timestamp ON sync_log(timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    /// Check if a Jira issue already exists for this CVE/package/version
    pub fn jira_issue_exists(
        &self,
        cve_id: &str,
        package: &str,
        version: &str,
    ) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT jira_key FROM jira_issues WHERE cve_id = ? AND package = ? AND version = ?",
        )?;

        let result = stmt.query_row(params![cve_id, package, version], |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(jira_key) => Ok(Some(jira_key)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Record a created Jira issue
    pub fn record_jira_issue(
        &self,
        cve_id: &str,
        package: &str,
        version: &str,
        jira_key: &str,
        jira_url: &str,
        status: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO jira_issues (cve_id, package, version, jira_key, jira_url, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![cve_id, package, version, jira_key, jira_url, status, &now, &now],
        )?;

        self.log_sync(
            "jira",
            jira_key,
            "created",
            Some(&format!("Created for {} in {}", cve_id, package)),
        )?;

        Ok(())
    }

    /// Check if a GitHub PR already exists for this CVE/package/version/repo
    pub fn github_pr_exists(
        &self,
        cve_id: &str,
        package: &str,
        version: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Option<u64>> {
        let mut stmt = self.conn.prepare(
            "SELECT pr_number FROM github_prs
             WHERE cve_id = ? AND package = ? AND version = ? AND owner = ? AND repo = ?",
        )?;

        let result = stmt.query_row(params![cve_id, package, version, owner, repo], |row| {
            row.get::<_, i64>(0)
        });

        match result {
            Ok(pr_number) => Ok(Some(pr_number as u64)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Record a created GitHub PR
    #[allow(clippy::too_many_arguments)]
    pub fn record_github_pr(
        &self,
        cve_id: &str,
        package: &str,
        version: &str,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_url: &str,
        state: &str,
        jira_key: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO github_prs
             (cve_id, package, version, owner, repo, pr_number, pr_url, state, jira_key, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![cve_id, package, version, owner, repo, pr_number as i64, pr_url, state, jira_key, &now, &now],
        )?;

        self.log_sync(
            "github_pr",
            &pr_number.to_string(),
            "created",
            Some(&format!(
                "Created for {} in {} ({}/{})",
                cve_id, package, owner, repo
            )),
        )?;

        Ok(())
    }

    /// Update Jira issue status
    pub fn update_jira_status(&self, jira_key: &str, new_status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE jira_issues SET status = ?, updated_at = ? WHERE jira_key = ?",
            params![new_status, &now, jira_key],
        )?;

        self.log_sync("jira", jira_key, "status_updated", Some(new_status))?;

        Ok(())
    }

    /// Update GitHub PR state
    pub fn update_pr_state(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        new_state: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "UPDATE github_prs SET state = ?, updated_at = ? WHERE owner = ? AND repo = ? AND pr_number = ?",
            params![new_state, &now, owner, repo, pr_number as i64],
        )?;

        self.log_sync(
            "github_pr",
            &pr_number.to_string(),
            "state_updated",
            Some(new_state),
        )?;

        Ok(())
    }

    /// Get all Jira issues
    pub fn get_all_jira_issues(&self) -> Result<Vec<JiraIssueRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT cve_id, package, version, jira_key, jira_url, status, created_at, updated_at
             FROM jira_issues
             ORDER BY created_at DESC",
        )?;

        let issues = stmt
            .query_map([], |row| {
                Ok(JiraIssueRecord {
                    cve_id: row.get(0)?,
                    package: row.get(1)?,
                    version: row.get(2)?,
                    jira_key: row.get(3)?,
                    jira_url: row.get(4)?,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(issues)
    }

    /// Get all GitHub PRs
    pub fn get_all_github_prs(&self) -> Result<Vec<GitHubPrRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT cve_id, package, version, owner, repo, pr_number, pr_url, state, jira_key, created_at, updated_at
             FROM github_prs
             ORDER BY created_at DESC"
        )?;

        let prs = stmt
            .query_map([], |row| {
                Ok(GitHubPrRecord {
                    cve_id: row.get(0)?,
                    package: row.get(1)?,
                    version: row.get(2)?,
                    owner: row.get(3)?,
                    repo: row.get(4)?,
                    pr_number: row.get::<_, i64>(5)? as u64,
                    pr_url: row.get(6)?,
                    state: row.get(7)?,
                    jira_key: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(prs)
    }

    /// Log a sync event
    fn log_sync(
        &self,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO sync_log (entity_type, entity_id, action, details, timestamp)
             VALUES (?, ?, ?, ?, ?)",
            params![entity_type, entity_id, action, details, &now],
        )?;

        Ok(())
    }

    /// Get recent sync events
    pub fn get_recent_sync_events(&self, limit: usize) -> Result<Vec<SyncLogRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT entity_type, entity_id, action, details, timestamp
             FROM sync_log
             ORDER BY timestamp DESC
             LIMIT ?",
        )?;

        let events = stmt
            .query_map(params![limit], |row| {
                Ok(SyncLogRecord {
                    entity_type: row.get(0)?,
                    entity_id: row.get(1)?,
                    action: row.get(2)?,
                    details: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }
}

/// Jira issue database record
#[derive(Debug, Clone)]
pub struct JiraIssueRecord {
    pub cve_id: String,
    pub package: String,
    pub version: String,
    pub jira_key: String,
    pub jira_url: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub PR database record
#[derive(Debug, Clone)]
pub struct GitHubPrRecord {
    pub cve_id: String,
    pub package: String,
    pub version: String,
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub pr_url: String,
    pub state: String,
    pub jira_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Sync log database record
#[derive(Debug, Clone)]
pub struct SyncLogRecord {
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub details: Option<String>,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = RemediationDatabase::new_in_memory().unwrap();

        // Verify tables exist by querying them
        let jira_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM jira_issues", [], |row| row.get(0))
            .unwrap();
        assert_eq!(jira_count, 0);

        let pr_count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM github_prs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(pr_count, 0);
    }

    #[test]
    fn test_jira_duplicate_detection() {
        let db = RemediationDatabase::new_in_memory().unwrap();

        // First check - should not exist
        let exists = db
            .jira_issue_exists("CVE-2024-1234", "log4j-core", "2.14.0")
            .unwrap();
        assert!(exists.is_none());

        // Record issue
        db.record_jira_issue(
            "CVE-2024-1234",
            "log4j-core",
            "2.14.0",
            "SEC-123",
            "https://jira.example.com/browse/SEC-123",
            "Open",
        )
        .unwrap();

        // Second check - should exist
        let exists = db
            .jira_issue_exists("CVE-2024-1234", "log4j-core", "2.14.0")
            .unwrap();
        assert_eq!(exists, Some("SEC-123".to_string()));
    }

    #[test]
    fn test_github_pr_duplicate_detection() {
        let db = RemediationDatabase::new_in_memory().unwrap();

        // First check - should not exist
        let exists = db
            .github_pr_exists("CVE-2024-1234", "log4j-core", "2.14.0", "myorg", "myrepo")
            .unwrap();
        assert!(exists.is_none());

        // Record PR
        db.record_github_pr(
            "CVE-2024-1234",
            "log4j-core",
            "2.14.0",
            "myorg",
            "myrepo",
            42,
            "https://github.com/myorg/myrepo/pull/42",
            "open",
            Some("SEC-123"),
        )
        .unwrap();

        // Second check - should exist
        let exists = db
            .github_pr_exists("CVE-2024-1234", "log4j-core", "2.14.0", "myorg", "myrepo")
            .unwrap();
        assert_eq!(exists, Some(42));
    }
}
