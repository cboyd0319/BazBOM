//! Package database parsers for different OS types

use anyhow::{Context, Result};
use std::path::Path;
use tracing::debug;

/// Installed package information
#[derive(Debug, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
    pub architecture: Option<String>,
    pub source: Option<String>,
}

/// Parse Alpine's /lib/apk/db/installed
pub fn parse_apk_installed(root: &Path) -> Result<Vec<InstalledPackage>> {
    let db_path = root.join("lib/apk/db/installed");

    if !db_path.exists() {
        return Ok(vec![]);
    }

    let content =
        std::fs::read_to_string(&db_path).context("Failed to read apk installed database")?;

    let mut packages = Vec::new();
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_arch = None;

    for line in content.lines() {
        if line.is_empty() {
            // End of package record
            if !current_name.is_empty() && !current_version.is_empty() {
                packages.push(InstalledPackage {
                    name: current_name.clone(),
                    version: current_version.clone(),
                    architecture: current_arch.clone(),
                    source: None,
                });
            }
            current_name.clear();
            current_version.clear();
            current_arch = None;
            continue;
        }

        if let Some(value) = line.strip_prefix("P:") {
            current_name = value.to_string();
        } else if let Some(value) = line.strip_prefix("V:") {
            current_version = value.to_string();
        } else if let Some(value) = line.strip_prefix("A:") {
            current_arch = Some(value.to_string());
        }
    }

    // Don't forget the last package
    if !current_name.is_empty() && !current_version.is_empty() {
        packages.push(InstalledPackage {
            name: current_name,
            version: current_version,
            architecture: current_arch,
            source: None,
        });
    }

    debug!("Parsed {} Alpine packages", packages.len());
    Ok(packages)
}

/// Parse Debian/Ubuntu's /var/lib/dpkg/status
pub fn parse_dpkg_status(root: &Path) -> Result<Vec<InstalledPackage>> {
    let status_path = root.join("var/lib/dpkg/status");

    if !status_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&status_path).context("Failed to read dpkg status")?;

    let mut packages = Vec::new();
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_arch = None;
    let mut current_source = None;
    let mut is_installed = false;

    for line in content.lines() {
        if line.is_empty() {
            // End of package record
            if is_installed && !current_name.is_empty() && !current_version.is_empty() {
                packages.push(InstalledPackage {
                    name: current_name.clone(),
                    version: current_version.clone(),
                    architecture: current_arch.clone(),
                    source: current_source.clone(),
                });
            }
            current_name.clear();
            current_version.clear();
            current_arch = None;
            current_source = None;
            is_installed = false;
            continue;
        }

        if let Some(value) = line.strip_prefix("Package: ") {
            current_name = value.to_string();
        } else if let Some(value) = line.strip_prefix("Version: ") {
            current_version = value.to_string();
        } else if let Some(value) = line.strip_prefix("Architecture: ") {
            current_arch = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Source: ") {
            // Source may have version in parens, strip it
            current_source = Some(value.split_whitespace().next().unwrap_or(value).to_string());
        } else if let Some(value) = line.strip_prefix("Status: ") {
            // Only include installed packages
            is_installed = value.contains("installed") && !value.contains("not-installed");
        }
    }

    // Don't forget the last package
    if is_installed && !current_name.is_empty() && !current_version.is_empty() {
        packages.push(InstalledPackage {
            name: current_name,
            version: current_version,
            architecture: current_arch,
            source: current_source,
        });
    }

    debug!("Parsed {} Debian/Ubuntu packages", packages.len());
    Ok(packages)
}

/// Parse RPM database
///
/// Note: Modern RHEL/Fedora use SQLite database at /var/lib/rpm/rpmdb.sqlite
/// Older versions use Berkeley DB at /var/lib/rpm/Packages
pub fn parse_rpm_database(root: &Path) -> Result<Vec<InstalledPackage>> {
    // Try SQLite first (modern)
    let sqlite_path = root.join("var/lib/rpm/rpmdb.sqlite");
    if sqlite_path.exists() {
        return parse_rpm_sqlite(&sqlite_path);
    }

    // Fall back to listing from /var/lib/rpm/Packages
    // This requires rpm tools, so we'll just detect that RPM packages exist
    let packages_path = root.join("var/lib/rpm/Packages");
    if packages_path.exists() {
        debug!("Found legacy RPM database, requires rpm tools to parse");
        // We can't easily parse Berkeley DB without rpm tools
        // Return empty and let external tools handle this
        return Ok(vec![]);
    }

    Ok(vec![])
}

fn parse_rpm_sqlite(path: &Path) -> Result<Vec<InstalledPackage>> {
    // SQLite parsing would require rusqlite dependency
    // For now, just detect it exists
    debug!("Found SQLite RPM database at {:?}", path);

    // TODO: Add rusqlite dependency and parse:
    // SELECT name, version, release, arch FROM Packages

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_apk_installed() {
        let dir = TempDir::new().unwrap();
        let db_dir = dir.path().join("lib/apk/db");
        std::fs::create_dir_all(&db_dir).unwrap();

        let mut f = std::fs::File::create(db_dir.join("installed")).unwrap();
        writeln!(f, "P:musl").unwrap();
        writeln!(f, "V:1.2.4-r2").unwrap();
        writeln!(f, "A:x86_64").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "P:busybox").unwrap();
        writeln!(f, "V:1.36.1-r5").unwrap();
        writeln!(f, "A:x86_64").unwrap();
        writeln!(f).unwrap();

        let packages = parse_apk_installed(dir.path()).unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "musl");
        assert_eq!(packages[0].version, "1.2.4-r2");
        assert_eq!(packages[1].name, "busybox");
    }

    #[test]
    fn test_parse_dpkg_status() {
        let dir = TempDir::new().unwrap();
        let dpkg_dir = dir.path().join("var/lib/dpkg");
        std::fs::create_dir_all(&dpkg_dir).unwrap();

        let mut f = std::fs::File::create(dpkg_dir.join("status")).unwrap();
        writeln!(f, "Package: openssl").unwrap();
        writeln!(f, "Status: install ok installed").unwrap();
        writeln!(f, "Version: 3.0.11-1~deb12u2").unwrap();
        writeln!(f, "Architecture: amd64").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "Package: removed-pkg").unwrap();
        writeln!(f, "Status: deinstall ok config-files").unwrap();
        writeln!(f, "Version: 1.0").unwrap();
        writeln!(f).unwrap();

        let packages = parse_dpkg_status(dir.path()).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "openssl");
        assert_eq!(packages[0].version, "3.0.11-1~deb12u2");
    }
}
