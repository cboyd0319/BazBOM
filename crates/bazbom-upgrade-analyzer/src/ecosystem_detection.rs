//! Ecosystem detection from package identifiers
//!
//! Auto-detects which package ecosystem a package belongs to based on its name format.
//! This allows users to run `bazbom fix express --explain` without specifying `--ecosystem npm`.

use bazbom_depsdev::System;

/// Detect ecosystem from package name format
///
/// This is the canonical ecosystem detection function for all of BazBOM.
/// It combines patterns from upgrade analysis and container scanning.
///
/// # Examples
///
/// ```
/// use bazbom_upgrade_analyzer::detect_ecosystem_from_package;
/// use bazbom_depsdev::System;
///
/// // Maven: group:artifact or Java package names
/// assert_eq!(detect_ecosystem_from_package("org.springframework:spring-core"), System::Maven);
/// assert_eq!(detect_ecosystem_from_package("org.apache.commons.lang3"), System::Maven);
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
/// assert_eq!(detect_ecosystem_from_package("django"), System::PyPI);
///
/// // Rust: crate name (hyphens or underscores)
/// assert_eq!(detect_ecosystem_from_package("serde"), System::Cargo);
/// assert_eq!(detect_ecosystem_from_package("tokio"), System::Cargo);
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

    // Maven: Java package names (org.*, com.*, io.*, net.*)
    if package.contains('.')
        && (package.starts_with("org.")
            || package.starts_with("com.")
            || package.starts_with("io.")
            || package.starts_with("net."))
    {
        return System::Maven;
    }

    // Go: import path (e.g., github.com/gin-gonic/gin, golang.org/x/crypto)
    if package.starts_with("github.com/")
        || package.starts_with("golang.org/")
        || package.starts_with("go.uber.org/")
        || package.starts_with("gopkg.in/")
        || package.starts_with("k8s.io/")
        || package.starts_with("sigs.k8s.io/")
        || package.starts_with("cloud.google.com/")
        || package.starts_with("google.golang.org/")
    {
        return System::Go;
    }

    // npm: scoped packages (e.g., @types/node, @angular/core)
    if package.starts_with('@') && package.contains('/') {
        return System::Npm;
    }

    // Ruby: gems with common suffixes or names
    if package.ends_with("-rails")
        || package.ends_with("-ruby")
        || package.ends_with("-rb")
        || matches!(
            package,
            "rails" | "rake" | "bundler" | "sinatra" | "puma" | "rspec" | "sidekiq"
            | "devise" | "pundit" | "redis" | "pg" | "mysql2" | "nokogiri" | "capybara"
            | "factory_bot" | "rubocop" | "activerecord" | "activesupport" | "actionpack"
        )
    {
        return System::RubyGems;
    }

    // PHP: composer packages (vendor/package format, e.g., symfony/symfony)
    // Uses bazbom-packagist crate for package intelligence instead of deps.dev
    if package.contains('/') && !package.starts_with('@') && !package.starts_with("github.com/") {
        let parts: Vec<&str> = package.split('/').collect();
        if parts.len() == 2
            && parts[0].chars().all(|c| c.is_lowercase() || c == '-')
            && parts[1].chars().all(|c| c.is_lowercase() || c == '-')
        {
            // Common PHP package vendors
            if matches!(
                parts[0],
                "symfony" | "laravel" | "phpunit" | "doctrine" | "guzzlehttp" | "monolog"
                | "psr" | "composer" | "league" | "illuminate" | "nesbot" | "ramsey"
                | "vlucas" | "fzaninotto" | "nikic" | "swiftmailer" | "twig" | "sensio"
                | "egulias" | "webmozart" | "friendsofphp" | "phpseclib" | "paragonie"
            ) {
                return System::Packagist;
            }
        }
    }

    // For simple lowercase names, use heuristics based on common packages
    if package
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c == '_' || c == '.' || c.is_numeric())
    {
        // Common npm packages (very popular ones)
        if package.starts_with("babel-")
            || package.starts_with("postcss-")
            || package.starts_with("rollup-")
            || package.starts_with("vite-")
            || package.starts_with("esbuild-")
            || package.starts_with("eslint-")
            || package.starts_with("webpack-")
            || matches!(
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
                    | "angular"
                    | "moment"
                    | "dayjs"
                    | "date-fns"
                    | "uuid"
                    | "chalk"
                    | "commander"
                    | "yargs"
                    | "inquirer"
                    | "ora"
                    | "debug"
                    | "dotenv"
                    | "cors"
                    | "body-parser"
                    | "mongoose"
                    | "sequelize"
                    | "prisma"
                    | "graphql"
                    | "apollo"
                    | "socket.io"
                    | "chart.js"
                    | "three"
                    | "d3"
            )
        {
            return System::Npm;
        }

        // Common Python packages (expanded list)
        if package.starts_with("python-")
            || package.starts_with("py")
            || package.ends_with("3")  // urllib3, etc.
            || matches!(
                package,
                "django"
                    | "flask"
                    | "numpy"
                    | "pandas"
                    | "requests"
                    | "pytest"
                    | "setuptools"
                    | "pip"
                    | "pillow"
                    | "cryptography"
                    | "scipy"
                    | "matplotlib"
                    | "tensorflow"
                    | "torch"
                    | "boto3"
                    | "botocore"
                    | "awscli"
                    | "celery"
                    | "redis"
                    | "sqlalchemy"
                    | "alembic"
                    | "keras"
                    | "transformers"
                    | "huggingface-hub"
                    | "scikit-image"
                    | "scikit-learn"
                    | "certifi"
                    | "charset-normalizer"
                    | "idna"
                    | "uvicorn"
                    | "fastapi"
                    | "starlette"
                    | "httpx"
                    | "aiohttp"
                    | "black"
                    | "mypy"
                    | "flake8"
                    | "isort"
                    | "poetry"
                    | "pipenv"
            )
        {
            return System::PyPI;
        }

        // Common Rust crates (expanded list)
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
                | "tracing"
                | "thiserror"
                | "futures"
                | "rand"
                | "regex"
                | "chrono"
                | "log"
                | "env_logger"
                | "lazy_static"
                | "once_cell"
                | "async-trait"
                | "pin-project"
                | "tower"
                | "warp"
                | "rocket"
                | "axum"
                | "diesel"
                | "sqlx"
                | "sea-orm"
                | "syn"
                | "quote"
                | "proc-macro2"
                | "bytes"
                | "parking_lot"
                | "crossbeam"
                | "rayon"
                | "itertools"
                | "num"
                | "bitflags"
        ) {
            return System::Cargo;
        }

        // Python packages with hyphens (strong signal, but not short ones)
        if package.contains('-') && package.len() > 5 && !package.contains('_') {
            return System::PyPI;
        }

        // Rust crates with underscores (strong signal)
        if package.contains('_') {
            return System::Cargo;
        }

        // Elixir/Hex packages (common ones)
        if matches!(
            package,
            "phoenix" | "ecto" | "plug" | "cowboy" | "jason" | "poison"
                | "httpoison" | "timex" | "ex_machina" | "bamboo" | "oban"
                | "absinthe" | "guardian" | "comeonin" | "bcrypt_elixir"
                | "ex_doc" | "credo" | "dialyxir" | "mix_test_watch"
        ) {
            return System::Hex;
        }

        // Dart/Pub packages (common ones)
        if matches!(
            package,
            "flutter" | "provider" | "bloc" | "riverpod" | "dio" | "http"
                | "path_provider" | "shared_preferences" | "sqflite" | "hive"
                | "get" | "mobx" | "freezed" | "json_serializable" | "equatable"
                | "dartz" | "rxdart" | "flutter_bloc" | "go_router"
        ) {
            return System::Pub;
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
    if package.contains('-')
        && package
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
    {
        return (System::PyPI, 0.6);
    }

    // Simple lowercase: low confidence, guess npm
    if package
        .chars()
        .all(|c| c.is_lowercase() || c == '-' || c == '_' || c.is_numeric())
    {
        // Common npm packages
        if matches!(
            package,
            "express" | "react" | "lodash" | "axios" | "vue" | "webpack"
        ) {
            return (System::Npm, 0.7);
        }

        // Common Rust crates
        if matches!(
            package,
            "serde" | "tokio" | "reqwest" | "anyhow" | "clap" | "actix"
        ) {
            return (System::Cargo, 0.7);
        }

        // Common Python packages
        if matches!(
            package,
            "django" | "flask" | "numpy" | "pandas" | "requests"
        ) {
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
        assert_eq!(detect_ecosystem_from_package("@angular/core"), System::Npm);

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
        assert_eq!(detect_ecosystem_from_package("go.uber.org/zap"), System::Go);
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
        assert_eq!(
            detect_ecosystem_from_package("package@1.0.0"),
            System::Maven
        );
    }
}
