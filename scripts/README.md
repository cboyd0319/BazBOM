# BazBOM Scripts

Helper scripts for installation, testing, and system verification.

## Available Scripts

### `check-system.sh`

**Purpose:** Comprehensive system dependency checker that validates your environment is ready for BazBOM.

**Usage:**

```bash
# Run remotely (recommended for first-time users)
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/scripts/check-system.sh | sh

# Or run locally
./scripts/check-system.sh
```

**What it checks:**

1. **Operating System**
   - Detects macOS/Linux version
   - Verifies architecture (x86_64/ARM64)
   - Confirms compatibility

2. **BazBOM Installation**
   - Checks if BazBOM is installed
   - Verifies it's in PATH
   - Tests functionality

3. **Core Dependencies**
   - Git, curl, tar

4. **JVM Dependencies** (for Java/Kotlin/Scala projects)
   - Java (version 11+)
   - JAVA_HOME environment variable
   - Maven, Gradle, Bazel (optional)

5. **Polyglot Language Support**
   - Node.js/npm (JavaScript/TypeScript)
   - Python (Python projects)
   - Go (Go projects)
   - Rust (Rust projects)
   - Ruby (Ruby projects)
   - PHP (PHP projects)

6. **PATH Configuration**
   - Verifies common binary paths

7. **System Resources**
   - Available disk space

8. **Test Scan**
   - Runs a minimal test scan to verify BazBOM works

**Output:**

The script provides a detailed report with:
- ✓ Passed checks (green)
- ⚠ Warnings (yellow) - optional improvements
- ✗ Failed checks (red) - requires attention
- Summary with pass/warn/fail counts

**Example Output:**

```
╔════════════════════════════════════════════════╗
║      BazBOM System Dependency Checker          ║
║  Validates your system is ready for BazBOM     ║
╚════════════════════════════════════════════════╝

══════ Operating System ══════
✓ macOS detected: 26.1
✓ macOS version is compatible
✓ Architecture: ARM64 (fully supported)

══════ BazBOM Installation ══════
✓ BazBOM is installed: bazbom 6.5.0
✓ Location: /usr/local/bin/bazbom
✓ BazBOM is functional

══════ Core Dependencies ══════
✓ Git is installed: v2.42.0
✓ curl is installed
✓ tar is installed

══════ JVM Dependencies (for Java/Kotlin/Scala projects) ══════
✓ Java is installed: 21.0.1
✓ Java version is compatible (11+)
✓ JAVA_HOME is set: /usr/local/opt/openjdk@21
✓ Maven is installed: v3.9.5

Summary:
  ✓ Passed: 15
  ⚠ Warnings: 2
  ✗ Failed: 0

✓ Your system is fully ready for BazBOM!
```

**Use Cases:**

1. **Pre-Installation:** Check if your system is ready before installing BazBOM
2. **Troubleshooting:** Diagnose issues with BazBOM installation
3. **Team Onboarding:** Verify new developer workstations are properly configured
4. **CI/CD Setup:** Validate CI environments have required dependencies

**Environment Variables:**

None required. The script auto-detects everything.

---

## Adding New Scripts

When adding new scripts to this directory:

1. **Make them executable:** `chmod +x scripts/your-script.sh`
2. **Add shebang:** Start with `#!/usr/bin/env bash`
3. **Document here:** Add a section above describing purpose and usage
4. **Test on macOS and Linux:** Ensure cross-platform compatibility
5. **Follow style:** Use the same color coding and output format as `check-system.sh`

---

## Script Development Guidelines

### Exit Codes

- `0` - Success
- `1` - General failure
- `2` - Missing dependencies
- `3` - Configuration error

### Output Formatting

Use the helper functions from `check-system.sh`:

```bash
info()    # Blue ℹ - Informational message
success() # Green ✓ - Check passed
warn()    # Yellow ⚠ - Warning (non-critical)
fail()    # Red ✗ - Check failed (critical)
section() # Cyan header - Section divider
```

### Error Handling

Always use `set -e` at the top of scripts to exit on errors (unless specific error handling is needed).

---

For questions or issues with these scripts, please [open an issue](https://github.com/cboyd0319/BazBOM/issues).
