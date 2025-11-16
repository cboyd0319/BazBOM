# bazbom-tool-verify

**External tool integrity verification for BazBOM**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE)

## Overview

`bazbom-tool-verify` provides automated verification of external security tools (Syft, Semgrep, Trivy, etc.) before BazBOM executes them. This ensures supply chain integrity and protects against compromised or tampered tools.

This crate implements **M-06: External Tool Integrity Verification** from the BazBOM v7.0 Trust & Safety roadmap.

## Features

- ✅ **SHA-256 Checksum Verification** - Verify tool binaries against known-good checksums
- ✅ **Tool Registry** - Curated database of known-good tool versions
- ✅ **Compromised Version Detection** - Prevent execution of known-compromised tools
- ✅ **Platform-Specific Checksums** - Support for Linux, macOS, Windows (x86_64, ARM64)
- ✅ **Automatic Version Detection** - Extract version from tool `--version` output
- ✅ **Configurable Enforcement** - Strict or permissive verification modes
- 🚧 **GPG Signature Verification** - Planned
- 🚧 **Cosign Signature Verification** - Planned

## Security Model

### Fail-Secure Design

- **Enforcement by default**: Tools must verify successfully before execution
- **No execution on mismatch**: Checksum mismatches block execution
- **Compromised version protection**: Known-bad versions always blocked
- **Unregistered tool handling**: Configurable (allow or deny)

### Tool Registry

The tool registry (`data/tool-registry.json`) contains:

```json
{
  "schema_version": "1.0.0",
  "last_updated": "2025-11-16T00:00:00Z",
  "tools": {
    "syft": {
      "name": "syft",
      "description": "SBOM generation tool from Anchore",
      "recommended_version": "1.16.0",
      "versions": [
        {
          "version": "1.16.0",
          "sha256": "abc123...",
          "release_date": "2024-11-01",
          "platforms": {
            "linux-amd64": "def456...",
            "darwin-arm64": "ghi789..."
          },
          "compromised": false
        }
      ]
    }
  }
}
```

## Usage

### Basic Verification

```rust
use bazbom_tool_verify::{ToolVerifier, VerifyStatus};

fn main() -> anyhow::Result<()> {
    let verifier = ToolVerifier::new();

    // Verify a tool before using it
    match verifier.verify_tool("syft")? {
        VerifyStatus::Verified => {
            println!("✅ Syft verified - safe to execute");
            // Execute syft...
        }
        VerifyStatus::Failed(reason) => {
            eprintln!("❌ Verification failed: {}", reason);
        }
        VerifyStatus::Compromised => {
            eprintln!("🚨 Tool is known to be compromised!");
        }
        VerifyStatus::Unregistered => {
            println!("⚠️  Tool not in registry");
        }
        VerifyStatus::Skipped => {
            println!("⏭️  Verification skipped");
        }
    }

    Ok(())
}
```

### Custom Configuration

```rust
use bazbom_tool_verify::{ToolVerifier, VerifyConfig};

let config = VerifyConfig {
    enforce: true,                  // Fail on mismatch
    check_compromised: true,        // Check for known-bad versions
    allow_unregistered: false,      // Block unregistered tools
    custom_registry: None,          // Use default registry
};

let verifier = ToolVerifier::with_config(config);
```

### Permissive Mode (Development)

```rust
use bazbom_tool_verify::{ToolVerifier, VerifyConfig};

// For development/testing - warn but don't block
let config = VerifyConfig {
    enforce: false,                 // Warn instead of fail
    check_compromised: true,        // Still check for compromised
    allow_unregistered: true,       // Allow unregistered tools
    custom_registry: None,
};

let verifier = ToolVerifier::with_config(config);
```

### Verify Tool at Specific Path

```rust
use bazbom_tool_verify::ToolVerifier;
use std::path::Path;

let verifier = ToolVerifier::new();
let tool_path = Path::new("/usr/local/bin/syft");

let status = verifier.verify_tool_at_path("syft", tool_path)?;
```

## Supported Tools

The default registry includes:

| Tool | Latest Version | Description |
|------|----------------|-------------|
| **syft** | 1.16.0 | SBOM generation (Anchore) |
| **semgrep** | 1.95.0 | Static analysis |
| **trivy** | 0.57.1 | Comprehensive security scanner |
| **grype** | 0.84.0 | Vulnerability scanner (Anchore) |
| **cosign** | 2.4.1 | Container signing (Sigstore) |
| **trufflehog** | 3.82.13 | Secret detection |

## Registry Updates

### Manual Registry Update

```rust
use bazbom_tool_verify::ToolRegistry;

// Load custom registry
let registry_json = std::fs::read_to_string("custom-registry.json")?;
let registry = ToolRegistry::from_json(&registry_json)?;
```

### Automatic Updates (Future)

Future versions will support automatic registry updates from:
- GitHub releases
- HTTPS endpoints with signature verification
- OCI registries

## Error Handling

```rust
use bazbom_tool_verify::{ToolVerifier, ToolVerifyError};

match verifier.verify_tool("syft") {
    Ok(status) => {
        // Handle status...
    }
    Err(ToolVerifyError::ToolNotFound(tool)) => {
        eprintln!("Tool not found in PATH: {}", tool);
    }
    Err(ToolVerifyError::ChecksumMismatch { tool, expected, actual, .. }) => {
        eprintln!("Checksum mismatch for {}: expected {}, got {}", tool, expected, actual);
    }
    Err(e) => {
        eprintln!("Verification error: {}", e);
    }
}
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│  BazBOM CLI / Application                      │
└────────────────┬────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────┐
│  ToolVerifier                                   │
│  - Orchestrates verification workflow           │
│  - Configurable enforcement                     │
└────────────────┬────────────────────────────────┘
                 │
       ┌─────────┴─────────┐
       ↓                   ↓
┌──────────────┐    ┌──────────────────┐
│  Registry    │    │  Verification    │
│  - Tool DB   │    │  - SHA-256       │
│  - Versions  │    │  - GPG (planned) │
│  - Checksums │    │  - Cosign (plan) │
└──────────────┘    └──────────────────┘
```

## Testing

```bash
# Run tests
cargo test -p bazbom-tool-verify

# Run tests with output
cargo test -p bazbom-tool-verify -- --nocapture

# Run specific test
cargo test -p bazbom-tool-verify test_registry_has_common_tools
```

## Security Considerations

### Threat Model

**Threats Mitigated:**
- ✅ Supply chain compromise of external tools
- ✅ Tampered tool binaries
- ✅ Known-compromised tool versions
- ✅ TOCTOU (Time-Of-Check-Time-Of-Use) attacks via atomic verification

**Threats NOT Mitigated (yet):**
- ⚠️ Malicious registry updates (planned: signed registry)
- ⚠️ Local privilege escalation (OS responsibility)
- ⚠️ Memory corruption in verified tools (tool responsibility)

### Best Practices

1. **Always enable enforcement in production**: `enforce: true`
2. **Keep registry updated**: Monitor for new tool releases
3. **Check for compromised versions**: `check_compromised: true`
4. **Use platform-specific checksums**: More precise verification
5. **Monitor verification failures**: Log and alert on mismatches

## Roadmap

- [x] SHA-256 checksum verification
- [x] Tool registry with JSON schema
- [x] Platform-specific checksums
- [x] Compromised version detection
- [ ] GPG signature verification
- [ ] Cosign signature verification
- [ ] Automatic registry updates
- [ ] Rekor transparency log integration
- [ ] SigStore bundle support
- [ ] Custom tool registration API

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

To add a new tool to the registry:

1. Add tool entry to `data/tool-registry.json`
2. Include SHA-256 checksums for all platforms
3. Set `recommended_version` and `min_version`
4. Add tests in `tests/` directory
5. Submit pull request

## License

MIT License - see [LICENSE](../../LICENSE)

## References

- [BazBOM v7.0 Trust & Safety Roadmap](../../docs/roadmaps/V7_TRUST_AND_SAFETY_COMPREHENSIVE.md)
- [SECURITY.md](../../SECURITY.md)
- [SLSA Framework](https://slsa.dev/)
- [Sigstore](https://sigstore.dev/)
