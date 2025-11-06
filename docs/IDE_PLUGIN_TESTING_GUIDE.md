# IDE Plugin Testing Guide

**Last Updated:** 2025-11-05  
**Status:** Active  
**Target:** Developers and QA testers

---

## Overview

This guide provides comprehensive testing procedures for BazBOM IDE plugins (VS Code and IntelliJ IDEA) to ensure quality and reliability before marketplace publishing.

---

## Table of Contents

1. [Testing Prerequisites](#testing-prerequisites)
2. [VS Code Extension Testing](#vs-code-extension-testing)
3. [IntelliJ IDEA Plugin Testing](#intellij-idea-plugin-testing)
4. [Performance Testing](#performance-testing)
5. [Compatibility Testing](#compatibility-testing)
6. [User Acceptance Testing](#user-acceptance-testing)
7. [Bug Reporting](#bug-reporting)

---

## Testing Prerequisites

### Environment Setup

**Required Tools:**
- Git (for cloning test projects)
- BazBOM CLI (latest version)
- VS Code (latest stable)
- IntelliJ IDEA 2023.3+ (Community or Ultimate)
- Java 17+ (for test projects)
- Maven 3.8+ (for Maven projects)
- Gradle 7.0+ (for Gradle projects)
- Bazel 6.0+ (for Bazel projects)

**Initial Setup:**

```bash
# 1. Install BazBOM CLI
brew install cboyd0319/tap/bazbom

# 2. Sync advisory database
bazbom db sync

# 3. Verify installation
bazbom --version
bazbom-lsp --version
```

### Test Projects

Create a dedicated testing directory with sample projects:

```bash
mkdir ~/bazbom-plugin-testing
cd ~/bazbom-plugin-testing
```

**Maven Test Project:**
```bash
git clone https://github.com/spring-projects/spring-petclinic.git
cd spring-petclinic
git checkout v3.1.0  # Known vulnerable version
cd ..
```

**Gradle Test Project:**
```bash
git clone https://github.com/gradle/gradle-build-scan-quickstart.git
cd gradle-build-scan-quickstart
# Modify build.gradle to use older dependencies
cd ..
```

**Bazel Test Project:**
```bash
git clone https://github.com/bazelbuild/examples.git bazel-examples
cd bazel-examples/java-tutorial
cd ../..
```

---

## VS Code Extension Testing

### Installation Testing

**Test 1: Manual Installation from VSIX**

1. Build extension:
   ```bash
   cd crates/bazbom-vscode-extension
   npm install
   npm run compile
   npx vsce package
   ```

2. Install in VS Code:
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "Extensions: Install from VSIX"
   - Select the generated `.vsix` file

3. Verify:
   - Extension appears in Extensions sidebar
   - BazBOM commands are available in command palette
   - No error messages in Output panel (View → Output → BazBOM Language Server)

**Expected Result:**  Extension installs without errors

---

### Functionality Testing

**Test 2: LSP Server Connection**

1. Open a Maven project (e.g., spring-petclinic)
2. Open `pom.xml`
3. Check Output panel (View → Output → BazBOM Language Server)
4. Look for "LSP server started" message

**Expected Result:**  LSP server connects and starts successfully

---

**Test 3: Real-time Scanning on File Save**

1. Open `pom.xml` in a test project
2. Add a known vulnerable dependency:
   ```xml
   <dependency>
     <groupId>org.apache.logging.log4j</groupId>
     <artifactId>log4j-core</artifactId>
     <version>2.14.1</version>
   </dependency>
   ```
3. Save the file (`Ctrl+S` or `Cmd+S`)
4. Wait 5-10 seconds
5. Check Problems panel (View → Problems)

**Expected Result:**  Vulnerability appears as a problem with CVE-2021-44228 (Log4Shell)

**Screenshot Location:** `docs/images/vscode-vulnerability-detection.png` (placeholder)

---

**Test 4: Diagnostic Severity Levels**

1. Add dependencies with different severity levels to `pom.xml`:
   ```xml
   <!-- Critical -->
   <dependency>
     <groupId>org.apache.logging.log4j</groupId>
     <artifactId>log4j-core</artifactId>
     <version>2.14.1</version>
   </dependency>
   
   <!-- High -->
   <dependency>
     <groupId>org.springframework</groupId>
     <artifactId>spring-web</artifactId>
     <version>5.3.20</version>
   </dependency>
   
   <!-- Medium -->
   <dependency>
     <groupId>com.fasterxml.jackson.core</groupId>
     <artifactId>jackson-databind</artifactId>
     <version>2.13.0</version>
   </dependency>
   ```

2. Save and wait for scan
3. Check Problems panel

**Expected Result:**
-  Critical/High: Red error markers
-  Medium: Yellow warning markers
-  Low: Blue info markers

---

**Test 5: Manual Scan Command**

1. Open command palette (`Ctrl+Shift+P` or `Cmd+Shift+P`)
2. Type "BazBOM: Scan Project"
3. Execute command
4. Check for progress indicator
5. Verify results appear in Problems panel

**Expected Result:**  Manual scan completes and displays results

---

**Test 6: Sync Advisory Database Command**

1. Open command palette
2. Type "BazBOM: Sync Advisory Database"
3. Execute command
4. Check Output panel for sync progress

**Expected Result:**  Database syncs successfully with progress messages

---

**Test 7: Configuration Settings**

1. Go to Settings (File → Preferences → Settings)
2. Search for "BazBOM"
3. Verify all settings are present:
   - `bazbom.lspPath`
   - `bazbom.enableRealTimeScanning`
   - `bazbom.scanOnOpen`
   - `bazbom.severityThreshold`
4. Change `severityThreshold` to "high"
5. Re-scan project
6. Verify only high/critical vulnerabilities appear

**Expected Result:**  All settings work correctly

---

**Test 8: File Type Detection**

Test with different build files:

| File | Should Activate | Test Result |
|------|----------------|-------------|
| `pom.xml` |  Yes | |
| `build.gradle` |  Yes | |
| `build.gradle.kts` |  Yes | |
| `BUILD` |  Yes | |
| `BUILD.bazel` |  Yes | |
| `WORKSPACE` |  Yes | |
| `MODULE.bazel` |  Yes | |
| `package.json` |  No | |
| `pom.xml.bak` |  No | |

**Expected Result:**  Extension activates only for supported build files

---

**Test 9: Error Handling**

1. Remove BazBOM CLI from PATH:
   ```bash
   export PATH="/usr/bin:/bin:/usr/sbin:/sbin"  # Minimal PATH
   ```
2. Open VS Code and try to scan
3. Check for user-friendly error message

**Expected Result:**  Clear error message explaining BazBOM CLI not found

---

### Performance Testing

**Test 10: Scan Performance**

Test scan times with different project sizes:

| Project Size | Expected Scan Time | Actual Time |
|--------------|-------------------|-------------|
| Small (<50 deps) | <5 seconds | |
| Medium (50-200 deps) | <10 seconds | |
| Large (200-500 deps) | <15 seconds | |
| Huge (500+ deps) | <30 seconds | |

**Procedure:**
1. Open project
2. Open build file
3. Note time before scan
4. Save file to trigger scan
5. Note time when diagnostics appear

**Expected Result:**  All scans complete within expected times

---

**Test 11: Memory Usage**

1. Open VS Code
2. Open Activity Monitor (macOS) or Task Manager (Windows)
3. Note baseline VS Code memory usage
4. Install and activate BazBOM extension
5. Scan multiple projects
6. Note peak memory usage

**Expected Result:**  Extension adds <100MB to VS Code memory footprint

---

### Compatibility Testing

**Test 12: VS Code Version Compatibility**

Test with different VS Code versions:

| Version | Status | Test Result |
|---------|--------|-------------|
| Latest Stable (1.84+) |  Primary Target | |
| Previous Stable (1.83) |  Should Work | |
| Insiders |  Nice to Have | |

---

## IntelliJ IDEA Plugin Testing

### Installation Testing

**Test 13: Manual Installation from ZIP**

1. Build plugin:
   ```bash
   cd crates/bazbom-intellij-plugin
   ./gradlew buildPlugin
   ```

2. Install in IntelliJ:
   - Settings → Plugins →  → Install Plugin from Disk
   - Select `build/distributions/bazbom-intellij-plugin-1.0.0.zip`
   - Restart IntelliJ

3. Verify:
   - Plugin appears in Installed Plugins
   - BazBOM tool window appears on right sidebar
   - BazBOM menu items appear in Tools menu

**Expected Result:**  Plugin installs and activates without errors

---

### Functionality Testing

**Test 14: Tool Window Dependency Tree**

1. Open Maven project (spring-petclinic)
2. Click BazBOM tool window on right sidebar
3. Click "Scan" button
4. Wait for scan to complete
5. Verify dependency tree displays:
   - Root dependencies
   - Transitive dependencies
   - Scope labels (compile, test, runtime)
   - Vulnerability indicators (color-coded)

**Expected Result:**  Dependency tree displays correctly with all information

**Screenshot Location:** `docs/images/intellij-dependency-tree.png` (placeholder)

---

**Test 15: Real-time Maven Annotation**

1. Open `pom.xml`
2. Add vulnerable dependency:
   ```xml
   <dependency>
     <groupId>org.apache.logging.log4j</groupId>
     <artifactId>log4j-core</artifactId>
     <version>2.14.1</version>
   </dependency>
   ```
3. Wait 2-3 seconds
4. Look for inline warning marker (yellow/red lightbulb)
5. Hover over warning

**Expected Result:** 
-  Warning appears inline next to dependency
-  Hover shows CVE details, severity, and fixed version
-  Warning color matches severity (red for critical, yellow for high)

---

**Test 16: Real-time Gradle Annotation**

1. Open `build.gradle` or `build.gradle.kts`
2. Add vulnerable dependency:
   ```kotlin
   implementation("org.apache.logging.log4j:log4j-core:2.14.1")
   ```
3. Wait 2-3 seconds
4. Verify inline warning appears

**Expected Result:**  Same behavior as Maven annotation

---

**Test 17: Real-time Bazel Annotation**

1. Open `BUILD.bazel`
2. Add vulnerable dependency in maven_install:
   ```python
   maven_install(
       artifacts = [
           "org.apache.logging.log4j:log4j-core:2.14.1",
       ],
   )
   ```
3. Wait 2-3 seconds
4. Verify inline warning appears

**Expected Result:**  Same behavior as Maven/Gradle annotation

---

**Test 18: Quick Fix Action (Alt+Enter)**

1. Position cursor on vulnerable dependency in `pom.xml`
2. Press `Alt+Enter` (Windows/Linux) or `⌥+Return` (macOS)
3. Verify "Upgrade to safe version" action appears
4. Select action
5. Wait for:
   - Dependency version update
   - Maven/Gradle sync
   - Test execution
   - Notification

**Expected Result:**
-  Dependency version upgrades correctly
-  Tests run in background
-  Success notification appears
-  If tests fail, rollback occurs with warning notification

**Screenshot Location:** `docs/images/intellij-quick-fix.png` (placeholder)

---

**Test 19: Settings Panel**

1. Go to Settings → Tools → BazBOM
2. Verify all settings are present:
   - Enable real-time scanning
   - Auto-scan on project open
   - Severity threshold selector
   - BazBOM CLI path
   - Policy file path
3. Change settings and click Apply
4. Restart IntelliJ
5. Verify settings persist

**Expected Result:**  All settings work and persist correctly

---

**Test 20: Auto-Scan on Project Open**

1. Enable "Auto-scan on project open" in settings
2. Close IntelliJ
3. Open IntelliJ with a project
4. Check for automatic scan initiation

**Expected Result:**  Scan starts automatically without user interaction

---

**Test 21: Manual Scan Action**

1. Go to Tools → Scan with BazBOM
2. Verify scan starts
3. Check tool window updates with results
4. Verify notification appears on completion

**Expected Result:**  Manual scan works and updates UI

---

**Test 22: Database Sync Action**

1. Go to Tools → Sync BazBOM Database
2. Verify background task starts
3. Check for completion notification
4. Verify no UI blocking

**Expected Result:**  Database sync completes in background

---

### Performance Testing

**Test 23: Scan Performance**

| Project Size | Expected Scan Time | Actual Time |
|--------------|-------------------|-------------|
| Small (<50 deps) | <10 seconds | |
| Medium (50-200 deps) | <30 seconds | |
| Large (200-500 deps) | <60 seconds | |
| Huge (500+ deps) | <120 seconds | |

**Note:** First scan is slower due to dependency graph building.

**Expected Result:**  Subsequent scans are significantly faster (cached)

---

**Test 24: UI Responsiveness**

1. Start scan on large project
2. Try to:
   - Type in editor
   - Navigate files
   - Open dialogs
   - Resize windows

**Expected Result:**  All UI operations remain responsive during scan

---

**Test 25: Memory Usage**

1. Open IntelliJ
2. Monitor memory usage (Help → Diagnostic Tools → Memory Indicator)
3. Install plugin
4. Scan multiple large projects
5. Note peak memory usage

**Expected Result:**  Plugin adds <200MB to IntelliJ memory footprint

---

### Compatibility Testing

**Test 26: IntelliJ Version Compatibility**

Test with different IntelliJ versions:

| Version | Edition | Status | Test Result |
|---------|---------|--------|-------------|
| 2024.1+ | Community |  Primary | |
| 2024.1+ | Ultimate |  Primary | |
| 2023.3 | Community |  Minimum | |
| 2023.3 | Ultimate |  Minimum | |
| 2023.2 | Any |  Not Supported | |

---

**Test 27: Operating System Compatibility**

| OS | Version | Test Result |
|----|---------|-------------|
| macOS | 13.0+ (Ventura+) | |
| macOS | 12.0 (Monterey) | |
| Windows | 11 | |
| Windows | 10 | |
| Linux | Ubuntu 22.04 | |
| Linux | Fedora 38 | |

---

**Test 28: Android Studio Compatibility**

1. Install plugin in Android Studio (based on IntelliJ)
2. Open Android project
3. Test all features

**Expected Result:**  Plugin works in Android Studio (may have limitations for Android-specific Gradle)

---

## User Acceptance Testing

### Usability Testing

**Test 29: First-Time User Experience**

Scenario: New user installing plugin for the first time

1. Install plugin
2. Open project
3. Observe if:
   - Clear onboarding or welcome message
   - Helpful error messages if BazBOM CLI not found
   - Settings are discoverable
   - Documentation is accessible

**Expected Result:**  New user can get started without external help

---

**Test 30: Developer Workflow Integration**

Scenario: Developer using plugin during daily work

1. Open project
2. Make code changes
3. Add new dependencies
4. Run tests
5. Commit changes

**Expected Result:**
-  Plugin does not interfere with normal workflow
-  Warnings appear at appropriate times
-  Quick fixes are convenient (Alt+Enter)
-  No annoying popups or interruptions

---

### Accessibility Testing

**Test 31: Screen Reader Compatibility**

1. Enable screen reader (VoiceOver on macOS, Narrator on Windows)
2. Navigate plugin UI
3. Verify:
   - All buttons have labels
   - Diagnostics are readable
   - Tool window is navigable

**Expected Result:**  Plugin is screen-reader accessible

---

**Test 32: Keyboard Navigation**

1. Use only keyboard (no mouse)
2. Navigate plugin features:
   - Open tool window
   - Execute scan
   - Apply quick fix
   - Change settings

**Expected Result:**  All features accessible via keyboard

---

**Test 33: Color Contrast**

1. Test with different IDE themes:
   - Light (IntelliJ Light, VS Code Light)
   - Dark (Darcula, VS Code Dark)
   - High Contrast (VS Code High Contrast)

**Expected Result:**  Severity indicators visible in all themes

---

## Bug Reporting

### Bug Report Template

When filing bugs, include:

**Environment:**
- OS: macOS 13.5 / Windows 11 / Ubuntu 22.04
- IDE: VS Code 1.84 / IntelliJ IDEA 2024.1 Community
- Plugin Version: 1.0.0
- BazBOM CLI Version: 0.5.1

**Steps to Reproduce:**
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Expected Behavior:**
[What should happen]

**Actual Behavior:**
[What actually happens]

**Screenshots:**
[Attach screenshots if applicable]

**Logs:**
```
[Paste relevant logs from Output panel or plugin logs]
```

**Additional Context:**
[Any other relevant information]

---

### Where to File Bugs

- **GitHub Issues:** https://github.com/cboyd0319/BazBOM/issues
- Label: `plugin:vscode` or `plugin:intellij`
- Severity: `critical`, `high`, `medium`, `low`

---

## Test Checklist

### VS Code Extension

#### Installation
- [ ] Test 1: Manual installation from VSIX

#### Functionality
- [ ] Test 2: LSP server connection
- [ ] Test 3: Real-time scanning on save
- [ ] Test 4: Diagnostic severity levels
- [ ] Test 5: Manual scan command
- [ ] Test 6: Sync advisory database
- [ ] Test 7: Configuration settings
- [ ] Test 8: File type detection
- [ ] Test 9: Error handling

#### Performance
- [ ] Test 10: Scan performance
- [ ] Test 11: Memory usage

#### Compatibility
- [ ] Test 12: VS Code version compatibility

---

### IntelliJ IDEA Plugin

#### Installation
- [ ] Test 13: Manual installation from ZIP

#### Functionality
- [ ] Test 14: Tool window dependency tree
- [ ] Test 15: Real-time Maven annotation
- [ ] Test 16: Real-time Gradle annotation
- [ ] Test 17: Real-time Bazel annotation
- [ ] Test 18: Quick fix action (Alt+Enter)
- [ ] Test 19: Settings panel
- [ ] Test 20: Auto-scan on project open
- [ ] Test 21: Manual scan action
- [ ] Test 22: Database sync action

#### Performance
- [ ] Test 23: Scan performance
- [ ] Test 24: UI responsiveness
- [ ] Test 25: Memory usage

#### Compatibility
- [ ] Test 26: IntelliJ version compatibility
- [ ] Test 27: Operating system compatibility
- [ ] Test 28: Android Studio compatibility

---

### User Acceptance

- [ ] Test 29: First-time user experience
- [ ] Test 30: Developer workflow integration

---

### Accessibility

- [ ] Test 31: Screen reader compatibility
- [ ] Test 32: Keyboard navigation
- [ ] Test 33: Color contrast

---

## Testing Completion

**Completion Criteria:**

To consider testing complete and ready for marketplace publishing:

1.  All critical tests pass (Tests 1-22)
2.  Performance meets expectations (Tests 10, 11, 23-25)
3.  No critical bugs found
4.  Compatibility verified on primary platforms
5.  User acceptance tests pass
6.  Accessibility minimum requirements met

**Sign-Off:**

- Tester Name: ___________________
- Date: ___________________
- Test Environment: ___________________
- Overall Status: PASS / FAIL / CONDITIONAL PASS
- Notes: ___________________

---

## Next Steps After Testing

Once all tests pass:

1. **Documentation Review:**
   - Verify README accuracy
   - Update screenshots
   - Add demo videos

2. **Marketplace Preparation:**
   - Create marketplace listings
   - Prepare marketing materials
   - Set up support channels

3. **Soft Launch:**
   - Beta test with internal users
   - Gather feedback
   - Fix any remaining issues

4. **Public Launch:**
   - Publish to marketplaces
   - Announce on social media
   - Monitor for bug reports

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Maintained By:** BazBOM Development Team  
**Feedback:** https://github.com/cboyd0319/BazBOM/issues
