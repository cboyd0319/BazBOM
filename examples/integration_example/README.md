# BazBOM Integration Example

This example demonstrates the BazBOM integration plan in action.

## What This Example Shows

1. **Directory Structure**: Output follows the integration plan exactly
   - `sbom/` - SPDX 2.3 + optional CycloneDX 1.5
   - `findings/` - Individual SARIF files + merged.sarif
   - `enrich/` - deps.dev enrichment data
   - `fixes/` - OpenRewrite recipes

2. **SARIF 2.1.0 Compliance**: All findings merge into a single valid SARIF
   - One run per tool
   - Deduplicated results
   - GitHub Code Scanning compatible

3. **Configuration-Driven**: Uses `bazbom.toml` for defaults

## Quick Start

```bash
# From this directory
cd examples/integration_example

# Run minimal scan (SCA only)
../../target/release/bazbom scan --out-dir ./output --no-upload

# Run full scan (all features)
../../target/release/bazbom scan \
  --cyclonedx \
  --with-semgrep \
  --out-dir ./output \
  --no-upload

# View results
tree output
cat output/findings/merged.sarif | jq '.version, .runs[] | .tool.driver.name'
```

## Output Structure

After running a scan, you'll see:

```
output/
├── sbom/
│   ├── spdx.json              # SPDX 2.3 SBOM (always generated)
│   └── cyclonedx.json         # CycloneDX 1.5 (when --cyclonedx used)
├── findings/
│   ├── sca.sarif              # SCA findings from OSV/NVD/GHSA
│   ├── semgrep.sarif          # Semgrep findings (when --with-semgrep used)
│   ├── codeql.sarif           # CodeQL findings (when --with-codeql used)
│   └── merged.sarif           # Single merged SARIF (all tools)
├── enrich/
│   └── depsdev.json           # deps.dev enrichment (when enabled in config)
└── fixes/
    └── openrewrite/           # Autofix recipes (when --autofix used)
```

## Configuration File

The `bazbom.toml` file controls default behavior:

```toml
[analysis]
cyclonedx = false              # Enable with --cyclonedx flag
semgrep = { enabled = false }  # Enable with --with-semgrep flag

[enrich]
depsdev = false                # Set to true to enable enrichment

[autofix]
mode = "off"                   # Options: off, dry-run, pr

[publish]
github_code_scanning = false   # Upload disabled in examples
artifact = true                # Archive outputs
```

## Integration with GitHub Actions

See `.github/workflows/bazbom-orchestrated-scan.yml` for a complete workflow example.

Key steps:
1. Install Semgrep (optional): `pipx install semgrep`
2. Install CodeQL (optional): Download and extract CLI
3. Run BazBOM scan with desired flags
4. Upload SARIF: `github/codeql-action/upload-sarif@v3`
5. Archive artifacts: `actions/upload-artifact@v4`

## Testing the Integration

Run the integration tests to validate:

```bash
# From repository root
cargo test --test integration_plan_validation
cargo test --test orchestration_test

# All tests should pass
```

## Further Reading

- [Integration Plan](../../docs/copilot/BAZBOM_INTEGRATION_PLAN.md) - Complete specification
- [Orchestrated Scan Guide](../../docs/ORCHESTRATED_SCAN.md) - Usage documentation
- [Usage Guide](../../docs/USAGE.md) - Command reference
