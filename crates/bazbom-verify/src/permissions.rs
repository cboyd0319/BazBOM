//! File permissions verification

use anyhow::Result;
use std::path::Path;

#[cfg(unix)]
pub fn check_permissions(binary_path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(binary_path)?;
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check for 755 permissions (rwxr-xr-x)
    // In Unix, 755 = 0o755 = 0b111_101_101
    let expected_mode = 0o755;
    let actual_mode = mode & 0o777; // Mask to get only permission bits

    Ok(actual_mode == expected_mode)
}

#[cfg(not(unix))]
pub fn check_permissions(_binary_path: &Path) -> Result<bool> {
    // On non-Unix systems (Windows), we can't check Unix permissions
    // Just return true to skip this check
    Ok(true)
}
