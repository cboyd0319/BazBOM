//! BazBOM Verification Tool
//!
//! Verifies BazBOM installations for integrity and authenticity.
//!
//! # Checks Performed
//!
//! 1. **Checksum Verification**: SHA-256 hash matches release
//! 2. **Signature Verification**: Cosign signature valid (if available)
//! 3. **SLSA Provenance**: Provenance attestation valid (if available)
//! 4. **File Permissions**: Correct permissions (755 for binaries)
//! 5. **Known Compromises**: Not in list of compromised versions
//!
//! # Usage
//!
//! ```bash
//! # Verify local binary
//! bazbom-verify /usr/local/bin/bazbom
//!
//! # Verify with specific version
//! bazbom-verify /usr/local/bin/bazbom --version v7.0.0
//!
//! # Verbose output
//! bazbom-verify /usr/local/bin/bazbom --verbose
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::path::PathBuf;

mod checksum;
mod github;
mod permissions;

use checksum::verify_checksum;
use permissions::check_permissions;

#[derive(Parser, Debug)]
#[command(name = "bazbom-verify")]
#[command(about = "Verify BazBOM installation integrity and authenticity")]
#[command(version)]
struct Args {
    /// Path to bazbom binary to verify
    #[arg(value_name = "PATH")]
    binary_path: PathBuf,

    /// Expected version (e.g., v7.0.0). If not provided, will be detected.
    #[arg(short, long)]
    version: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip signature verification (only checksum)
    #[arg(long)]
    skip_signature: bool,

    /// Skip provenance verification
    #[arg(long)]
    skip_provenance: bool,
}

#[derive(Debug)]
struct VerificationResult {
    checksum_valid: bool,
    signature_valid: Option<bool>,
    provenance_valid: Option<bool>,
    permissions_correct: bool,
    not_compromised: bool,
}

impl VerificationResult {
    fn is_fully_valid(&self) -> bool {
        self.checksum_valid
            && self.signature_valid.unwrap_or(true)
            && self.provenance_valid.unwrap_or(true)
            && self.permissions_correct
            && self.not_compromised
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", "BazBOM Installation Verification".bold());
    println!("{}", "=================================".bold());
    println!();

    // Check if binary exists
    if !args.binary_path.exists() {
        eprintln!("{} Binary not found: {:?}", "✗".red(), args.binary_path);
        std::process::exit(1);
    }

    println!("{} Verifying: {:?}", "→".blue(), args.binary_path);
    println!();

    // Detect or use provided version
    let version = if let Some(v) = args.version {
        v
    } else {
        detect_version(&args.binary_path)?
    };

    println!("{} Version: {}", "→".blue(), version);
    println!();

    let mut result = VerificationResult {
        checksum_valid: false,
        signature_valid: None,
        provenance_valid: None,
        permissions_correct: false,
        not_compromised: true,
    };

    // 1. Verify checksum
    print!("{}  Checksum verification... ", "1.".bold());
    match verify_checksum(&args.binary_path, &version, args.verbose) {
        Ok(true) => {
            println!("{}", "PASS".green().bold());
            result.checksum_valid = true;
        }
        Ok(false) => {
            println!("{}", "FAIL".red().bold());
            eprintln!("   {} Checksum mismatch!", "WARN".yellow());
        }
        Err(e) => {
            println!("{}", "SKIP".yellow());
            if args.verbose {
                eprintln!("   Error: {}", e);
            }
        }
    }

    // 2. Verify file permissions
    print!("{}  File permissions... ", "2.".bold());
    match check_permissions(&args.binary_path) {
        Ok(true) => {
            println!("{}", "PASS".green().bold());
            result.permissions_correct = true;
        }
        Ok(false) => {
            println!("{}", "WARN".yellow().bold());
            eprintln!("   {} Permissions should be 755", "WARN".yellow());
            result.permissions_correct = false;
        }
        Err(e) => {
            println!("{}", "SKIP".yellow());
            if args.verbose {
                eprintln!("   Error: {}", e);
            }
        }
    }

    // 3. Check for known compromises
    print!("{}  Known compromises check... ", "3.".bold());
    if check_not_compromised(&version) {
        println!("{}", "PASS".green().bold());
        result.not_compromised = true;
    } else {
        println!("{}", "FAIL".red().bold());
        eprintln!(
            "   {} This version is known to be compromised!",
            "WARN".red()
        );
        result.not_compromised = false;
    }

    // 4. Signature verification (optional)
    if !args.skip_signature {
        print!("{}  Signature verification... ", "4.".bold());
        println!("{}", "SKIP".yellow());
        println!(
            "   {} Cosign verification not yet implemented",
            "INFO".blue()
        );
        // TODO: Implement cosign verification
        result.signature_valid = Some(false);
    }

    // 5. Provenance verification (optional)
    if !args.skip_provenance {
        print!("{}  SLSA provenance... ", "5.".bold());
        println!("{}", "SKIP".yellow());
        println!(
            "   {} Provenance verification not yet implemented",
            "INFO".blue()
        );
        // TODO: Implement SLSA provenance verification
        result.provenance_valid = Some(false);
    }

    // Final result
    println!();
    println!("{}", "═══════════════════════════════".bold());

    if result.is_fully_valid() {
        println!(
            "{} {}",
            "OK".green().bold(),
            "Installation verification PASSED".green().bold()
        );
        std::process::exit(0);
    } else if result.checksum_valid && result.permissions_correct && result.not_compromised {
        println!(
            "{} {}",
            "WARN".yellow().bold(),
            "Installation verification PASSED (with warnings)"
                .yellow()
                .bold()
        );
        std::process::exit(0);
    } else {
        println!(
            "{} {}",
            "FAIL".red().bold(),
            "Installation verification FAILED".red().bold()
        );
        std::process::exit(1);
    }
}

/// Detect version from binary
fn detect_version(binary_path: &PathBuf) -> Result<String> {
    // Try to run binary with --version
    let output = std::process::Command::new(binary_path)
        .arg("--version")
        .output()
        .context("Failed to execute binary to detect version")?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse version from output (format: "bazbom 7.0.0")
    if let Some(version_str) = stdout.split_whitespace().nth(1) {
        Ok(format!("v{}", version_str))
    } else {
        Ok("unknown".to_string())
    }
}

/// Check if version is in known compromised list
fn check_not_compromised(version: &str) -> bool {
    // List of known compromised versions (none currently)
    let compromised_versions: Vec<&str> = vec![];

    !compromised_versions.contains(&version)
}
