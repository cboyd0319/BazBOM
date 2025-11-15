// Security validation for CLI arguments
//
// This module provides validation functions to prevent common security issues
// like path traversal, command injection, and malicious input.

use anyhow::{bail, Result};
use std::path::Path;

/// Validate a file or directory path argument
///
/// SECURITY: Validates that the path does not contain suspicious patterns
/// that could lead to path traversal or other security issues.
///
/// # Arguments
/// * `path` - The path to validate
/// * `must_exist` - If true, the path must exist
///
/// # Returns
/// * `Ok(())` if the path is valid
/// * `Err` if the path contains suspicious patterns
pub fn validate_path(path: &str, must_exist: bool) -> Result<()> {
    // Reject empty paths
    if path.trim().is_empty() {
        bail!("Path cannot be empty");
    }

    // Reject paths that are excessively long (potential buffer overflow)
    if path.len() > 4096 {
        bail!("Path too long: {} characters (max 4096)", path.len());
    }

    // Check for null bytes (path traversal attack)
    if path.contains('\0') {
        bail!("Path contains null byte");
    }

    // Warn about parent directory references in relative paths
    // Note: This is a warning, not an error, as ".." can be legitimate
    let path_obj = Path::new(path);
    let components: Vec<_> = path_obj.components().collect();
    let parent_count = components
        .iter()
        .filter(|c| matches!(c, std::path::Component::ParentDir))
        .count();

    if parent_count > 3 {
        eprintln!(
            "[!] WARNING: Path contains {} parent directory references: {}",
            parent_count, path
        );
    }

    // If must_exist is true, verify the path exists
    if must_exist && !path_obj.exists() {
        bail!("Path does not exist: {}", path);
    }

    Ok(())
}

/// Validate a profile name argument
///
/// SECURITY: Ensures profile names are alphanumeric to prevent injection attacks
///
/// # Arguments
/// * `profile` - The profile name to validate
///
/// # Returns
/// * `Ok(())` if the profile name is valid
/// * `Err` if the profile name contains suspicious characters
pub fn validate_profile_name(profile: &str) -> Result<()> {
    // Reject empty profile names
    if profile.trim().is_empty() {
        bail!("Profile name cannot be empty");
    }

    // Limit length
    if profile.len() > 64 {
        bail!("Profile name too long: {} characters (max 64)", profile.len());
    }

    // Allow only alphanumeric, dash, and underscore
    if !profile
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Profile name must be alphanumeric (with dash/underscore): {}", profile);
    }

    Ok(())
}

/// Validate a Bazel query expression
///
/// SECURITY: Basic validation to prevent command injection via Bazel queries
///
/// # Arguments
/// * `query` - The Bazel query to validate
///
/// # Returns
/// * `Ok(())` if the query is safe
/// * `Err` if the query contains suspicious patterns
pub fn validate_bazel_query(query: &str) -> Result<()> {
    // Reject empty queries
    if query.trim().is_empty() {
        bail!("Bazel query cannot be empty");
    }

    // Limit length
    if query.len() > 1024 {
        bail!("Bazel query too long: {} characters (max 1024)", query.len());
    }

    // Check for shell metacharacters that could indicate command injection
    // Bazel queries shouldn't contain these characters
    let dangerous_chars = ['$', '`', '&', '|', ';', '<', '>', '\n', '\r'];
    for ch in dangerous_chars {
        if query.contains(ch) {
            bail!("Bazel query contains suspicious character: '{}'", ch);
        }
    }

    Ok(())
}

/// Validate a Bazel target
///
/// SECURITY: Ensures Bazel targets follow expected format
///
/// # Arguments
/// * `target` - The Bazel target to validate
///
/// # Returns
/// * `Ok(())` if the target is valid
/// * `Err` if the target format is invalid
pub fn validate_bazel_target(target: &str) -> Result<()> {
    // Reject empty targets
    if target.trim().is_empty() {
        bail!("Bazel target cannot be empty");
    }

    // Limit length
    if target.len() > 512 {
        bail!("Bazel target too long: {} characters (max 512)", target.len());
    }

    // Bazel targets should start with // or @
    if !target.starts_with("//") && !target.starts_with("@") && !target.starts_with(":") {
        bail!("Bazel target must start with //, @, or :: {}", target);
    }

    // Check for suspicious characters
    let dangerous_chars = ['$', '`', '&', '|', ';', '<', '>', '\n', '\r'];
    for ch in dangerous_chars {
        if target.contains(ch) {
            bail!("Bazel target contains suspicious character: '{}'", ch);
        }
    }

    Ok(())
}

/// Validate an output format argument
///
/// # Arguments
/// * `format` - The format to validate
///
/// # Returns
/// * `Ok(())` if the format is valid
/// * `Err` if the format is not supported
pub fn validate_output_format(format: &str) -> Result<()> {
    let valid_formats = ["spdx", "cyclonedx", "sarif", "json"];
    if !valid_formats.contains(&format.to_lowercase().as_str()) {
        bail!(
            "Invalid output format: {}. Valid formats: {}",
            format,
            valid_formats.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_basic() {
        assert!(validate_path(".", false).is_ok());
        assert!(validate_path("src/main.rs", false).is_ok());
        assert!(validate_path("/absolute/path", false).is_ok());
    }

    #[test]
    fn test_validate_path_empty() {
        assert!(validate_path("", false).is_err());
        assert!(validate_path("   ", false).is_err());
    }

    #[test]
    fn test_validate_path_null_byte() {
        assert!(validate_path("path\0with\0null", false).is_err());
    }

    #[test]
    fn test_validate_profile_name() {
        assert!(validate_profile_name("production").is_ok());
        assert!(validate_profile_name("dev-strict").is_ok());
        assert!(validate_profile_name("ci_pipeline").is_ok());
    }

    #[test]
    fn test_validate_profile_name_invalid() {
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("profile with spaces").is_err());
        assert!(validate_profile_name("profile;injection").is_err());
    }

    #[test]
    fn test_validate_bazel_query() {
        assert!(validate_bazel_query("//...").is_ok());
        assert!(validate_bazel_query("kind(java_library, //...)").is_ok());
    }

    #[test]
    fn test_validate_bazel_query_injection() {
        assert!(validate_bazel_query("//...; rm -rf /").is_err());
        assert!(validate_bazel_query("$(malicious)").is_err());
        assert!(validate_bazel_query("query | bash").is_err());
    }

    #[test]
    fn test_validate_bazel_target() {
        assert!(validate_bazel_target("//src:main").is_ok());
        assert!(validate_bazel_target("@maven//:guava").is_ok());
        assert!(validate_bazel_target(":local_target").is_ok());
    }

    #[test]
    fn test_validate_output_format() {
        assert!(validate_output_format("spdx").is_ok());
        assert!(validate_output_format("cyclonedx").is_ok());
        assert!(validate_output_format("SPDX").is_ok()); // case insensitive
        assert!(validate_output_format("invalid").is_err());
    }
}
