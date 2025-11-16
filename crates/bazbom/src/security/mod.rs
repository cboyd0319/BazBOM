//! Security utilities for BazBOM
//!
//! This module provides security-related helper functions including:
//! - Log sanitization (prevent log injection attacks)
//! - Audit logging (security event tracking)
//! - Input validation

pub mod audit_log;
pub mod log_sanitizer;

pub use audit_log::{AuditEventType, AuditLogEntry, AuditLogger, AuditResult};
pub use log_sanitizer::sanitize_for_log;
