//! Actionable error messages with quick fixes and documentation links
//!
//! Makes BazBOM errors helpful instead of cryptic

use crate::output;

/// Common BazBOM error scenarios with solutions
pub enum BazBomError {
    /// No build system detected
    NoBuildSystem { path: String },
    /// Cargo.lock not found or invalid
    InvalidCargoLock { path: String },
    /// Maven pom.xml not found or invalid
    InvalidPom { path: String },
    /// Gradle build files not found
    InvalidGradle { path: String },
    /// Network/API failure
    NetworkError { endpoint: String, error: String },
    /// GitHub CLI not installed
    GhCliMissing,
    /// Git not found in PATH
    GitMissing,
    /// Insufficient permissions
    PermissionDenied { file: String },
    /// OPAL reachability analysis failed
    ReachabilityFailed { error: String },
    /// Baseline file not found
    BaselineNotFound { path: String },
    /// Invalid SARIF file
    InvalidSarif { path: String, error: String },
    /// No vulnerabilities database available
    NoDatabaseConnection,
    /// Profile not found in bazbom.toml
    ProfileNotFound { profile: String, config_path: String },
}

impl BazBomError {
    /// Display the error with actionable messages
    pub fn display(&self) {
        match self {
            BazBomError::NoBuildSystem { path } => {
                output::print_error(
                    "No Build System Detected",
                    &format!("Could not detect a supported build system in: {}\n\nSupported build systems:\n  • Cargo (Cargo.toml + Cargo.lock)\n  • Maven (pom.xml)\n  • Gradle (build.gradle / build.gradle.kts)\n  • Bazel (WORKSPACE / MODULE.bazel)", path),
                    Some("# Initialize a Cargo project:\ncargo init\n\n# Or scan a different directory:\nbazbom scan /path/to/project"),
                    Some("https://docs.bazbom.dev/build-systems"),
                );
            },

            BazBomError::InvalidCargoLock { path } => {
                output::print_error(
                    "Invalid Cargo.lock",
                    &format!("Cargo.lock not found or corrupted: {}", path),
                    Some("# Regenerate Cargo.lock:\ncargo update\n\n# Or if build fails:\ncargo clean && cargo build"),
                    Some("https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html"),
                );
            },

            BazBomError::InvalidPom { path } => {
                output::print_error(
                    "Invalid pom.xml",
                    &format!("Maven pom.xml not found or invalid: {}", path),
                    Some("# Validate your pom.xml:\nmvn validate\n\n# Install dependencies:\nmvn install"),
                    Some("https://maven.apache.org/guides/introduction/introduction-to-the-pom.html"),
                );
            },

            BazBomError::InvalidGradle { path } => {
                output::print_error(
                    "Invalid Gradle Build",
                    &format!("Gradle build files not found or invalid: {}", path),
                    Some("# Initialize Gradle:\ngradle init\n\n# Or regenerate build files:\ngradle wrapper --gradle-version latest"),
                    Some("https://docs.gradle.org/current/userguide/tutorial_using_tasks.html"),
                );
            },

            BazBomError::NetworkError { endpoint, error } => {
                output::print_error(
                    "Network Connection Failed",
                    &format!("Failed to connect to: {}\n\nError: {}", endpoint, error),
                    Some("# Check your internet connection\n# Or use --no-upload to skip API calls:\nbazbom scan --no-upload\n\n# Set a proxy if needed:\nexport HTTP_PROXY=http://proxy.example.com:8080"),
                    Some("https://docs.bazbom.dev/troubleshooting/network"),
                );
            },

            BazBomError::GhCliMissing => {
                output::print_error(
                    "GitHub CLI Not Found",
                    "The 'gh' command is required for GitHub integration but was not found in PATH.",
                    Some("# Install GitHub CLI:\n# macOS:\nbrew install gh\n\n# Linux:\nsudo apt install gh  # or yum, dnf, etc.\n\n# Windows:\nwinget install GitHub.cli\n\n# Then authenticate:\ngh auth login"),
                    Some("https://cli.github.com/manual/installation"),
                );
            },

            BazBomError::GitMissing => {
                output::print_error(
                    "Git Not Found",
                    "Git is required but was not found in PATH.",
                    Some("# Install Git:\n# macOS:\nbrew install git\n\n# Linux:\nsudo apt install git\n\n# Windows:\nwinget install Git.Git"),
                    Some("https://git-scm.com/book/en/v2/Getting-Started-Installing-Git"),
                );
            },

            BazBomError::PermissionDenied { file } => {
                output::print_error(
                    "Permission Denied",
                    &format!("Insufficient permissions to access: {}", file),
                    Some("# Fix permissions:\nsudo chmod +r <file>\n\n# Or run with sudo (NOT RECOMMENDED):\nsudo bazbom scan"),
                    None,
                );
            },

            BazBomError::ReachabilityFailed { error } => {
                output::print_error(
                    "Reachability Analysis Failed",
                    &format!("OPAL reachability analysis encountered an error:\n\n{}", error),
                    Some("# Try without reachability:\nbazbom scan --fast\n\n# Or check if Java bytecode is valid:\njavap -c <class-file>\n\n# Report this issue if it persists:\nbazbom report --include-logs"),
                    Some("https://docs.bazbom.dev/features/reachability"),
                );
            },

            BazBomError::BaselineNotFound { path } => {
                output::print_error(
                    "Baseline File Not Found",
                    &format!("Could not find baseline file: {}", path),
                    Some("# Generate a baseline first:\nbazbom scan --json > baseline.json\n\n# Or use diff mode without baseline:\nbazbom pr"),
                    Some("https://docs.bazbom.dev/features/diff-mode"),
                );
            },

            BazBomError::InvalidSarif { path, error } => {
                output::print_error(
                    "Invalid SARIF File",
                    &format!("Failed to parse SARIF file: {}\n\nError: {}", path, error),
                    Some("# Regenerate SARIF:\nbazbom scan --format sarif > findings.sarif\n\n# Validate SARIF schema:\njq . findings.sarif"),
                    Some("https://sarifweb.azurewebsites.net/"),
                );
            },

            BazBomError::NoDatabaseConnection => {
                output::print_error(
                    "Vulnerability Database Unavailable",
                    "Could not connect to vulnerability database.\n\nYou're likely offline or the service is temporarily unavailable.",
                    Some("# Use local cache:\nbazbom scan --offline\n\n# Or update database:\nbazbom db update"),
                    Some("https://docs.bazbom.dev/troubleshooting/database"),
                );
            },

            BazBomError::ProfileNotFound { profile, config_path } => {
                output::print_error(
                    "Profile Not Found",
                    &format!("Profile '{}' not found in: {}", profile, config_path),
                    Some("# List available profiles:\ncat bazbom.toml\n\n# Or initialize config with example profiles:\nbazbom init"),
                    Some("https://docs.bazbom.dev/configuration/profiles"),
                );
            },
        }
    }

    /// Get a short error description (for logs)
    pub fn short_description(&self) -> String {
        match self {
            BazBomError::NoBuildSystem { .. } => "No build system detected".to_string(),
            BazBomError::InvalidCargoLock { .. } => "Invalid Cargo.lock".to_string(),
            BazBomError::InvalidPom { .. } => "Invalid pom.xml".to_string(),
            BazBomError::InvalidGradle { .. } => "Invalid Gradle build".to_string(),
            BazBomError::NetworkError { .. } => "Network connection failed".to_string(),
            BazBomError::GhCliMissing => "GitHub CLI not found".to_string(),
            BazBomError::GitMissing => "Git not found".to_string(),
            BazBomError::PermissionDenied { .. } => "Permission denied".to_string(),
            BazBomError::ReachabilityFailed { .. } => "Reachability analysis failed".to_string(),
            BazBomError::BaselineNotFound { .. } => "Baseline file not found".to_string(),
            BazBomError::InvalidSarif { .. } => "Invalid SARIF file".to_string(),
            BazBomError::NoDatabaseConnection => "Database unavailable".to_string(),
            BazBomError::ProfileNotFound { .. } => "Profile not found".to_string(),
        }
    }
}

/// Helper to display common errors in a user-friendly way
pub fn display_io_error(operation: &str, path: &str, error: &std::io::Error) {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            output::print_error(
                "File Not Found",
                &format!("Could not {} because file does not exist:\n  {}", operation, path),
                Some(&format!("# Check if the file exists:\nls -la {}\n\n# Or scan a different path:\nbazbom scan /correct/path", path)),
                None,
            );
        },
        std::io::ErrorKind::PermissionDenied => {
            BazBomError::PermissionDenied { file: path.to_string() }.display();
        },
        _ => {
            output::print_error(
                &format!("I/O Error: {}", operation),
                &format!("Failed to {}: {}\n\nFile: {}", operation, error, path),
                Some("# Check file permissions and disk space:\nls -la .\ndf -h"),
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        // Just make sure error display doesn't panic
        BazBomError::NoBuildSystem { path: ".".to_string() }.display();
        BazBomError::GhCliMissing.display();
        BazBomError::NetworkError {
            endpoint: "https://api.github.com".to_string(),
            error: "Connection timeout".to_string(),
        }.display();
    }

    #[test]
    fn test_short_descriptions() {
        assert_eq!(
            BazBomError::GhCliMissing.short_description(),
            "GitHub CLI not found"
        );
        assert_eq!(
            BazBomError::NoDatabaseConnection.short_description(),
            "Database unavailable"
        );
    }
}
