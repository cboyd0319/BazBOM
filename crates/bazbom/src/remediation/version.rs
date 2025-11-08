// Version parsing utilities for remediation

/// Parse semantic version strings into (major, minor, patch) tuple
///
/// # Examples
/// ```
/// use bazbom::remediation::version::parse_semantic_version;
///
/// assert_eq!(parse_semantic_version("1.2.3"), Some((1, 2, 3)));
/// assert_eq!(parse_semantic_version("1.2.3-SNAPSHOT"), Some((1, 2, 3)));
/// assert_eq!(parse_semantic_version("invalid"), None);
/// ```
#[allow(clippy::should_implement_trait)]
pub fn parse_semantic_version(version: &str) -> Option<(u32, u32, u32)> {
    let clean_version = version.split('-').next()?;
    let parts: Vec<&str> = clean_version.split('.').collect();

    if parts.len() < 3 {
        return None;
    }

    let parse_part = |s: &str| -> Option<u32> {
        if s.chars().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };

    let major = parse_part(parts[0])?;
    let minor = parse_part(parts[1])?;
    let patch = parse_part(parts[2])?;

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semantic_version() {
        assert_eq!(parse_semantic_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semantic_version("0.0.1"), Some((0, 0, 1)));
        assert_eq!(parse_semantic_version("10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn test_parse_semantic_version_with_suffix() {
        assert_eq!(parse_semantic_version("1.2.3-SNAPSHOT"), Some((1, 2, 3)));
        assert_eq!(parse_semantic_version("1.2.3-alpha"), Some((1, 2, 3)));
        assert_eq!(parse_semantic_version("1.2.3-beta.1"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_semantic_version_invalid() {
        assert_eq!(parse_semantic_version("1.2"), None);
        assert_eq!(parse_semantic_version("1"), None);
        assert_eq!(parse_semantic_version("invalid"), None);
        assert_eq!(parse_semantic_version("1.2.x"), None);
    }
}
