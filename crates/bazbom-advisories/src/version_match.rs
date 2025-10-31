use crate::merge::{VersionEvent, VersionRange};
use anyhow::{Context, Result};
use semver::Version;

/// Check if a version is affected by any of the given version ranges
pub fn is_version_affected(version: &str, ranges: &[VersionRange]) -> Result<bool> {
    for range in ranges {
        match range.range_type.as_str() {
            "SEMVER" => {
                if is_version_affected_semver(version, range)? {
                    return Ok(true);
                }
            }
            "ECOSYSTEM" => {
                // For ECOSYSTEM type, we need to know the ecosystem to properly parse versions
                // For now, try semver first, fall back to string comparison
                if is_version_affected_semver(version, range).unwrap_or(false) {
                    return Ok(true);
                } else if is_version_affected_string(version, range) {
                    return Ok(true);
                }
            }
            "GIT" => {
                // GIT ranges use commit SHAs, just do string comparison
                if is_version_affected_string(version, range) {
                    return Ok(true);
                }
            }
            _ => {
                // Unknown range type - log and be conservative
                eprintln!("[bazbom] warning: unknown version range type '{}', assuming affected", range.range_type);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Check if a semver version is affected by a version range
fn is_version_affected_semver(version_str: &str, range: &VersionRange) -> Result<bool> {
    let version = Version::parse(version_str)
        .with_context(|| format!("failed to parse version: {}", version_str))?;

    let mut introduced: Option<Version> = None;
    let mut fixed: Option<Version> = None;
    let mut last_affected: Option<Version> = None;

    for event in &range.events {
        match event {
            VersionEvent::Introduced { introduced: v } => {
                if v == "0" {
                    // Special case: "0" means from the beginning
                    introduced = Some(Version::new(0, 0, 0));
                } else {
                    introduced = Version::parse(v).ok();
                }
            }
            VersionEvent::Fixed { fixed: v } => {
                fixed = Version::parse(v).ok();
            }
            VersionEvent::LastAffected { last_affected: v } => {
                last_affected = Version::parse(v).ok();
            }
        }
    }

    // Check if version is in the affected range
    let after_introduced = introduced.map_or(true, |intro| version >= intro);
    
    let before_fixed = if let Some(fix) = fixed {
        version < fix
    } else if let Some(last) = last_affected {
        version <= last
    } else {
        // No upper bound specified, assume affected if after introduced
        true
    };

    Ok(after_introduced && before_fixed)
}

/// Simple string-based version comparison for non-semver ecosystems
/// Note: This uses lexicographic comparison which is unreliable for numeric versions
/// (e.g., "10.0.0" < "2.0.0" lexicographically). This is a known limitation for
/// ecosystems that don't follow semver. In practice, this means we may produce
/// false positives for GIT ranges or non-standard version schemes.
fn is_version_affected_string(version: &str, range: &VersionRange) -> bool {
    let mut introduced: Option<&str> = None;
    let mut fixed: Option<&str> = None;
    let mut last_affected: Option<&str> = None;

    for event in &range.events {
        match event {
            VersionEvent::Introduced { introduced: v } => {
                introduced = Some(v.as_str());
            }
            VersionEvent::Fixed { fixed: v } => {
                fixed = Some(v.as_str());
            }
            VersionEvent::LastAffected { last_affected: v } => {
                last_affected = Some(v.as_str());
            }
        }
    }

    // Very conservative string matching with lexicographic comparison
    // This is inherently unreliable for numeric versions but works for GIT hashes
    if let Some(intro) = introduced {
        if intro != "0" && version < intro {
            return false;
        }
    }

    if let Some(fix) = fixed {
        if version >= fix {
            return false;
        }
    }

    if let Some(last) = last_affected {
        if version > last {
            return false;
        }
    }

    // If we get here and have some bounds, version might be affected
    introduced.is_some() || fixed.is_some() || last_affected.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_range(range_type: &str, events: Vec<VersionEvent>) -> VersionRange {
        VersionRange {
            range_type: range_type.to_string(),
            events,
        }
    }

    #[test]
    fn test_semver_version_affected_in_range() {
        let range = make_range(
            "SEMVER",
            vec![
                VersionEvent::Introduced {
                    introduced: "1.0.0".to_string(),
                },
                VersionEvent::Fixed {
                    fixed: "2.0.0".to_string(),
                },
            ],
        );

        assert!(is_version_affected("1.5.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("1.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("1.99.99", &[range.clone()]).unwrap());
        assert!(!is_version_affected("2.0.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("0.9.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("2.1.0", &[range]).unwrap());
    }

    #[test]
    fn test_semver_version_affected_from_zero() {
        let range = make_range(
            "SEMVER",
            vec![
                VersionEvent::Introduced {
                    introduced: "0".to_string(),
                },
                VersionEvent::Fixed {
                    fixed: "1.5.0".to_string(),
                },
            ],
        );

        assert!(is_version_affected("0.1.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("1.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("1.4.9", &[range.clone()]).unwrap());
        assert!(!is_version_affected("1.5.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("2.0.0", &[range]).unwrap());
    }

    #[test]
    fn test_semver_version_affected_last_affected() {
        let range = make_range(
            "SEMVER",
            vec![
                VersionEvent::Introduced {
                    introduced: "1.0.0".to_string(),
                },
                VersionEvent::LastAffected {
                    last_affected: "2.5.0".to_string(),
                },
            ],
        );

        assert!(is_version_affected("1.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("2.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("2.5.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("2.5.1", &[range.clone()]).unwrap());
        assert!(!is_version_affected("0.9.0", &[range]).unwrap());
    }

    #[test]
    fn test_semver_open_ended_range() {
        let range = make_range(
            "SEMVER",
            vec![VersionEvent::Introduced {
                introduced: "1.0.0".to_string(),
            }],
        );

        assert!(is_version_affected("1.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("2.0.0", &[range.clone()]).unwrap());
        assert!(is_version_affected("99.0.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("0.9.0", &[range]).unwrap());
    }

    #[test]
    fn test_multiple_ranges() {
        let ranges = vec![
            make_range(
                "SEMVER",
                vec![
                    VersionEvent::Introduced {
                        introduced: "1.0.0".to_string(),
                    },
                    VersionEvent::Fixed {
                        fixed: "1.5.0".to_string(),
                    },
                ],
            ),
            make_range(
                "SEMVER",
                vec![
                    VersionEvent::Introduced {
                        introduced: "2.0.0".to_string(),
                    },
                    VersionEvent::Fixed {
                        fixed: "2.5.0".to_string(),
                    },
                ],
            ),
        ];

        // Version in first range
        assert!(is_version_affected("1.2.0", &ranges).unwrap());
        // Version in second range
        assert!(is_version_affected("2.3.0", &ranges).unwrap());
        // Version between ranges
        assert!(!is_version_affected("1.7.0", &ranges).unwrap());
        // Version before all ranges
        assert!(!is_version_affected("0.9.0", &ranges).unwrap());
        // Version after all ranges
        assert!(!is_version_affected("3.0.0", &ranges).unwrap());
    }

    #[test]
    fn test_ecosystem_type_falls_back_to_semver() {
        let range = make_range(
            "ECOSYSTEM",
            vec![
                VersionEvent::Introduced {
                    introduced: "1.0.0".to_string(),
                },
                VersionEvent::Fixed {
                    fixed: "2.0.0".to_string(),
                },
            ],
        );

        assert!(is_version_affected("1.5.0", &[range.clone()]).unwrap());
        assert!(!is_version_affected("2.0.0", &[range]).unwrap());
    }

    #[test]
    fn test_invalid_semver_returns_error() {
        let range = make_range(
            "SEMVER",
            vec![
                VersionEvent::Introduced {
                    introduced: "1.0.0".to_string(),
                },
                VersionEvent::Fixed {
                    fixed: "2.0.0".to_string(),
                },
            ],
        );

        // Invalid version should return error
        assert!(is_version_affected("invalid", &[range]).is_err());
    }

    #[test]
    fn test_string_comparison_for_git_ranges() {
        let range = make_range(
            "GIT",
            vec![
                VersionEvent::Introduced {
                    introduced: "abc123".to_string(),
                },
                VersionEvent::Fixed {
                    fixed: "def456".to_string(),
                },
            ],
        );

        // For GIT ranges, we do conservative string matching
        assert!(is_version_affected("cde345", &[range]).unwrap());
    }

    #[test]
    fn test_empty_ranges_not_affected() {
        let ranges: Vec<VersionRange> = vec![];
        assert!(!is_version_affected("1.0.0", &ranges).unwrap());
    }
}
