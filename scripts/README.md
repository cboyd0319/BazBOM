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

### `monorepo-diagnostics.sh`

**Purpose:** Comprehensive diagnostic tool for analyzing large monorepos to help tune BazBOM for optimal performance with complex projects.

**Usage:**

```bash
# Navigate to your monorepo
cd /path/to/your/monorepo

# Run the diagnostic script
/path/to/BazBOM/scripts/monorepo-diagnostics.sh
```

**What it analyzes:**

1. **Repository Size & Structure** - File counts, languages detected, lines of code
2. **Directory Tree** - Project organization (3 levels deep)
3. **Build Systems Detection** - Maven, Gradle, Bazel (Bzlmod + legacy), sbt, npm, Go, Rust, Ruby, PHP
4. **Sample Build Files** - Examines root-level build configurations (pom.xml, build.gradle, MODULE.bazel, etc.)
5. **Bazel Detailed Analysis** - Target counts, module dependencies, Maven integration, rules_jvm_external
6. **Module/Project Structure** - Subprojects, modules, package organization
7. **Dependency Patterns** - How dependencies are declared and managed
8. **Git Repository Stats** - History, contributors, tracked files
9. **Special Configurations** - CI/CD (GitHub Actions, GitLab CI, Jenkins), Docker, Kubernetes
10. **Performance Indicators** - Metrics relevant to BazBOM optimization

**Output Files:**

Creates `./bazbom-diagnostics/` directory with:

- `00-SUMMARY.txt` - Quick overview and recommendations ⭐ **Start here**
- `01-repo-structure.txt` - File counts, sizes, languages
- `02-directory-tree.txt` - Directory hierarchy
- `03-build-systems.txt` - All build systems detected
- `04-build-file-samples.txt` - Sample build configurations
- `05-bazel-analysis.txt` - Bazel targets and dependencies (if Bazel detected)
- `06-module-structure.txt` - Project/module organization
- `07-dependency-patterns.txt` - Dependency management patterns
- `08-git-stats.txt` - Repository history and statistics
- `09-special-configs.txt` - CI/CD, Docker, Kubernetes configurations
- `10-performance-indicators.txt` - Performance metrics

Also creates `bazbom-diagnostics.tar.gz` archive for easy sharing.

**Features:**

- ✅ **zsh compatible** - No shell globbing issues
- ✅ **Bazel 6+ Bzlmod support** - Detects MODULE.bazel and Bzlmod modules
- ✅ **Timeout protection** - Won't hang on huge repos (configurable timeouts)
- ✅ **Colored output** - Clear progress indicators
- ✅ **No dependencies** - Uses only standard Unix tools (optional: bazel, tree, cloc for enhanced output)
- ✅ **Safe** - Read-only analysis, no modifications
- ✅ **Large repo optimized** - Limits output size, samples first N files

**Requirements:**

**Required:**
- bash
- git
- Standard Unix tools (find, grep, wc, tar, du)

**Optional (for enhanced output):**
- `bazel` - Enables Bazel target analysis and queries
- `tree` - Better directory tree visualization
- `cloc` - Lines of code analysis by language

**Example Output:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  1/10 Repository Size & Structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[✓] Completed: Repository structure analysis

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  5/10 Bazel Detailed Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[INFO] Counting all Bazel targets (may take a while for large repos)...
[INFO] Counting Java targets...
[✓] Completed: Bazel analysis

...

╔════════════════════════════════════════════════════════════════╗
║         BazBOM Monorepo Diagnostics - Summary Report          ║
╚════════════════════════════════════════════════════════════════╝

Repository size: 2.5G
Total files: 45,234
Tracked files: 38,901

BUILD SYSTEMS DETECTED:
✓ Bazel: 1,234 BUILD files (Bzlmod/MODULE.bazel)
✓ Maven: 15 pom.xml files
✓ npm/yarn: 8 package.json files

KEY RECOMMENDATIONS FOR BAZBOM:
⚠ VERY LARGE MONOREPO (1234 build files)
  → Recommend using --limit parameter for testing
  → Enable caching for repeated scans
  → Consider incremental scanning
```

**Use Cases:**

1. **BazBOM Tuning:** Understand your monorepo structure to optimize BazBOM configuration
2. **Performance Troubleshooting:** Identify why scans are slow
3. **Pre-Scan Planning:** Estimate scan time and resource requirements
4. **Bug Reports:** Provide context when reporting BazBOM issues
5. **Monorepo Documentation:** Generate comprehensive overview of repository structure

**Bazel-Specific Analysis:**

For Bazel monorepos, the script provides:

- Bzlmod (MODULE.bazel) vs legacy WORKSPACE detection
- Target type counts (java_library, kt_jvm_library, java_test, proto_library, etc.)
- Maven dependency management analysis (maven.install, rules_jvm_external)
- maven_install.json lockfile locations
- Module dependency graph (for Bzlmod)
- .bazelrc configuration sample

**Timeouts:**

To prevent hanging on massive repositories, the script uses:

- **Short operations** (30s): File listings, bazel info
- **Medium operations** (60s): Target type queries, module graph
- **Long operations** (120s): Full target listing (`bazel query "//..."`)

If a query times out, the script logs it and continues with remaining diagnostics.

**Sharing Diagnostics:**

Before sharing the output:

1. Review `00-SUMMARY.txt` for quick overview
2. Check files for sensitive data (internal package names, proprietary info)
3. Sanitize if needed:
   ```bash
   # Example: Replace internal package names
   sed -i 's/com\.yourcompany/com.example/g' bazbom-diagnostics/*.txt
   ```
4. Share the `bazbom-diagnostics.tar.gz` file or entire directory

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
