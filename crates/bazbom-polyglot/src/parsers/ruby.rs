//! Ruby Bundler parser
//!
//! Parses Gemfile and Gemfile.lock files

use crate::detection::Ecosystem;
use crate::ecosystems::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

/// Scan Ruby ecosystem
pub async fn scan(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(
        "Ruby".to_string(),
        ecosystem.root_path.display().to_string(),
    );

    // Parse Gemfile.lock if available (most accurate)
    if let Some(ref lockfile_path) = ecosystem.lockfile {
        parse_gemfile_lock(lockfile_path, &mut result)?;
    } else if let Some(ref manifest_path) = ecosystem.manifest_file {
        // Fallback to Gemfile (less accurate)
        eprintln!(
            "Warning: Gemfile found but no Gemfile.lock - run 'bundle lock' for accurate versions"
        );
        parse_gemfile(manifest_path, &mut result)?;
    }

    // Run reachability analysis
    if let Err(e) = analyze_reachability(ecosystem, &mut result) {
        eprintln!("Warning: Ruby reachability analysis failed: {}", e);
    }

    Ok(result)
}

/// Analyze reachability for Ruby project
fn analyze_reachability(ecosystem: &Ecosystem, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_ruby_reachability::analyze_ruby_project;

    let report = analyze_ruby_project(&ecosystem.root_path)?;
    let mut vulnerable_packages_reachable = HashMap::new();

    for package in &result.packages {
        let key = format!("{}@{}", package.name, package.version);
        vulnerable_packages_reachable.insert(key, !report.reachable_functions.is_empty());
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });
    Ok(())
}

/// Parse Gemfile.lock
/// Format:
///   GEM
///     remote: https://rubygems.org/
///     specs:
///       rails (7.0.4)
///         actioncable (= 7.0.4)
///         actionpack (= 7.0.4)
///       rack (2.2.4)
///       puma (5.6.5)
///
///   PLATFORMS
///     ruby
///
///   DEPENDENCIES
///     rails (~> 7.0.0)
///     puma
fn parse_gemfile_lock(
    lockfile_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path).context("Failed to read Gemfile.lock")?;

    let mut in_specs_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for sections
        if trimmed == "specs:" {
            in_specs_section = true;
            continue;
        }

        // End specs section when we hit another top-level section
        if !line.starts_with(' ') && in_specs_section {
            in_specs_section = false;
            continue;
        }

        // Parse gem specifications
        if in_specs_section {
            // Calculate indentation (number of leading spaces)
            let indent = line.len() - line.trim_start().len();

            // Top-level gems in specs (typically 4 spaces)
            if indent == 4 && !trimmed.starts_with('-') {
                if let Some((name, version)) = parse_gem_spec_line(trimmed) {
                    result.add_package(Package {
                        name: name.to_string(),
                        version: version.to_string(),
                        ecosystem: "RubyGems".to_string(),
                        namespace: Some("rubygems.org".to_string()),
                        dependencies: Vec::new(),
                        license: None,
                        description: None,
                        homepage: None,
                        repository: None,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Parse a gem specification line
/// Examples:
///   rails (7.0.4)
///   rack (2.2.4)
///   puma (5.6.5)
fn parse_gem_spec_line(line: &str) -> Option<(&str, &str)> {
    // Format: "name (version)"
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim();
    let version_part = parts[1].trim();

    // Extract version from parentheses
    let version = version_part.strip_prefix('(')?.strip_suffix(')')?.trim();

    if !name.is_empty() && !version.is_empty() {
        Some((name, version))
    } else {
        None
    }
}

/// Parse Gemfile (basic fallback)
fn parse_gemfile(manifest_path: &std::path::Path, result: &mut EcosystemScanResult) -> Result<()> {
    let content = fs::read_to_string(manifest_path).context("Failed to read Gemfile")?;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse gem statements
        // Examples:
        //   gem 'rails', '~> 7.0'
        //   gem "puma", "~> 5.0"
        //   gem 'bootsnap', require: false
        if line.starts_with("gem ") {
            if let Some((name, version)) = parse_gemfile_line(line) {
                result.add_package(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    ecosystem: "RubyGems".to_string(),
                    namespace: Some("rubygems.org".to_string()),
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }
    }

    Ok(())
}

/// Parse a Gemfile gem line
/// Examples:
///   gem 'rails', '~> 7.0'
///   gem "puma", "~> 5.0"
///   gem 'bootsnap', require: false
fn parse_gemfile_line(line: &str) -> Option<(&str, &str)> {
    // Remove "gem " prefix
    let line = line.strip_prefix("gem ")?.trim();

    // Split by comma
    let parts: Vec<&str> = line.split(',').collect();
    if parts.is_empty() {
        return None;
    }

    // Extract name (remove quotes)
    let name = parts[0].trim().trim_matches('\'').trim_matches('"');

    // Extract version if present (second part)
    let version = if parts.len() > 1 {
        let version_part = parts[1].trim();
        // Check if it's a version string (not a hash key like "require:")
        if !version_part.contains(':') {
            version_part
                .trim_matches('\'')
                .trim_matches('"')
                .trim_start_matches('~')
                .trim_start_matches('>')
                .trim_start_matches('=')
                .trim()
        } else {
            "*" // No version specified
        }
    } else {
        "*" // No version specified
    };

    if !name.is_empty() {
        Some((name, version))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_gem_spec_line() {
        assert_eq!(
            parse_gem_spec_line("rails (7.0.4)"),
            Some(("rails", "7.0.4"))
        );
        assert_eq!(parse_gem_spec_line("rack (2.2.4)"), Some(("rack", "2.2.4")));
        assert_eq!(parse_gem_spec_line("puma (5.6.5)"), Some(("puma", "5.6.5")));
    }

    #[test]
    fn test_parse_gemfile_line() {
        assert_eq!(
            parse_gemfile_line("gem 'rails', '~> 7.0'"),
            Some(("rails", "7.0"))
        );
        assert_eq!(
            parse_gemfile_line("gem \"puma\", \"~> 5.0\""),
            Some(("puma", "5.0"))
        );
        assert_eq!(
            parse_gemfile_line("gem 'bootsnap', require: false"),
            Some(("bootsnap", "*"))
        );
    }

    #[tokio::test]
    async fn test_parse_gemfile_lock() {
        let temp = TempDir::new().unwrap();
        let gemfile_lock = temp.path().join("Gemfile.lock");

        fs::write(
            &gemfile_lock,
            r#"
GEM
  remote: https://rubygems.org/
  specs:
    rails (7.0.4)
      actioncable (= 7.0.4)
      actionpack (= 7.0.4)
    rack (2.2.4)
    puma (5.6.5)

PLATFORMS
  ruby

DEPENDENCIES
  rails (~> 7.0.0)
  puma
"#,
        )
        .unwrap();

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Ruby,
            temp.path().to_path_buf(),
            None,
            Some(gemfile_lock),
        );

        let result = scan(&ecosystem).await.unwrap();
        assert_eq!(result.total_packages, 3);

        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "rails" && p.version == "7.0.4"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "rack" && p.version == "2.2.4"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "puma" && p.version == "5.6.5"));
    }
}
