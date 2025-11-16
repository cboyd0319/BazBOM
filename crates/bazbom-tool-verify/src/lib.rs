//! External tool integrity verification for BazBOM
//!
//! This crate provides verification of external security tools (Syft, Semgrep, etc.)
//! before BazBOM executes them, ensuring supply chain integrity.
//!
//! # Security Model
//!
//! - All external tools are verified before execution
//! - Multiple verification methods: SHA-256 checksums, GPG signatures, Cosign
//! - Tool registry maintained with known-good versions
//! - Automatic updates from trusted sources
//! - Fail-secure: execution blocked on verification failure
//!
//! # Example
//!
//! ```no_run
//! use bazbom_tool_verify::ToolVerifier;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Create a verifier
//! let verifier = ToolVerifier::new();
//!
//! // Verify a tool before using it
//! match verifier.verify_tool("syft")? {
//!     bazbom_tool_verify::VerifyStatus::Verified => {
//!         println!("Tool verified - safe to use");
//!     }
//!     bazbom_tool_verify::VerifyStatus::Failed(reason) => {
//!         eprintln!("Tool verification failed: {}", reason);
//!     }
//!     bazbom_tool_verify::VerifyStatus::Compromised => {
//!         eprintln!("Tool is known to be compromised!");
//!     }
//!     _ => {}
//! }
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod registry;
pub mod verifier;

pub use error::{ToolVerifyError, ToolVerifyResult};
pub use registry::{Tool, ToolRegistry, ToolVersion};
pub use verifier::{ToolVerifier, VerifyConfig, VerifyStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads() {
        let registry = ToolRegistry::default();
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_registry_has_common_tools() {
        let registry = ToolRegistry::default();

        // Check for commonly used tools
        assert!(registry.get_tool("syft").is_some());
        assert!(registry.get_tool("semgrep").is_some());
        assert!(registry.get_tool("trivy").is_some());
        assert!(registry.get_tool("grype").is_some());
        assert!(registry.get_tool("cosign").is_some());
        assert!(registry.get_tool("trufflehog").is_some());
    }

    #[test]
    fn test_tool_has_recommended_version() {
        let registry = ToolRegistry::default();

        if let Some(syft) = registry.get_tool("syft") {
            assert!(syft.get_recommended_version().is_some());
        }
    }
}
