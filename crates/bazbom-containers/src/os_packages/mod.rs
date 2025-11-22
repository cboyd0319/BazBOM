//! Native OS package detection and vulnerability scanning
//!
//! Parses OS package databases from container filesystems and looks up
//! vulnerabilities using native BazBOM crates (no external tools required).

mod detection;
mod parsers;
mod scanner;

pub use detection::{detect_os, OsInfo, OsType};
pub use parsers::{parse_apk_installed, parse_dpkg_status, parse_rpm_database, InstalledPackage};
pub use scanner::{scan_os_packages, OsScanResult, OsVulnerability};
