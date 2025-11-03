# Phase 4: Manual Testing Guide

**Purpose:** Guide manual testing of Phase 4 IDE integration, automated remediation, and pre-commit hooks.

**Target Audience:** QA testers, contributors, early adopters

**Prerequisites:**
- BazBOM CLI installed and in PATH
- Advisory database synced (`bazbom db sync`)
- Java 11+ for reachability analysis (optional)

---

## Test Environment Setup

### 1. Sample Projects

Create or use existing sample projects for each build system:

**Maven Sample:**
```bash
git clone https://github.com/spring-projects/spring-petclinic.git
cd spring-petclinic
```

**Gradle Sample:**
```bash
git clone https://github.com/gradle/gradle-build-tool.git
cd gradle-build-tool
```

**Bazel Sample:**
```bash
git clone https://github.com/bazelbuild/examples.git
cd examples/java-tutorial
```

### 2. Install IDE Plugins

**IntelliJ IDEA:**
```bash
cd /path/to/BazBOM/crates/bazbom-intellij-plugin
./gradlew buildPlugin
# Install build/distributions/bazbom-intellij-plugin-1.0.0.zip via Settings → Plugins
```

**VS Code:**
```bash
cd /path/to/BazBOM/crates/bazbom-vscode-extension
npm install
npm run compile
npx vsce package
# Install bazbom-1.0.0.vsix via Extensions → Install from VSIX
```

**LSP Server:**
```bash
cd /path/to/BazBOM/crates/bazbom-lsp
cargo build --release
# Binary: ../../target/release/bazbom-lsp
# Configure path in VS Code settings
```

---

## Test Suite 1: IntelliJ IDEA Plugin

### Test 1.1: Installation and Configuration

**Steps:**
1. Install plugin from ZIP file
2. Restart IntelliJ IDEA
3. Open Settings → Tools → BazBOM
4. Verify all settings options are present
5. Set BazBOM CLI path if needed
6. Click "Test Connection"

**Expected Results:**
- ✅ Plugin installs without errors
- ✅ Settings panel shows all options
- ✅ Test Connection succeeds
- ✅ Shows BazBOM version

**Pass/Fail:** ___________

### Test 1.2: Tool Window - Maven Project

**Steps:**
1. Open Maven project (e.g., spring-petclinic)
2. View → Tool Windows → BazBOM
3. Click "Scan" button
4. Wait for scan to complete
5. Expand dependency tree

**Expected Results:**
- ✅ Tool window opens successfully
- ✅ Scan completes without errors
- ✅ Dependency tree displays with color coding
- ✅ Vulnerabilities shown with severity indicators
- ✅ Can expand/collapse nodes

**Pass/Fail:** ___________

### Test 1.3: Real-Time Warnings - pom.xml

**Steps:**
1. Open `pom.xml` in Maven project
2. Find a dependency with known vulnerability (or add one):
   ```xml
   <dependency>
     <groupId>org.apache.logging.log4j</groupId>
     <artifactId>log4j-core</artifactId>
     <version>2.14.1</version>
   </dependency>
   ```
3. Wait a few seconds for analysis

**Expected Results:**
- ✅ Warning appears on version line
- ✅ Hover shows CVE details
- ✅ Error-level for CRITICAL, warning for HIGH/MEDIUM
- ✅ Quick fix available (lightbulb icon)

**Pass/Fail:** ___________

### Test 1.4: Quick Fix - Maven

**Steps:**
1. Place cursor on vulnerable dependency
2. Press Alt+Enter (or ⌥⏎ on macOS)
3. Select "Upgrade to safe version X.Y.Z"
4. Wait for upgrade to complete

**Expected Results:**
- ✅ Quick fix menu appears
- ✅ Shows target version
- ✅ Updates version in pom.xml
- ✅ Maven reload triggered automatically
- ✅ Notification shows progress
- ✅ Final notification: success or failure

**Pass/Fail:** ___________

### Test 1.5: Gradle Project Support

**Steps:**
1. Open Gradle project
2. Open `build.gradle` or `build.gradle.kts`
3. Verify warnings appear on dependencies
4. Test quick fix on Gradle dependency

**Expected Results:**
- ✅ Gradle project detected
- ✅ Warnings appear in build.gradle
- ✅ Quick fix works for Gradle
- ✅ Gradle sync triggered after fix

**Pass/Fail:** ___________

### Test 1.6: Bazel Project Support

**Steps:**
1. Open Bazel project with maven_install
2. Open `MODULE.bazel` or `WORKSPACE`
3. Verify warnings appear
4. Test quick fix

**Expected Results:**
- ✅ Bazel project detected
- ✅ Warnings appear in Bazel files
- ✅ Quick fix updates version
- ✅ Notification reminds to run `bazel run @maven//:pin`

**Pass/Fail:** ___________

### Test 1.7: Auto-Scan on Project Open

**Steps:**
1. Enable "Auto-scan on project open" in settings
2. Close project
3. Reopen project
4. Check tool window

**Expected Results:**
- ✅ Scan starts automatically on project open
- ✅ Results populate in tool window
- ✅ No errors or crashes

**Pass/Fail:** ___________

### Test 1.8: Settings Persistence

**Steps:**
1. Change settings (disable auto-scan, change severity)
2. Close and reopen IntelliJ IDEA
3. Check settings panel

**Expected Results:**
- ✅ Settings persist across IDE restarts
- ✅ Saved in .idea/bazbom.xml

**Pass/Fail:** ___________

---

## Test Suite 2: VS Code Extension

### Test 2.1: Installation and Configuration

**Steps:**
1. Install VSIX file
2. Reload VS Code
3. Open Settings (search "BazBOM")
4. Set LSP server path
5. Open Output panel → BazBOM

**Expected Results:**
- ✅ Extension installs successfully
- ✅ Settings available
- ✅ LSP server starts (check Output panel)
- ✅ No errors in Output

**Pass/Fail:** ___________

### Test 2.2: Diagnostics - Maven Project

**Steps:**
1. Open Maven project in VS Code
2. Open `pom.xml`
3. Wait for diagnostics to appear
4. Open Problems panel (Ctrl+Shift+M)

**Expected Results:**
- ✅ Diagnostics appear in Problems panel
- ✅ Inline squiggles show in pom.xml
- ✅ Hover shows CVE details
- ✅ Correct severity (error/warning/info)

**Pass/Fail:** ___________

### Test 2.3: Quick Fix - VS Code

**Steps:**
1. Click on squiggle or place cursor on warning
2. Click lightbulb icon or press Ctrl+.
3. Select "Upgrade to safe version X.Y.Z"

**Expected Results:**
- ✅ Quick fix menu appears
- ✅ Shows correct target version
- ✅ Version updated in file

**Pass/Fail:** ___________

### Test 2.4: Commands

**Steps:**
1. Open Command Palette (Ctrl+Shift+P)
2. Search "BazBOM"
3. Test "BazBOM: Scan Project"
4. Test "BazBOM: Sync Advisory Database"

**Expected Results:**
- ✅ Commands appear in palette
- ✅ Scan command triggers scan
- ✅ Sync command updates database
- ✅ Output panel shows progress

**Pass/Fail:** ___________

### Test 2.5: File Watching

**Steps:**
1. Have pom.xml open with diagnostics
2. Modify a dependency version
3. Save file
4. Wait for diagnostics to update

**Expected Results:**
- ✅ Diagnostics update on file save
- ✅ New warnings appear if applicable
- ✅ Old warnings disappear if fixed

**Pass/Fail:** ___________

---

## Test Suite 3: Automated Remediation

### Test 3.1: Fix Suggest Mode

**Steps:**
```bash
cd /path/to/vulnerable/project
bazbom scan .
bazbom fix --suggest
```

**Expected Results:**
- ✅ Shows list of fixable vulnerabilities
- ✅ Educational "why fix this?" explanation
- ✅ CVSS score, KEV status, EPSS probability shown
- ✅ Build-system-specific instructions
- ✅ JSON report created

**Pass/Fail:** ___________

### Test 3.2: Fix Apply Mode - Maven

**Steps:**
```bash
cd /path/to/maven/project
bazbom fix --apply
```

**Expected Results:**
- ✅ Updates versions in pom.xml
- ✅ Runs tests automatically
- ✅ Shows progress indicators
- ✅ Reports success/failure for each fix
- ✅ Preserves formatting

**Pass/Fail:** ___________

### Test 3.3: Rollback on Test Failure

**Steps:**
1. Use project with tests
2. Manually break a test (edit test file)
3. Run `bazbom fix --apply`

**Expected Results:**
- ✅ Fix applied initially
- ✅ Tests run and fail
- ✅ Changes automatically rolled back
- ✅ Error message explains rollback
- ✅ pom.xml reverted to original state

**Pass/Fail:** ___________

### Test 3.4: PR Generation

**Prerequisites:** GitHub token and repo set

**Steps:**
```bash
export GITHUB_TOKEN="ghp_..."
export GITHUB_REPOSITORY="owner/repo"
bazbom fix --pr
```

**Expected Results:**
- ✅ Creates new branch with timestamp
- ✅ Applies fixes
- ✅ Runs tests
- ✅ Commits changes
- ✅ Pushes to remote
- ✅ Creates PR via GitHub API
- ✅ PR has detailed description with vulnerability table
- ✅ Returns PR URL

**Pass/Fail:** ___________

### Test 3.5: Gradle Support

**Steps:**
```bash
cd /path/to/gradle/project
bazbom fix --apply
```

**Expected Results:**
- ✅ Updates versions in build.gradle
- ✅ Runs `./gradlew test` (or `gradle test`)
- ✅ Works for both .gradle and .gradle.kts
- ✅ Handles both implementation() and compile()

**Pass/Fail:** ___________

### Test 3.6: Bazel Support

**Steps:**
```bash
cd /path/to/bazel/project
bazbom fix --apply
```

**Expected Results:**
- ✅ Updates MODULE.bazel or WORKSPACE
- ✅ Updates maven_install.json
- ✅ Reminds to run `bazel run @maven//:pin`
- ✅ Runs `bazel test //...`

**Pass/Fail:** ___________

---

## Test Suite 4: Pre-Commit Hooks

### Test 4.1: Hook Installation

**Steps:**
```bash
cd /path/to/project
bazbom install-hooks
ls -la .git/hooks/pre-commit
```

**Expected Results:**
- ✅ Hook file created at .git/hooks/pre-commit
- ✅ File is executable (Unix)
- ✅ Success message displayed

**Pass/Fail:** ___________

### Test 4.2: Hook Execution - No Violations

**Steps:**
```bash
# In project with no vulnerabilities
git add .
git commit -m "Test commit"
```

**Expected Results:**
- ✅ Hook runs automatically
- ✅ Shows "Scanning dependencies..." message
- ✅ Shows "No policy violations" message
- ✅ Commit succeeds

**Pass/Fail:** ___________

### Test 4.3: Hook Execution - With Violations

**Steps:**
1. Add vulnerable dependency
2. Try to commit:
   ```bash
   git add .
   git commit -m "Add vulnerable dependency"
   ```

**Expected Results:**
- ✅ Hook runs and detects violations
- ✅ Shows clear error message
- ✅ Commit is blocked
- ✅ Suggests remediation commands

**Pass/Fail:** ___________

### Test 4.4: Bypass Mechanism

**Steps:**
```bash
# With policy violations present
git commit --no-verify -m "Bypass test"
```

**Expected Results:**
- ✅ Hook is bypassed
- ✅ Commit succeeds
- ✅ Warning shown (if any)

**Pass/Fail:** ___________

### Test 4.5: Fast Mode

**Steps:**
```bash
bazbom install-hooks --fast
git add .
time git commit -m "Test fast mode"
```

**Expected Results:**
- ✅ Hook completes in <10 seconds
- ✅ Scan is successful
- ✅ Policy check works

**Pass/Fail:** ___________

### Test 4.6: Custom Policy File

**Steps:**
```bash
# Create custom policy
cat > custom-policy.yml << EOF
severity_threshold: HIGH
block_on_critical: true
EOF

bazbom install-hooks --policy=custom-policy.yml
git commit -m "Test custom policy"
```

**Expected Results:**
- ✅ Hook uses custom policy file
- ✅ Policy rules are enforced
- ✅ Works correctly

**Pass/Fail:** ___________

---

## Test Suite 5: Cross-Platform Testing

### Test 5.1: Linux Testing

**Steps:**
1. Test all suites on Linux (Ubuntu, Debian, Fedora)
2. Verify paths and permissions
3. Test CLI and IDE plugins

**Expected Results:**
- ✅ All features work on Linux
- ✅ No path-related issues
- ✅ Permissions set correctly

**Pass/Fail:** ___________

### Test 5.2: macOS Testing

**Steps:**
1. Test all suites on macOS (Intel and Apple Silicon)
2. Verify native builds work
3. Test IDE plugins

**Expected Results:**
- ✅ All features work on macOS
- ✅ Both architectures supported
- ✅ No platform-specific issues

**Pass/Fail:** ___________

### Test 5.3: Windows Testing

**Steps:**
1. Test CLI on Windows with Git Bash
2. Test VS Code extension
3. Verify pre-commit hooks work

**Expected Results:**
- ✅ CLI works in Git Bash
- ✅ VS Code extension works
- ✅ Hooks work with Git for Windows

**Pass/Fail:** ___________

---

## Test Suite 6: Performance Testing

### Test 6.1: Large Project Scan

**Steps:**
1. Find project with 500+ dependencies
2. Time full scan:
   ```bash
   time bazbom scan .
   ```

**Expected Results:**
- ✅ Completes in reasonable time (< 2 minutes)
- ✅ No memory issues
- ✅ Accurate results

**Pass/Fail:** ___________

### Test 6.2: IDE Performance

**Steps:**
1. Open large project in IntelliJ
2. Monitor IDE responsiveness
3. Test quick fixes on multiple dependencies

**Expected Results:**
- ✅ IDE remains responsive
- ✅ No freezing or stuttering
- ✅ Quick fixes are quick (<5 seconds)

**Pass/Fail:** ___________

### Test 6.3: Pre-Commit Hook Speed

**Steps:**
1. Test hook in various project sizes
2. Measure time for each:
   ```bash
   time git commit -m "Performance test"
   ```

**Expected Results:**
- ✅ Small projects (<50 deps): <5 seconds
- ✅ Medium projects (50-200 deps): <10 seconds
- ✅ Large projects (200+ deps): <15 seconds (fast mode)

**Pass/Fail:** ___________

---

## Test Suite 7: Error Handling

### Test 7.1: Invalid CLI Path

**Steps:**
1. Set invalid BazBOM CLI path in IDE settings
2. Try to trigger scan

**Expected Results:**
- ✅ Clear error message
- ✅ Suggests checking path
- ✅ No crash

**Pass/Fail:** ___________

### Test 7.2: Network Issues (Offline Mode)

**Steps:**
1. Disconnect from internet
2. Try to scan project
3. Try to sync database

**Expected Results:**
- ✅ Scan works with cached database
- ✅ Sync fails gracefully with clear message
- ✅ IDE plugin continues to function

**Pass/Fail:** ___________

### Test 7.3: Malformed Build Files

**Steps:**
1. Create invalid pom.xml
2. Try to scan

**Expected Results:**
- ✅ Clear error message
- ✅ Points to problematic file
- ✅ No crash

**Pass/Fail:** ___________

---

## Test Suite 8: Integration Testing

### Test 8.1: CI/CD Integration

**Steps:**
1. Set up GitHub Actions workflow
2. Run scan in CI
3. Check for SARIF upload

**Expected Results:**
- ✅ Scan runs successfully in CI
- ✅ SARIF file generated
- ✅ GitHub Code Scanning populated

**Pass/Fail:** ___________

### Test 8.2: Multi-Module Maven Project

**Steps:**
1. Open multi-module Maven project
2. Scan entire project
3. Check tool window

**Expected Results:**
- ✅ All modules detected
- ✅ Dependencies from all modules shown
- ✅ Quick fixes work across modules

**Pass/Fail:** ___________

### Test 8.3: Monorepo with Multiple Build Systems

**Steps:**
1. Create repo with Maven, Gradle, and Bazel projects
2. Scan each project
3. Test IDE features

**Expected Results:**
- ✅ Each build system detected correctly
- ✅ Features work for each type
- ✅ No conflicts between systems

**Pass/Fail:** ___________

---

## Bug Reporting Template

If you find issues during testing, please report them using this template:

```markdown
### Bug Report

**Component:** [IntelliJ Plugin / VS Code Extension / LSP Server / CLI]

**Severity:** [Critical / High / Medium / Low]

**Environment:**
- OS: [Linux / macOS / Windows]
- BazBOM Version: [output of `bazbom --version`]
- IDE Version: [IntelliJ 2024.x / VS Code 1.x]
- Java Version: [output of `java -version`]

**Steps to Reproduce:**
1. 
2. 
3. 

**Expected Behavior:**


**Actual Behavior:**


**Screenshots/Logs:**
[Attach screenshots or log files]

**Additional Context:**
[Any other relevant information]
```

---

## Test Results Summary

After completing all tests, fill out this summary:

**Test Date:** ___________  
**Tester Name:** ___________  
**Environment:** ___________

### Overall Results

| Test Suite | Pass | Fail | Skip | Notes |
|------------|------|------|------|-------|
| 1. IntelliJ Plugin | __ / 8 | __ | __ | |
| 2. VS Code Extension | __ / 5 | __ | __ | |
| 3. Automated Remediation | __ / 6 | __ | __ | |
| 4. Pre-Commit Hooks | __ / 6 | __ | __ | |
| 5. Cross-Platform | __ / 3 | __ | __ | |
| 6. Performance | __ / 3 | __ | __ | |
| 7. Error Handling | __ / 3 | __ | __ | |
| 8. Integration | __ / 3 | __ | __ | |

**Total Pass Rate:** _____%

### Critical Issues Found
1. 
2. 
3. 

### Recommendations
1. 
2. 
3. 

### Sign-Off

**Tester:** ___________  
**Date:** ___________  
**Status:** [✅ Approved for Release / ⚠️ Approved with Notes / ❌ Needs Work]

---

## Additional Resources

- [Phase 4 Specification](PHASE_4_DEVELOPER_EXPERIENCE.md)
- [Phase 4 Completion Summary](PHASE_4_COMPLETION.md)
- [Usage Guide](../USAGE.md)
- [IDE Setup Guide](../guides/IDE_SETUP.md)
- [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
