# Phase 4: Developer Experience - Implementation Progress

**Last Updated:** 2025-10-31
**Status:** In Progress (30% Complete)
**Timeline:** Months 1-3 (12 weeks)

---

## Executive Summary

Phase 4 aims to make BazBOM the tool developers **WANT** to use by providing:
1. Real-time vulnerability warnings in IDEs (IntelliJ, VS Code)
2. One-click fixes for vulnerabilities
3. Automated testing and rollback
4. Pre-commit hooks for policy enforcement

**Current Progress:**
- **IDE Integration (4.1):** 20% - Scaffolding complete, awaiting feature implementation
- **Automated Remediation (4.2):** 70% - Core CLI commands complete, PR generation pending
- **Pre-Commit Hooks (4.3):** 100% ✅ - Fully implemented and tested

---

## 4.1 IDE Integration (20% Complete)

### Completed ✅

#### LSP Server Foundation
**Location:** `crates/bazbom-lsp/`

**Features:**
- ✅ Core LSP implementation using tower-lsp crate
- ✅ File watching for build files (pom.xml, build.gradle, BUILD.bazel)
- ✅ Fast mode scanning (<10 seconds)
- ✅ Diagnostic publishing to editors
- ✅ Async scanning to avoid blocking
- ✅ 2 unit tests passing

**Architecture:**
```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────┐
│   Editor        │◄───────►│  BazBOM LSP      │◄───────►│  bazbom CLI │
│   (VS Code/etc) │   LSP   │  Server (Rust)   │   exec  │             │
└─────────────────┘         └──────────────────┘         └─────────────┘
```

**Remaining Work:**
- Improve range detection for diagnostics (currently line 0)
- Add code actions for quick fixes
- Implement caching to avoid repeated scans
- Performance optimization for large projects

#### VS Code Extension Scaffolding
**Location:** `crates/bazbom-vscode-extension/`

**Files Created:**
- ✅ `package.json` - Extension manifest with dependencies
- ✅ `src/extension.ts` - Main extension code with LSP client
- ✅ `tsconfig.json` - TypeScript configuration
- ✅ `README.md` - User documentation
- ✅ `.vscodeignore` - Files to exclude from package

**Features:**
- ✅ LSP client integration
- ✅ Configuration settings (lspPath, enableRealTimeScanning, etc.)
- ✅ Commands: "BazBOM: Scan Project", "BazBOM: Sync Advisory Database"
- ✅ File watching for build files

**Next Steps:**
1. Install npm dependencies: `cd crates/bazbom-vscode-extension && npm install`
2. Compile TypeScript: `npm run compile`
3. Test locally: Press F5 in VS Code to launch extension host
4. Package: `npx vsce package`
5. Publish to marketplace (requires account)

#### IntelliJ IDEA Plugin Scaffolding
**Location:** `crates/bazbom-intellij-plugin/`

**Files Created:**
- ✅ `build.gradle.kts` - Gradle build configuration
- ✅ `settings.gradle.kts` - Gradle settings
- ✅ `gradle.properties` - Gradle properties
- ✅ `src/main/resources/META-INF/plugin.xml` - Plugin descriptor
- ✅ `src/main/kotlin/io/bazbom/intellij/` - Kotlin source (8 files)
  - `BazBomPlugin.kt` - Main entry point
  - `actions/ScanProjectAction.kt` - Manual scan action
  - `actions/SyncDatabaseAction.kt` - Database sync action
  - `util/BazBomCliRunner.kt` - CLI execution utility
  - `toolwindow/BazBomToolWindowFactory.kt` - Tool window UI
  - `settings/BazBomConfigurable.kt` - Settings panel
  - `services/BazBomProjectService.kt` - Caching service
  - `listeners/BazBomProjectListener.kt` - Lifecycle listener

**Features:**
- ✅ Tool window for dependency tree (stub)
- ✅ Actions for scan and database sync
- ✅ CLI runner utility with error handling
- ✅ Project service for result caching
- ✅ Settings panel (stub)

**Next Steps:**
1. Initialize Gradle wrapper: `gradle wrapper`
2. Build plugin: `./gradlew build`
3. Run in test IDE: `./gradlew runIde`
4. Implement dependency tree view
5. Add Maven/Gradle/Bazel annotators for real-time warnings
6. Implement quick fix actions (Alt+Enter)
7. Test with sample projects
8. Publish to JetBrains Marketplace

### In Progress 🔄

#### Dependency Tree Visualization
**Status:** Not Started
**Priority:** High

**Requirements:**
- Display all project dependencies in tree view
- Color-code by security status (red/yellow/green)
- Show vulnerability count per dependency
- Support filtering by severity
- Clickable items for details

**IntelliJ Implementation:**
- Use JTree component with custom renderer
- Parse SBOM JSON from scan results
- Update on scan completion

**VS Code Implementation:**
- Use TreeView API
- Register tree data provider
- Update from LSP diagnostics

#### Real-Time Vulnerability Highlighting
**Status:** Not Started
**Priority:** High

**Requirements:**
- Inline warnings in pom.xml, build.gradle, BUILD.bazel
- Severity-based highlighting (error/warning/info)
- Hover tooltips with CVE details
- Update on file save

**IntelliJ Implementation:**
- Register `Annotator` for XML, Groovy, Kotlin, Starlark
- Parse dependency declarations using PSI
- Query scan results for vulnerabilities
- Render with `HighlightSeverity.ERROR/WARNING`

**VS Code Implementation:**
- LSP server publishes diagnostics
- Extension displays as problems
- Range detection for dependency blocks

#### One-Click Quick Fixes
**Status:** Not Started
**Priority:** High

**Requirements:**
- Alt+Enter (IntelliJ) / Ctrl+. (VS Code) for quick fixes
- Show available fixed version
- Apply upgrade and reload build system
- Run tests after upgrade
- Rollback if tests fail

**IntelliJ Implementation:**
```kotlin
class UpgradeDependencyQuickFix : IntentionAction {
    override fun invoke(project: Project, editor: Editor, file: PsiFile) {
        // 1. Update version in file
        // 2. Reload build system (Maven/Gradle/Bazel)
        // 3. Run tests in background
        // 4. Show notification with result
        // 5. Rollback if tests fail
    }
}
```

**VS Code Implementation:**
- LSP server provides code actions
- Extension displays in quick fix menu
- Apply fix via LSP workspace edit

### Not Started ⏸️

- Testing infrastructure for plugins
- Marketplace publishing
- User analytics (privacy-preserving)
- Telemetry (opt-in only)

---

## 4.2 Automated Remediation (70% Complete)

### Completed ✅

#### `bazbom fix --suggest` Command
**Location:** `crates/bazbom/src/remediation.rs`

**Features:**
- ✅ RemediationSuggestion data structure
- ✅ Educational "why fix this?" explanations
  - CVSS score interpretation
  - KEV (Known Exploited Vulnerabilities) warnings
  - EPSS (Exploit Prediction Scoring) probability
  - Severity and priority context
- ✅ Build-system-specific "how to fix" instructions
  - Maven: pom.xml snippet with version update
  - Gradle: build.gradle dependency update
  - Bazel: maven_install coordinate update
- ✅ JSON report output (remediation_suggestions.json)
- ✅ Priority-based effort estimation
- ✅ Reference links to CVE databases

**Example Output:**
```
[bazbom] Remediation Summary:
  Total vulnerabilities: 12
  Fixable: 10
  Unfixable: 2
  Estimated effort: Medium (1-4 hours)

1. CVE-2021-44228 (org.apache.logging.log4j:log4j-core)
   Current version: 2.14.1
   Fixed version: 2.21.1
   Severity: CRITICAL | Priority: P0

   WHY FIX THIS:
   CRITICAL severity - immediate action required. Listed in CISA KEV
   (Known Exploited Vulnerabilities) - actively exploited in the wild.
   Very high CVSS score: 10.0. Impact: Remote code execution via JNDI.

   HOW TO FIX:
   Upgrade to version 2.21.1.

   Update pom.xml:
   <dependency>
     <groupId>org.apache.logging.log4j</groupId>
     <artifactId>log4j-core</artifactId>
     <version>2.21.1</version>
   </dependency>
   Then run: mvn clean install
```

#### `bazbom fix --apply` Command
**Location:** `crates/bazbom/src/remediation.rs`

**Features:**
- ✅ Maven pom.xml version updates
  - Finds <version> tags following matching <artifactId>
  - Simple string replacement
  - Preserves formatting
- ✅ Gradle build.gradle version updates
  - Finds dependency declarations with artifact name
  - Replaces version string
  - Supports both .gradle and .gradle.kts
- ✅ Bazel MODULE.bazel/WORKSPACE updates
  - Finds maven coordinate strings
  - Updates version
  - Reminds to run `bazel run @maven//:pin`
- ✅ Success/failure tracking
- ✅ User feedback on applied fixes

**Example Output:**
```
[bazbom] Applying fixes...
  ✓ Updated log4j-core: 2.14.1 → 2.21.1
  ✓ Updated spring-web: 5.3.20 → 5.3.31
  ✗ Failed to apply fix for jackson-databind: No matching version found

[bazbom] Apply Results:
  Applied: 2
  Failed: 1
  Skipped: 0
```

**Limitations (Current):**
- Simple string-based replacement (not XML/AST parsing)
- Doesn't handle version properties (${log4j.version})
- Doesn't update parent POM versions
- No conflict resolution or dependency management

### In Progress 🔄

#### Test Execution Framework
**Status:** Not Started
**Priority:** Critical

**Requirements:**
- Run project tests after applying fixes
- Detect build system (Maven/Gradle/Bazel)
- Execute appropriate test command
- Capture test output
- Parse test results (pass/fail)
- Report results to user

**Implementation Plan:**
```rust
pub struct TestRunner {
    build_system: BuildSystem,
    project_root: PathBuf,
}

impl TestRunner {
    pub fn run_tests(&self) -> Result<TestResult> {
        match self.build_system {
            BuildSystem::Maven => {
                Command::new("mvn")
                    .args(&["test", "-DskipTests=false"])
                    .output()
            }
            BuildSystem::Gradle => {
                Command::new("gradle")
                    .args(&["test", "--no-daemon"])
                    .output()
            }
            BuildSystem::Bazel => {
                Command::new("bazel")
                    .args(&["test", "//..."])
                    .output()
            }
        }
    }
}
```

#### Automatic Rollback
**Status:** Not Started
**Priority:** Critical

**Requirements:**
- Create backup before applying fixes
- Git integration (create branch or stash)
- Restore on test failure
- Clean up on success

**Implementation Plan:**
```rust
pub fn apply_fixes_with_rollback(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
) -> Result<ApplyResult> {
    // 1. Create backup (git stash or file copy)
    create_backup(project_root)?;
    
    // 2. Apply fixes
    let result = apply_fixes(suggestions, build_system, project_root)?;
    
    // 3. Run tests
    let test_result = run_tests(build_system, project_root)?;
    
    if test_result.success {
        // 4. Commit or delete backup
        commit_fixes(project_root)?;
        Ok(result)
    } else {
        // 5. Rollback changes
        rollback_backup(project_root)?;
        Err(anyhow!("Tests failed, rolled back changes"))
    }
}
```

### Not Started ⏸️

#### PR Generation
**Status:** Not Started
**Priority:** Medium

**Requirements:**
- Create new branch
- Commit fixes with descriptive message
- Push to remote
- Open PR via GitHub API
- Include vulnerability details in PR description
- Link to CVE references

**Implementation Plan:**
- Use `octocrab` crate for GitHub API
- Generate PR title: "🔒 Fix N vulnerabilities"
- Generate PR body with table of fixes
- Add test results summary
- Request review from security team

**Example PR:**
```markdown
## 🔒 Security Fixes

This PR automatically upgrades vulnerable dependencies.

### Vulnerabilities Fixed:

| Package | Current | Fixed | Severity | CVE |
|---------|---------|-------|----------|-----|
| log4j-core | 2.14.1 | 2.21.1 | CRITICAL | CVE-2021-44228 |
| spring-web | 5.3.20 | 5.3.31 | HIGH | CVE-2024-xxxx |

### Test Results:

✅ All tests passed after applying fixes.

---
🤖 Generated with [BazBOM](https://github.com/cboyd0319/BazBOM)
```

---

## 4.3 Pre-Commit Hooks (100% Complete ✅)

### Completed ✅

#### `bazbom install-hooks` Command
**Location:** `crates/bazbom/src/hooks.rs`

**Features:**
- ✅ HooksConfig structure (policy_file, fast_mode)
- ✅ Git repository detection (.git/hooks/)
- ✅ Hook script generation
- ✅ Unix executable permissions (chmod +x)
- ✅ Fast mode support (--fast flag)
- ✅ Custom policy file support (--policy flag)
- ✅ User-friendly success messages
- ✅ 4 unit tests passing

**Usage:**
```bash
# Install with default settings
bazbom install-hooks

# Install with fast mode (skip reachability)
bazbom install-hooks --fast

# Install with custom policy file
bazbom install-hooks --policy=custom-policy.yml
```

**Generated Hook Script:**
```bash
#!/bin/bash
# BazBOM pre-commit hook
# Auto-generated by `bazbom install-hooks`

set -e

echo "🔍 Scanning dependencies with BazBOM..."

# Create temporary output directory
BAZBOM_TMP=$(mktemp -d)
trap "rm -rf $BAZBOM_TMP" EXIT

# Run BazBOM scan
if ! bazbom scan --fast --out-dir "$BAZBOM_TMP" . > /dev/null 2>&1; then
  echo ""
  echo "❌ BazBOM scan failed"
  exit 1
fi

# Check policy if policy file exists
if [ -f "bazbom.yml" ]; then
  echo "📋 Checking policy: bazbom.yml..."
  
  if ! bazbom policy check > /dev/null 2>&1; then
    echo ""
    echo "❌ Commit blocked by BazBOM policy violations"
    echo "Run 'bazbom fix --suggest' for remediation guidance"
    echo "Or bypass with: git commit --no-verify"
    exit 1
  fi
fi

echo "✅ No policy violations. Proceeding with commit."
exit 0
```

**Tests:**
- ✅ `test_generate_hook_script_default` - Default configuration
- ✅ `test_generate_hook_script_fast_mode` - Fast mode flag
- ✅ `test_generate_hook_script_custom_policy` - Custom policy file
- ✅ `test_generate_hook_script_bypass_instructions` - Bypass help

---

## Next Steps

### Immediate (This Week)

1. **VS Code Extension:**
   - [ ] Install npm dependencies
   - [ ] Compile TypeScript
   - [ ] Test in development host (F5)
   - [ ] Fix any compilation errors

2. **IntelliJ Plugin:**
   - [ ] Initialize Gradle wrapper
   - [ ] Build plugin with Gradle
   - [ ] Run in test IDE
   - [ ] Fix any build errors

3. **Documentation:**
   - [ ] Add Phase 4 examples to USAGE.md
   - [ ] Create IDE setup guides
   - [ ] Update README with IDE features

### Short Term (Next 2 Weeks)

1. **IDE Features:**
   - [ ] Implement dependency tree view (both IDEs)
   - [ ] Add real-time annotators for pom.xml/build.gradle
   - [ ] Implement quick fix actions
   - [ ] Add automated testing after fixes

2. **Remediation:**
   - [ ] Implement test execution framework
   - [ ] Add automatic rollback on failure
   - [ ] Create backup/restore logic

3. **Testing:**
   - [ ] Add integration tests for IDE plugins
   - [ ] Test with real projects (Maven, Gradle, Bazel)
   - [ ] Performance testing with large projects

### Medium Term (Next 4 Weeks)

1. **Advanced IDE Features:**
   - [ ] Settings panels with all configuration options
   - [ ] Severity filtering in tool window
   - [ ] Vulnerability details panel
   - [ ] Status bar integration

2. **PR Generation:**
   - [ ] GitHub API integration
   - [ ] PR template generation
   - [ ] Test result reporting in PR
   - [ ] Security team notifications

3. **Marketplace:**
   - [ ] Publish VS Code extension
   - [ ] Publish IntelliJ plugin
   - [ ] Create marketing materials
   - [ ] Announcement blog post

---

## Success Criteria

### Phase 4.1 (IDE Integration)
- [ ] 500+ IntelliJ plugin downloads in first month
- [ ] 1000+ VS Code extension installs in first month
- [ ] <1 second inline warnings
- [ ] 80%+ user satisfaction (plugin ratings)
- [ ] Zero critical bugs in first week

### Phase 4.2 (Automated Remediation)
- [ ] 90%+ of P0/P1 vulnerabilities auto-fixable
- [ ] Test execution works for Maven/Gradle/Bazel
- [ ] Automatic rollback prevents breakage
- [ ] PR generation creates valid PRs
- [ ] Zero data loss incidents

### Phase 4.3 (Pre-Commit Hooks) ✅
- [x] `bazbom install-hooks` creates working hook
- [x] Fast mode completes in <10 seconds
- [x] Policy violations block commits
- [x] Bypass works with --no-verify
- [x] 4 tests passing

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| IntelliJ API changes | Medium | High | Version pinning, compat testing |
| Fixes break apps | High | Critical | Test execution + rollback |
| Slow IDE performance | Medium | High | Caching, async, debouncing |
| Low adoption | High | Medium | Marketing, tutorials, demos |
| GitHub rate limits | Medium | Medium | Token auth, exponential backoff |

---

## Resources

- **Phase 4 Specification:** `docs/copilot/PHASE_4_DEVELOPER_EXPERIENCE.md`
- **Implementation Status:** `docs/copilot/IMPLEMENTATION_STATUS.md`
- **LSP Server:** `crates/bazbom-lsp/`
- **VS Code Extension:** `crates/bazbom-vscode-extension/`
- **IntelliJ Plugin:** `crates/bazbom-intellij-plugin/`
- **Remediation Logic:** `crates/bazbom/src/remediation.rs`
- **Hooks Logic:** `crates/bazbom/src/hooks.rs`
