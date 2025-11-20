//! OS detection from container filesystem

use anyhow::{Context, Result};
use std::path::Path;
use tracing::debug;

/// Detected OS type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsType {
    Alpine,
    Debian,
    Ubuntu,
    Rhel,
    CentOS,
    Fedora,
    Unknown(String),
}

/// OS information
#[derive(Debug, Clone)]
pub struct OsInfo {
    pub os_type: OsType,
    pub version: String,
    pub version_id: String,
    pub pretty_name: String,
}

/// Detect OS from extracted container filesystem
///
/// Reads /etc/os-release to determine the OS type and version
pub fn detect_os(root: &Path) -> Result<OsInfo> {
    let os_release_path = root.join("etc/os-release");

    if !os_release_path.exists() {
        // Try alternate location
        let alt_path = root.join("usr/lib/os-release");
        if alt_path.exists() {
            return parse_os_release(&alt_path);
        }

        // Check for Alpine-specific file
        let alpine_release = root.join("etc/alpine-release");
        if alpine_release.exists() {
            let version = std::fs::read_to_string(&alpine_release)
                .context("Failed to read alpine-release")?
                .trim()
                .to_string();
            return Ok(OsInfo {
                os_type: OsType::Alpine,
                version: version.clone(),
                version_id: version,
                pretty_name: "Alpine Linux".to_string(),
            });
        }

        anyhow::bail!("Could not find os-release file");
    }

    parse_os_release(&os_release_path)
}

fn parse_os_release(path: &Path) -> Result<OsInfo> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read os-release")?;

    let mut id = String::new();
    let mut version_id = String::new();
    let mut pretty_name = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim_matches('"').trim_matches('\'');
            match key {
                "ID" => id = value.to_lowercase(),
                "VERSION_ID" => version_id = value.to_string(),
                "PRETTY_NAME" => pretty_name = value.to_string(),
                _ => {}
            }
        }
    }

    let os_type = match id.as_str() {
        "alpine" => OsType::Alpine,
        "debian" => OsType::Debian,
        "ubuntu" => OsType::Ubuntu,
        "rhel" => OsType::Rhel,
        "centos" => OsType::CentOS,
        "fedora" => OsType::Fedora,
        other => OsType::Unknown(other.to_string()),
    };

    debug!("Detected OS: {:?} version {}", os_type, version_id);

    Ok(OsInfo {
        os_type,
        version: version_id.clone(),
        version_id,
        pretty_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_debian_os_release() {
        let dir = TempDir::new().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();

        let mut f = std::fs::File::create(etc.join("os-release")).unwrap();
        writeln!(f, "ID=debian").unwrap();
        writeln!(f, "VERSION_ID=\"12\"").unwrap();
        writeln!(f, "PRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"").unwrap();

        let info = detect_os(dir.path()).unwrap();
        assert_eq!(info.os_type, OsType::Debian);
        assert_eq!(info.version_id, "12");
    }

    #[test]
    fn test_parse_alpine_release() {
        let dir = TempDir::new().unwrap();
        let etc = dir.path().join("etc");
        std::fs::create_dir_all(&etc).unwrap();

        std::fs::write(etc.join("alpine-release"), "3.19.1\n").unwrap();

        let info = detect_os(dir.path()).unwrap();
        assert_eq!(info.os_type, OsType::Alpine);
        assert_eq!(info.version, "3.19.1");
    }
}
