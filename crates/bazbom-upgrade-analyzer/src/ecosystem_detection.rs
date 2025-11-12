//! Ecosystem detection from package identifiers
//!
//! Auto-detects which package ecosystem a package belongs to based on its name format.
//! This allows users to run `bazbom fix express --explain` without specifying `--ecosystem npm`.

use bazbom_depsdev::System;

/// Detect ecosystem from package name format
///
/// # Examples
///
/// ```
/// use bazbom_upgrade_analyzer::detect_ecosystem_from_package;
/// use bazbom_depsdev::System;
///
/// // Maven: group:artifact
/// assert_eq!(detect_ecosystem_from_package("org.springframework:spring-core"), System::Maven);
///
/// // npm: scoped or simple name
/// assert_eq!(detect_ecosystem_from_package("@types/node"), System::Npm);
/// assert_eq!(detect_ecosystem_from_package("express"), System::Npm);
///
/// // Go: import path
/// assert_eq!(detect_ecosystem_from_package("github.com/gin-gonic/gin"), System::Go);
///
/// // Python: package name (hyphens common)
/// assert_eq!(detect_ecosystem_from_package("scikit-learn"), System::PyPI);
///
/// // Rust: crate name (hyphens or underscores)
/// assert_eq!(detect_ecosystem_from_package("serde"), System::Cargo);
/// ```
pub fn detect_ecosystem_from_package(package: &str) -> System {
    // Empty or whitespace - default to Maven
    if package.trim().is_empty() {
        return System::Maven;
    }

    // Maven: group:artifact format (e.g., org.springframework:spring-core)
    if package.contains(':') && !package.starts_with("github.com/") {
        return System::Maven;
    }

    // Go: import path (e.g., github.com/gin-gonic/gin, golang.org/x/crypto)
    if package.starts_with("github.com/")
        || package.starts_with("golang.org/")
        || package.starts_with("go.uber.org/")
        || package.starts_with("gopkg.in/")
    {
        return System::Go;
    }

    // npm: scoped packages (e.g., @types/node, @angular/core)
    if package.starts_with('@') && package.contains('/') {
        return System::Npm;
    }

    // Ruby: gems often have hyphens and are lowercase
    // But this is ambiguous with Python, so we need more context
    // For now, let's use a heuristic: if it ends with common Ruby suffixes
    if package.ends_with("-rails") || package.ends_with("-ruby") || package == "rails" {
        return System::RubyGems;
    }

    // PHP: composer packages (vendor/package format, e.g., symfony/symfony)
    if package.contains('/') && !package.starts_with('@') && !package.starts_with("github.com/") {
        // Check if it looks like a composer package (lowercase, hyphens allowed)
        let parts: Vec<&str> = package.split('/').collect();
        if parts.len() == 2
            && parts[0].chars().all(|c| c.is_lowercase() || c == '-')
            && parts[1].chars().all(|c| c.is_lowercase() || c == '-')
        {
            // Could be PHP or others, but PHP is more common with vendor/package format
            // Check for common PHP package patterns
            if parts[0] == "symfony"
                || parts[0] == "laravel"
                || parts[0] == "phpunit"
                || parts[0] == "doctrine"
            {
                return System::NuGet; // Placeholder - need to add PHP support to System enum
            }
        }
    }

    // For simple lowercase names, use heuristics based on common packages
    if package.chars().all(|c| c.is_lowercase() || c == '-' || c == '_' || c == '.' || c.is_numeric()) {
        // Common npm packages (very popular ones)
        if matches!(
            package,
            "express"
                | "react"
                | "lodash"
                | "axios"
                | "vue"
                | "webpack"
                | "next"
                | "typescript"
                | "eslint"
                | "prettier"
                | "jest"
                | "mocha"
        ) {
            return System::Npm;
        }

        // Common Python packages
        if matches!(
            package,
            "django"
                | "flask"
                | "numpy"
                | "pandas"
                | "requests"
                | "pytest"
                | "setuptools"
                | "pip"
        ) {
            return System::PyPI;
        }

        // Common Rust crates
        if matches!(
            package,
            "serde"
                | "tokio"
                | "reqwest"
                | "anyhow"
                | "clap"
                | "actix"
                | "hyper"
                | "async_std"
        ) {
            return System::Cargo;
        }

        // Python packages with hyphens (strong signal)
        if package.contains('-') && package.len() > 5 {
            return System::PyPI;
        }

        // Rust crates with underscores (strong signal)
        if package.contains('_') {
            return System::Cargo;
        }

        // Default: npm is most popular for simple names
        return System::Npm;
    }

    // Default fallback: Maven (original behavior)
    System::Maven
}

/// Detect ecosystem with confidence score
///
/// Returns (System, confidence) where confidence is 0.0-1.0
/// - 1.0 = 100% certain (e.g., Maven group:artifact format)
/// - 0.5 = ambiguous (e.g., simple name could be npm or Rust)
/// - 0.0 = no idea, guessing
pub fn detect_ecosystem_with_confidence(package: &str) -> (System, f64) {
    // Maven: very high confidence
    if package.contains(':') && !package.starts_with("github.com/") {
        return (System::Maven, 1.0);
    }

    // Go: high confidence
    if package.starts_with("github.com/")
        || package.starts_with("golang.org/")
        || package.starts_with("go.uber.org/")
        || package.starts_with("gopkg.in/")
    {
        return (System::Go, 0.95);
    }

    // npm scoped: high confidence
    if package.starts_with('@') && package.contains('/') {
        return (System::Npm, 0.9);
    }

    // PHP composer: medium-high confidence
    if package.contains('/') && !package.starts_with('@') && !package.starts_with("github.com/") {
        let parts: Vec<&str> = package.split('/').collect();
        if parts.len() == 2 {
            if parts[0] == "symfony" || parts[0] == "laravel" {
                return (System::NuGet, 0.8); // Placeholder
            }
            return (System::NuGet, 0.6); // Could be PHP
        }
    }

    // Python with hyphens: medium confidence
    if package.contains('-') && package.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric()) {
        return (System::PyPI, 0.6);
    }

    // Simple lowercase: low confidence, guess npm
    if package.chars().all(|c| c.is_lowercase() || c == '-' || c == '_' || c.is_numeric()) {
        // Common npm packages
        if matches!(package, "express" | "react" | "lodash" | "axios" | "vue" | "webpack") {
            return (System::Npm, 0.7);
        }

        // Common Rust crates
        if matches!(package, "serde" | "tokio" | "reqwest" | "anyhow" | "clap" | "actix") {
            return (System::Cargo, 0.7);
        }

        // Common Python packages
        if matches!(package, "django" | "flask" | "numpy" | "pandas" | "requests") {
            return (System::PyPI, 0.7);
        }

        // Default to npm for ambiguous simple names
        return (System::Npm, 0.5);
    }

    // No idea, default to Maven
    (System::Maven, 0.3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maven_detection() {
        assert_eq!(
            detect_ecosystem_from_package("org.springframework:spring-core"),
            System::Maven
        );
        assert_eq!(
            detect_ecosystem_from_package("com.google.guava:guava"),
            System::Maven
        );
        assert_eq!(
            detect_ecosystem_from_package("org.apache.logging.log4j:log4j-core"),
            System::Maven
        );
    }

    #[test]
    fn test_npm_detection() {
        // Scoped packages
        assert_eq!(detect_ecosystem_from_package("@types/node"), System::Npm);
        assert_eq!(
            detect_ecosystem_from_package("@angular/core"),
            System::Npm
        );

        // Simple names (common npm packages)
        assert_eq!(detect_ecosystem_from_package("express"), System::Npm);
        assert_eq!(detect_ecosystem_from_package("react"), System::Npm);
        assert_eq!(detect_ecosystem_from_package("lodash"), System::Npm);
    }

    #[test]
    fn test_go_detection() {
        assert_eq!(
            detect_ecosystem_from_package("github.com/gin-gonic/gin"),
            System::Go
        );
        assert_eq!(
            detect_ecosystem_from_package("golang.org/x/crypto"),
            System::Go
        );
        assert_eq!(
            detect_ecosystem_from_package("go.uber.org/zap"),
            System::Go
        );
    }

    #[test]
    fn test_python_detection() {
        assert_eq!(detect_ecosystem_from_package("scikit-learn"), System::PyPI);
        assert_eq!(detect_ecosystem_from_package("django"), System::PyPI);
        assert_eq!(detect_ecosystem_from_package("flask"), System::PyPI);
    }

    #[test]
    fn test_rust_detection() {
        assert_eq!(detect_ecosystem_from_package("serde"), System::Cargo);
        assert_eq!(detect_ecosystem_from_package("tokio"), System::Cargo);
        assert_eq!(detect_ecosystem_from_package("reqwest"), System::Cargo);
    }

    #[test]
    fn test_confidence_scores() {
        // Maven should be 100% confident
        let (system, confidence) =
            detect_ecosystem_with_confidence("org.springframework:spring-core");
        assert_eq!(system, System::Maven);
        assert_eq!(confidence, 1.0);

        // npm scoped should be high confidence
        let (system, confidence) = detect_ecosystem_with_confidence("@types/node");
        assert_eq!(system, System::Npm);
        assert!(confidence >= 0.9);

        // Ambiguous simple name should be low-medium confidence
        let (_system, confidence) = detect_ecosystem_with_confidence("somepackage");
        assert!(confidence <= 0.7);
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!(detect_ecosystem_from_package(""), System::Maven);

        // Whitespace
        assert_eq!(detect_ecosystem_from_package("  "), System::Maven);

        // Special characters
        assert_eq!(detect_ecosystem_from_package("package@1.0.0"), System::Maven);
    }
}
