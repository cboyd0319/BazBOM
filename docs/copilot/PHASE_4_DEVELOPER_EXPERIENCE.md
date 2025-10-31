# Phase 4: Developer Experience Revolution

**Status:** Planned (Not Started)
**Priority:** 🔴 P0 - Critical Path
**Timeline:** Months 1-3 (12 weeks)
**Team Size:** 2-3 developers
**Dependencies:** Phase 0-3 (Complete)
**Budget:** $60K-90K (if hiring contractors)

---

## Executive Summary

**Goal:** Make BazBOM the tool developers WANT to use, not just security teams MANDATE.

**Current State:** BazBOM is a powerful CLI tool but lacks integration into developers' daily workflows. Competitors like Snyk dominate because they provide instant feedback in IDEs, automated fixes, and frictionless remediation.

**Target State:** Developers get real-time vulnerability warnings in their editor, fix CVEs with one click, and never leave their development environment.

**Success Metrics:**
- ✅ 500+ IntelliJ plugin downloads in first month
- ✅ <10 second scan time in IDE
- ✅ 90% of P0/P1 vulnerabilities auto-fixable
- ✅ 80% developer satisfaction score (user survey)

**Competitive Benchmark:** Match Snyk's IDE integration quality while adding unique advantages (Bazel support, build-time accuracy, privacy-preserving).

---

## Problem Statement

### Current Developer Workflow (Broken)

**Scenario:** Developer adds a new dependency to `pom.xml`

**Today's Experience (BazBOM):**
1. Edit `pom.xml`, add `<dependency>log4j-core:2.14.1</dependency>`
2. Save file
3. Switch to terminal
4. Run `bazbom scan .` (30-60 seconds)
5. Read JSON/SARIF output (requires security knowledge)
6. Search for fix: Google "log4j CVE safe version"
7. Edit `pom.xml` again, update version to 2.21.1
8. Run tests manually
9. Commit if tests pass

**Pain Points:**
- ❌ Context switching (editor → terminal → browser → editor)
- ❌ Slow feedback (30-60 seconds after saving)
- ❌ Manual fix research (requires security expertise)
- ❌ No automated testing (might break application)

### Desired Developer Workflow (Target)

**Scenario:** Developer adds a new dependency to `pom.xml`

**Tomorrow's Experience (BazBOM with Phase 4):**
1. Start typing `log4j-core` in `pom.xml`
2. **Inline warning appears:** "⚠️ log4j-core 2.14.1 has CVE-2021-44228 (CRITICAL, CISA KEV, reachable)"
3. Click "Quick Fix" → "Upgrade to safe version 2.21.1"
4. BazBOM updates version, runs tests in background
5. **Notification:** "✅ Tests passed. Safe to commit."
6. Developer continues coding (never left editor)

**Improvements:**
- ✅ Zero context switching (stays in editor)
- ✅ Instant feedback (<1 second inline warning)
- ✅ One-click fixes (no security expertise required)
- ✅ Automated testing (safety verification built-in)

---

## Phase Objectives

### 4.1 IDE Integration (Weeks 1-6)

**Deliverables:**
- IntelliJ IDEA plugin (primary focus)
- VS Code extension (secondary)
- Language Server Protocol (LSP) for cross-IDE reusability

### 4.2 Automated Remediation (Weeks 7-10)

**Deliverables:**
- `bazbom fix --apply` implementation for Maven, Gradle, Bazel
- Automated testing + rollback on failure
- PR generation for GitHub (GitLab/Bitbucket later)

### 4.3 Pre-Commit Hooks (Weeks 11-12)

**Deliverables:**
- `bazbom install-hooks` command
- Fast scan mode (<10 seconds)
- Policy enforcement (block P0/P1 commits)

---

## 4.1 IDE Integration - Detailed Specifications

### 4.1.1 IntelliJ IDEA Plugin

**Target Users:** Java/Kotlin/Scala developers using Maven, Gradle, or Bazel

**Plugin Name:** "BazBOM Security Scanner"
**Marketplace:** https://plugins.jetbrains.com/
**Compatibility:** IntelliJ IDEA 2023.3+ (Community & Ultimate)
**License:** MIT (same as BazBOM)

#### Core Features

**Feature 1: Dependency Tree Visualization**

**Description:** Side panel showing dependency tree with security status

**UI Design:**
```
┌─────────────────────────────────────────┐
│  BazBOM Security (Tool Window)          │
├─────────────────────────────────────────┤
│ Project: my-app                          │
│ Build System: Maven                      │
│ Last Scan: 2 minutes ago  [Refresh]     │
├─────────────────────────────────────────┤
│ Dependencies (247 total)                 │
│                                          │
│ ▼ Compile (189)                          │
│   ├── ✅ com.google.guava:guava:32.1.1  │
│   ├── ⚠️ log4j-core:2.17.0             │
│   │     CVE-2021-44832 (MEDIUM)        │
│   │     [Quick Fix]                     │
│   ├── 🔴 spring-web:5.3.20             │
│   │     CVE-2024-xxxx (CRITICAL, KEV)  │
│   │     Reachable                       │
│   │     [Quick Fix] [Details]          │
│   └── ✅ jackson-databind:2.15.2       │
│                                          │
│ ▼ Test (58)                              │
│   └── ✅ junit:junit:4.13.2             │
│                                          │
│ Vulnerabilities: 2 (1 CRITICAL, 1 MED) │
└─────────────────────────────────────────┘
```

**Implementation:**
- Tool window registered via `plugin.xml`
- TreeView component with custom cell renderers (red/yellow/green icons)
- Refresh button triggers `bazbom scan --format json`, parses SBOM
- Right-click context menu: "Show in dependency graph", "Fix vulnerability", "Ignore in VEX"

**Code Reference:**
```kotlin
// crates/bazbom-intellij-plugin/src/main/kotlin/io/bazbom/intellij/toolwindow/DependencyTreeWindow.kt
class DependencyTreeWindow(project: Project) : SimpleToolWindowPanel(true, true) {
    private val tree: Tree
    private val refreshButton: JButton

    init {
        // Build tree from SBOM JSON
        tree = Tree(buildTreeModel())
        tree.cellRenderer = DependencyNodeRenderer()  // Custom icons

        refreshButton.addActionListener {
            runBackgroundTask("Scanning with BazBOM") {
                val process = Runtime.getRuntime().exec("bazbom scan --format json .")
                val sbom = parseJsonOutput(process.inputStream)
                updateTreeModel(sbom)
            }
        }
    }
}
```

**Feature 2: Real-Time Vulnerability Highlighting**

**Description:** Inline warnings in `pom.xml`, `build.gradle`, `BUILD.bazel` for vulnerable dependencies

**UI Design:**
```xml
<!-- pom.xml -->
<dependency>
    <groupId>org.apache.logging.log4j</groupId>
    <artifactId>log4j-core</artifactId>
    <version>2.17.0</version>  ⚠️ CVE-2021-44832 (MEDIUM): RCE via JDBC Appender
</dependency>                      💡 Quick Fix: Upgrade to 2.21.1
```

**Implementation:**
- Register IntelliJ `Annotator` for XML, Groovy, Kotlin DSL, Starlark (Bazel)
- Parse dependency declarations using PSI (Program Structure Interface)
- Query BazBOM cache for vulnerabilities (fast lookup, no re-scan)
- Render inline warnings with `HighlightSeverity.WARNING` for MEDIUM/HIGH
- Render `HighlightSeverity.ERROR` for CRITICAL vulnerabilities

**Code Reference:**
```kotlin
// crates/bazbom-intellij-plugin/src/main/kotlin/io/bazbom/intellij/annotator/MavenDependencyAnnotator.kt
class MavenDependencyAnnotator : Annotator {
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        if (element is XmlTag && element.name == "dependency") {
            val groupId = element.findFirstSubTag("groupId")?.value?.text
            val artifactId = element.findFirstSubTag("artifactId")?.value?.text
            val version = element.findFirstSubTag("version")?.value?.text

            if (groupId != null && artifactId != null && version != null) {
                val vulns = BazBomCache.getVulnerabilities("$groupId:$artifactId:$version")

                if (vulns.isNotEmpty()) {
                    val critical = vulns.filter { it.severity == "CRITICAL" }
                    val severity = if (critical.isNotEmpty()) HighlightSeverity.ERROR else HighlightSeverity.WARNING

                    val message = vulns.joinToString(", ") { "${it.id} (${it.severity}): ${it.summary}" }
                    holder.newAnnotation(severity, message)
                        .range(element.findFirstSubTag("version")?.textRange)
                        .withFix(UpgradeDependencyQuickFix(groupId, artifactId, vulns.first().fixedVersion))
                        .create()
                }
            }
        }
    }
}
```

**Feature 3: One-Click Quick Fixes**

**Description:** IntelliJ Quick Fix actions to upgrade vulnerable dependencies

**User Experience:**
1. Developer sees inline warning: "⚠️ log4j-core 2.17.0 has CVE-2021-44832"
2. Presses `Alt+Enter` (quick fix shortcut)
3. Menu appears: "Upgrade to safe version 2.21.1"
4. Selects option
5. IntelliJ updates version in `pom.xml`, runs Maven reload
6. Background task runs tests
7. Notification: "✅ Tests passed. Dependency upgraded."

**Implementation:**
```kotlin
// crates/bazbom-intellij-plugin/src/main/kotlin/io/bazbom/intellij/quickfix/UpgradeDependencyQuickFix.kt
class UpgradeDependencyQuickFix(
    private val groupId: String,
    private val artifactId: String,
    private val targetVersion: String
) : IntentionAction {

    override fun getText() = "Upgrade to safe version $targetVersion"

    override fun invoke(project: Project, editor: Editor, file: PsiFile) {
        WriteCommandAction.runWriteCommandAction(project) {
            // Update version in XML/Gradle/Bazel
            val versionTag = findVersionTag(file, groupId, artifactId)
            versionTag?.value?.text = targetVersion

            // Trigger build system reload
            when (detectBuildSystem(project)) {
                BuildSystem.MAVEN -> MavenProjectsManager.getInstance(project).forceUpdateAllProjectsOrFindAllAvailablePomFiles()
                BuildSystem.GRADLE -> GradleProjectManager.getInstance(project).refreshProject(project.basePath!!)
                BuildSystem.BAZEL -> runBazelSync(project)
            }

            // Run tests in background
            ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Running tests after upgrade") {
                override fun run(indicator: ProgressIndicator) {
                    val result = runTests(project)
                    if (result.success) {
                        Notifications.Bus.notify(
                            Notification("BazBOM", "Upgrade Successful", "✅ Tests passed. Safe to commit.", NotificationType.INFORMATION)
                        )
                    } else {
                        Notifications.Bus.notify(
                            Notification("BazBOM", "Upgrade Failed", "❌ Tests failed. Reverting changes.", NotificationType.ERROR)
                        )
                        rollbackChanges()
                    }
                }
            })
        }
    }
}
```

**Feature 4: Build System Auto-Detection**

**Implementation:**
- Detect Maven: Look for `pom.xml` in project root
- Detect Gradle: Look for `build.gradle`, `build.gradle.kts`, `settings.gradle`
- Detect Bazel: Look for `WORKSPACE`, `MODULE.bazel`, `BUILD.bazel`
- Store detection result in project settings (cache for performance)

**Feature 5: Settings Panel**

**UI Design:**
```
┌─────────────────────────────────────────┐
│ BazBOM Settings                          │
├─────────────────────────────────────────┤
│ ☑ Enable real-time scanning             │
│ ☑ Show inline warnings                  │
│ ☑ Auto-scan on file save                │
│ ☐ Auto-scan on project open             │
│                                          │
│ Severity Thresholds:                     │
│   ☑ Show CRITICAL (error)               │
│   ☑ Show HIGH (warning)                 │
│   ☑ Show MEDIUM (warning)               │
│   ☐ Show LOW (info)                     │
│                                          │
│ Policy File: [bazbom.yml          ] 📁  │
│                                          │
│ BazBOM CLI Path:                         │
│   [/usr/local/bin/bazbom          ] 📁  │
│                                          │
│ [Test Connection]  [Reset to Defaults]  │
└─────────────────────────────────────────┘
```

**Implementation:**
- Settings stored in `.idea/bazbom.xml` (per-project)
- Test Connection button runs `bazbom --version` to verify CLI installation
- Policy file picker opens file chooser, validates YAML schema

#### Technical Architecture

**Plugin Structure:**
```
crates/bazbom-intellij-plugin/
├── src/main/
│   ├── kotlin/io/bazbom/intellij/
│   │   ├── BazBomPlugin.kt                 # Plugin entry point
│   │   ├── settings/
│   │   │   ├── BazBomSettings.kt           # Persistent state
│   │   │   └── BazBomConfigurable.kt       # Settings UI
│   │   ├── toolwindow/
│   │   │   ├── DependencyTreeWindow.kt     # Side panel
│   │   │   └── DependencyNodeRenderer.kt   # Custom icons
│   │   ├── annotator/
│   │   │   ├── MavenDependencyAnnotator.kt # pom.xml warnings
│   │   │   ├── GradleDependencyAnnotator.kt # build.gradle warnings
│   │   │   └── BazelDependencyAnnotator.kt # BUILD.bazel warnings
│   │   ├── quickfix/
│   │   │   └── UpgradeDependencyQuickFix.kt # Alt+Enter fix
│   │   ├── cache/
│   │   │   └── BazBomCache.kt              # In-memory SBOM cache
│   │   ├── actions/
│   │   │   ├── ScanProjectAction.kt        # Manual scan trigger
│   │   │   └── GenerateSbomAction.kt       # Export SBOM
│   │   └── util/
│   │       ├── BazBomCliRunner.kt          # Execute bazbom CLI
│   │       ├── BuildSystemDetector.kt      # Maven/Gradle/Bazel
│   │       └── TestRunner.kt               # Run project tests
│   └── resources/
│       ├── META-INF/
│       │   └── plugin.xml                  # Plugin descriptor
│       └── icons/
│           ├── bazbom-16.svg               # Tool window icon
│           ├── vulnerability-critical.svg  # Red icon
│           ├── vulnerability-high.svg      # Orange icon
│           └── vulnerability-medium.svg    # Yellow icon
├── build.gradle.kts                        # Gradle build config
└── README.md
```

**Dependencies:**
- IntelliJ Platform SDK 2023.3+
- Kotlin 1.9+
- Jackson (JSON parsing)
- kotlinx.serialization (SBOM deserialization)

**Build System:** Gradle with `org.jetbrains.intellij` plugin

**build.gradle.kts:**
```kotlin
plugins {
    id("org.jetbrains.kotlin.jvm") version "1.9.20"
    id("org.jetbrains.intellij") version "1.16.0"
}

group = "io.bazbom"
version = "1.0.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2023.3")
    type.set("IC")  // IntelliJ IDEA Community
    plugins.set(listOf("maven", "gradle", "Kotlin"))  // Required dependencies
}

dependencies {
    implementation("com.fasterxml.jackson.core:jackson-databind:2.15.2")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.0")
}

tasks {
    patchPluginXml {
        sinceBuild.set("233")
        untilBuild.set("241.*")
    }
}
```

#### Performance Optimization

**Challenge:** Real-time scanning can't block UI (must be <1 second)

**Solution: Multi-Tier Caching**

**Tier 1: In-Memory Cache (Fastest)**
- Store last scan results in HashMap
- Key: `"$groupId:$artifactId:$version"`
- Value: `List<Vulnerability>`
- Invalidate on file save or manual refresh
- Hit rate: ~95% for typical development (same deps repeatedly)

**Tier 2: Disk Cache (Fast)**
- BazBOM CLI caches SBOM in `.bazbom/cache/sbom.json`
- Plugin reads from cache if fresh (<5 minutes old)
- Avoids re-running `bazbom scan`
- Hit rate: ~80% for repeated scans

**Tier 3: Full Scan (Slow but Accurate)**
- Run `bazbom scan --format json .` (30-60 seconds)
- Update Tier 1 and Tier 2 caches
- Only when cache miss or user forces refresh

**Code:**
```kotlin
object BazBomCache {
    private val memoryCache = ConcurrentHashMap<String, List<Vulnerability>>()
    private var lastScanTime: Instant? = null

    fun getVulnerabilities(purl: String): List<Vulnerability> {
        // Tier 1: Memory cache
        memoryCache[purl]?.let { return it }

        // Tier 2: Disk cache
        val cacheFile = File(".bazbom/cache/sbom.json")
        if (cacheFile.exists() && cacheFile.lastModified() > System.currentTimeMillis() - 5.minutes.inWholeMilliseconds) {
            val sbom = Json.decodeFromString<Sbom>(cacheFile.readText())
            populateMemoryCache(sbom)
            return memoryCache[purl] ?: emptyList()
        }

        // Tier 3: Full scan (background task)
        runFullScan()
        return emptyList()  // Return empty for now, will populate on next annotation pass
    }

    private fun runFullScan() {
        if (lastScanTime != null && Duration.between(lastScanTime, Instant.now()) < 1.minutes) {
            return  // Debounce: Don't scan more than once per minute
        }

        ProgressManager.getInstance().run(object : Task.Backgroundable(null, "Scanning with BazBOM", false) {
            override fun run(indicator: ProgressIndicator) {
                val process = Runtime.getRuntime().exec("bazbom scan --format json .")
                val sbom = Json.decodeFromString<Sbom>(process.inputStream.bufferedReader().readText())
                populateMemoryCache(sbom)
                lastScanTime = Instant.now()
            }
        })
    }
}
```

#### Testing Strategy

**Unit Tests:**
- Test PURL parsing from XML/Gradle/Bazel
- Test vulnerability matching logic
- Test cache hit/miss scenarios
- Test quick fix version replacement

**Integration Tests:**
- Test with sample Maven project (e.g., Spring Boot starter)
- Test with sample Gradle project (e.g., Android app)
- Test with sample Bazel project (e.g., bazel-examples)
- Verify inline warnings appear
- Verify quick fixes update files correctly

**Manual Testing Checklist:**
- [ ] Plugin installs from JetBrains Marketplace
- [ ] Tool window appears in right sidebar
- [ ] Dependency tree loads for Maven project
- [ ] Inline warnings appear in pom.xml
- [ ] Quick fix upgrades version correctly
- [ ] Settings panel saves preferences
- [ ] Works with IntelliJ IDEA Community Edition
- [ ] Works with IntelliJ IDEA Ultimate Edition

#### Rollout Plan

**Alpha (Week 6):**
- Internal testing only
- Publish to alpha channel (manual download)
- Gather feedback from 10 test users

**Beta (Week 8):**
- JetBrains Marketplace (beta channel)
- Public announcement on GitHub, Twitter, Bazel Slack
- Target: 100 beta users
- Monitor crash reports, bug submissions

**GA (Week 10):**
- Promote to stable channel
- Press release, blog post
- Target: 500 downloads in first month

---

### 4.1.2 VS Code Extension

**Target Users:** Developers using VS Code (broader ecosystem than IntelliJ)

**Extension Name:** "BazBOM Security Scanner"
**Marketplace:** https://marketplace.visualstudio.com/
**Compatibility:** VS Code 1.85+
**License:** MIT

#### Core Features

**Feature 1: Language Server Protocol (LSP)**

**Why LSP?**
- Reusable across editors (VS Code, Vim, Emacs, Sublime Text)
- Standardized protocol for language tooling
- Reduces code duplication (write once, run everywhere)

**Architecture:**
```
┌─────────────────┐         ┌──────────────────┐         ┌─────────────┐
│   VS Code       │◄───────►│  BazBOM LSP      │◄───────►│  bazbom CLI │
│   Extension     │   LSP   │  Server (Rust)   │   exec  │             │
└─────────────────┘         └──────────────────┘         └─────────────┘
      │                              │                            │
      │                              │                            │
      ▼                              ▼                            ▼
   UI Layer                   Protocol Handler              Security Engine
   (TypeScript)               (tower-lsp crate)            (Existing code)
```

**Implementation:**

**LSP Server (Rust):**
```rust
// crates/bazbom-lsp/src/main.rs
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct BazBomLanguageServer {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for BazBomLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions::default())),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Trigger scan when pom.xml, build.gradle, or BUILD.bazel is saved
        let uri = params.text_document.uri;
        if uri.path().ends_with("pom.xml") || uri.path().ends_with("build.gradle") || uri.path().ends_with("BUILD.bazel") {
            self.scan_and_publish_diagnostics(&uri).await;
        }
    }
}

impl BazBomLanguageServer {
    async fn scan_and_publish_diagnostics(&self, uri: &Url) {
        // Run bazbom scan
        let output = std::process::Command::new("bazbom")
            .args(&["scan", "--format", "json", "."])
            .output()
            .expect("Failed to run bazbom");

        let sbom: Sbom = serde_json::from_slice(&output.stdout).unwrap();
        let diagnostics = self.convert_to_diagnostics(&sbom, uri);

        self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
    }

    fn convert_to_diagnostics(&self, sbom: &Sbom, uri: &Url) -> Vec<Diagnostic> {
        // Convert BazBOM findings to LSP diagnostics
        sbom.vulnerabilities.iter().map(|vuln| {
            Diagnostic {
                range: self.find_dependency_range(uri, &vuln.purl),  // Find line in file
                severity: Some(match vuln.severity.as_str() {
                    "CRITICAL" => DiagnosticSeverity::ERROR,
                    "HIGH" => DiagnosticSeverity::WARNING,
                    _ => DiagnosticSeverity::INFORMATION,
                }),
                message: format!("{} ({}): {}", vuln.id, vuln.severity, vuln.summary),
                source: Some("BazBOM".to_string()),
                ..Default::default()
            }
        }).collect()
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| BazBomLanguageServer { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

**VS Code Extension (TypeScript):**
```typescript
// crates/bazbom-vscode-extension/src/extension.ts
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

export function activate(context: vscode.ExtensionContext) {
    // Path to bazbom-lsp binary (bundled with extension or installed separately)
    const serverCommand = vscode.workspace.getConfiguration('bazbom').get<string>('lspPath') || 'bazbom-lsp';

    const serverOptions: ServerOptions = {
        command: serverCommand,
        args: []
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'xml', pattern: '**/pom.xml' },
            { scheme: 'file', language: 'groovy', pattern: '**/build.gradle' },
            { scheme: 'file', language: 'kotlin', pattern: '**/build.gradle.kts' },
            { scheme: 'file', language: 'starlark', pattern: '**/BUILD.bazel' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{pom.xml,build.gradle,build.gradle.kts,BUILD.bazel}')
        }
    };

    const client = new LanguageClient('bazbom', 'BazBOM Security Scanner', serverOptions, clientOptions);
    client.start();

    context.subscriptions.push(client);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('bazbom.scan', () => {
            vscode.window.showInformationMessage('Scanning with BazBOM...');
            // Trigger LSP scan
            client.sendRequest('bazbom/scan');
        })
    );
}

export function deactivate() {}
```

**package.json:**
```json
{
  "name": "bazbom",
  "displayName": "BazBOM Security Scanner",
  "description": "Real-time vulnerability scanning for Java projects",
  "version": "1.0.0",
  "publisher": "bazbom",
  "engines": {
    "vscode": "^1.85.0"
  },
  "categories": ["Linters", "Security"],
  "activationEvents": ["onLanguage:xml", "onLanguage:groovy", "onLanguage:kotlin"],
  "main": "./out/extension.js",
  "contributes": {
    "commands": [
      {
        "command": "bazbom.scan",
        "title": "BazBOM: Scan Project"
      }
    ],
    "configuration": {
      "title": "BazBOM",
      "properties": {
        "bazbom.lspPath": {
          "type": "string",
          "default": "bazbom-lsp",
          "description": "Path to bazbom-lsp binary"
        },
        "bazbom.enableRealTimeScanning": {
          "type": "boolean",
          "default": true,
          "description": "Enable real-time vulnerability scanning"
        }
      }
    }
  },
  "scripts": {
    "vscode:prepublish": "npm run compile",
    "compile": "tsc -p ./",
    "watch": "tsc -watch -p ./"
  },
  "dependencies": {
    "vscode-languageclient": "^9.0.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/vscode": "^1.85.0",
    "typescript": "^5.2.0"
  }
}
```

#### Differences from IntelliJ Plugin

**Simpler UI:**
- No custom tool window (VS Code uses Problems panel)
- Diagnostics shown inline with squiggles (red/yellow underlines)
- Hover for details (tooltip with CVE info)

**Faster Development:**
- LSP architecture means less code (Rust server does heavy lifting)
- TypeScript extension is thin wrapper (~200 lines vs. 2000 lines for IntelliJ)

**Broader Reach:**
- Works with VS Code, VSCodium, and any LSP-compatible editor
- Easier to maintain (bug fixes in Rust LSP benefit all editors)

#### Testing Strategy

**LSP Server Tests:**
- Unit tests for diagnostic conversion
- Integration tests with sample pom.xml files
- Protocol compliance tests (tower-lsp test suite)

**Extension Tests:**
- VS Code extension test framework
- Mock LSP server responses
- UI interaction tests (command palette, settings)

---

## 4.2 Automated Remediation - Detailed Specifications

**Goal:** Implement `bazbom fix --apply` to automatically upgrade vulnerable dependencies

**Current State:** Phase 4 not started (docs/copilot/IMPLEMENTATION_STATUS.md:304)

### 4.2.1 `bazbom fix` Command Design

**CLI Interface:**
```bash
# Show fix suggestions (safe, read-only)
bazbom fix --suggest

# Apply fixes automatically (writes to files)
bazbom fix --apply

# Generate PR (GitHub only initially)
bazbom fix --pr

# Dry run (show what would change without modifying files)
bazbom fix --dry-run

# Filter by severity
bazbom fix --apply --severity=critical,high

# Target specific vulnerabilities
bazbom fix --apply --cve=CVE-2021-44228
```

**Workflow:**
1. Run `bazbom scan` to identify vulnerabilities
2. Query advisories for fixed versions
3. Check compatibility (semver, breaking changes)
4. Update dependency files (pom.xml, build.gradle, maven_install.json)
5. Run tests to verify fixes don't break application
6. Commit changes or open PR

### 4.2.2 Maven Remediation Implementation

**File:** `crates/bazbom/src/fixes/maven.rs`

**Functionality:**
- Parse `pom.xml` using `quick-xml` crate
- Identify `<dependency>` elements with vulnerable versions
- Update version to safe value
- Handle `<dependencyManagement>` sections
- Respect version properties (e.g., `${log4j.version}`)

**Code:**
```rust
// crates/bazbom/src/fixes/maven.rs
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

pub struct MavenFixer {
    pom_path: PathBuf,
}

impl MavenFixer {
    pub fn apply_fixes(&self, fixes: &[Fix]) -> Result<()> {
        let pom_content = fs::read_to_string(&self.pom_path)?;
        let mut reader = Reader::from_str(&pom_content);
        let mut writer = Writer::new(Cursor::new(Vec::new()));

        let mut buf = Vec::new();
        let mut in_dependency = false;
        let mut current_group_id = String::new();
        let mut current_artifact_id = String::new();

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) if e.name().as_ref() == b"dependency" => {
                    in_dependency = true;
                    writer.write_event(Event::Start(e))?;
                }
                Event::Start(e) if e.name().as_ref() == b"groupId" && in_dependency => {
                    writer.write_event(Event::Start(e.clone()))?;
                    if let Event::Text(t) = reader.read_event_into(&mut buf)? {
                        current_group_id = String::from_utf8(t.to_vec())?;
                        writer.write_event(Event::Text(t))?;
                    }
                }
                Event::Start(e) if e.name().as_ref() == b"artifactId" && in_dependency => {
                    writer.write_event(Event::Start(e.clone()))?;
                    if let Event::Text(t) = reader.read_event_into(&mut buf)? {
                        current_artifact_id = String::from_utf8(t.to_vec())?;
                        writer.write_event(Event::Text(t))?;
                    }
                }
                Event::Start(e) if e.name().as_ref() == b"version" && in_dependency => {
                    writer.write_event(Event::Start(e.clone()))?;
                    if let Event::Text(t) = reader.read_event_into(&mut buf)? {
                        let current_version = String::from_utf8(t.to_vec())?;
                        let purl = format!("pkg:maven/{}/{}@{}", current_group_id, current_artifact_id, current_version);

                        // Check if we have a fix for this dependency
                        if let Some(fix) = fixes.iter().find(|f| f.purl == purl) {
                            println!("Upgrading {} from {} to {}", fix.package_name, current_version, fix.target_version);
                            writer.write_event(Event::Text(BytesText::new(&fix.target_version)))?;
                        } else {
                            writer.write_event(Event::Text(t))?;
                        }
                    }
                }
                Event::End(e) if e.name().as_ref() == b"dependency" => {
                    in_dependency = false;
                    current_group_id.clear();
                    current_artifact_id.clear();
                    writer.write_event(Event::End(e))?;
                }
                Event::Eof => break,
                e => writer.write_event(e)?,
            }
            buf.clear();
        }

        // Write updated content back to file
        let result = writer.into_inner().into_inner();
        fs::write(&self.pom_path, result)?;

        Ok(())
    }

    pub fn run_tests(&self) -> Result<bool> {
        // Run Maven tests to verify fix doesn't break application
        let output = Command::new("mvn")
            .args(&["test", "-DskipTests=false"])
            .output()?;

        Ok(output.status.success())
    }

    pub fn rollback(&self) -> Result<()> {
        // Restore from backup or git reset
        Command::new("git")
            .args(&["checkout", "HEAD", "pom.xml"])
            .status()?;
        Ok(())
    }
}
```

**Edge Cases:**
- Properties: `<version>${log4j.version}</version>` → Update property definition instead
- Parent POM: Check if version is inherited, update parent or child
- Dependency Management: Update in `<dependencyManagement>` section if present
- Exclusions: Preserve `<exclusions>` when upgrading

### 4.2.3 Gradle Remediation Implementation

**File:** `crates/bazbom/src/fixes/gradle.rs`

**Challenges:**
- Gradle DSL is not XML (Groovy or Kotlin DSL)
- No standard parser (must use regex or Gradle API)
- Version catalogs (TOML files) add complexity

**Approach 1: Regex Replacement (Simple, Fragile)**
```rust
// crates/bazbom/src/fixes/gradle.rs
pub fn apply_gradle_fix_regex(build_gradle: &Path, fixes: &[Fix]) -> Result<()> {
    let content = fs::read_to_string(build_gradle)?;
    let mut updated = content.clone();

    for fix in fixes {
        // Match: implementation 'org.apache.logging.log4j:log4j-core:2.17.0'
        let pattern = format!(
            r#"(['"]){}:{}:([^'"]+)(['"])"#,
            regex::escape(&fix.group_id),
            regex::escape(&fix.artifact_id)
        );
        let re = Regex::new(&pattern)?;

        updated = re.replace_all(&updated, |caps: &regex::Captures| {
            format!("{}{}:{}:{}{}",
                &caps[1], &fix.group_id, &fix.artifact_id, &fix.target_version, &caps[3]
            )
        }).to_string();
    }

    fs::write(build_gradle, updated)?;
    Ok(())
}
```

**Approach 2: Gradle Tooling API (Robust, Complex)**
```rust
// Use Gradle Tooling API via JNI
// More reliable but requires Java runtime and complex FFI
// Deferred to later iteration
```

**Approach 3: Version Catalog Updates (TOML)**
```rust
// crates/bazbom/src/fixes/gradle_version_catalog.rs
use toml_edit::{Document, value};

pub fn update_version_catalog(catalog_path: &Path, fixes: &[Fix]) -> Result<()> {
    let content = fs::read_to_string(catalog_path)?;
    let mut doc = content.parse::<Document>()?;

    for fix in fixes {
        let lib_key = format!("{}-{}", fix.group_id.replace('.', "-"), fix.artifact_id);

        if let Some(libs) = doc["libraries"].as_table_mut() {
            if let Some(lib) = libs.get_mut(&lib_key) {
                if let Some(version_ref) = lib.get("version").and_then(|v| v.as_str()) {
                    // Update version reference
                    if let Some(versions) = doc["versions"].as_table_mut() {
                        versions[version_ref] = value(&fix.target_version);
                    }
                } else if let Some(module) = lib.get_mut("module") {
                    // Inline version: { module = "group:artifact", version = "1.0" }
                    lib["version"] = value(&fix.target_version);
                }
            }
        }
    }

    fs::write(catalog_path, doc.to_string())?;
    Ok(())
}
```

**Testing:**
```bash
# Run Gradle tests after fix
gradle test --no-daemon
```

### 4.2.4 Bazel Remediation Implementation

**File:** `crates/bazbom/src/fixes/bazel.rs`

**Approach:** Update `maven_install.json` and re-pin dependencies

**Implementation:**
```rust
// crates/bazbom/src/fixes/bazel.rs
use serde_json::Value;

pub fn apply_bazel_fix(maven_install_json: &Path, fixes: &[Fix]) -> Result<()> {
    let content = fs::read_to_string(maven_install_json)?;
    let mut json: Value = serde_json::from_str(&content)?;

    if let Some(artifacts) = json["dependency_tree"]["dependencies"].as_array_mut() {
        for artifact in artifacts {
            if let Some(coord) = artifact["coord"].as_str() {
                // coord format: "group:artifact:version"
                let parts: Vec<&str> = coord.split(':').collect();
                if parts.len() == 3 {
                    let (group, artifact_id, version) = (parts[0], parts[1], parts[2]);
                    let purl = format!("pkg:maven/{}/{}@{}", group, artifact_id, version);

                    if let Some(fix) = fixes.iter().find(|f| f.purl == purl) {
                        // Update coord
                        artifact["coord"] = json!(format!("{}:{}:{}", group, artifact_id, fix.target_version));
                        println!("Updating Bazel dependency: {} -> {}", coord, fix.target_version);
                    }
                }
            }
        }
    }

    // Write updated JSON
    let updated = serde_json::to_string_pretty(&json)?;
    fs::write(maven_install_json, updated)?;

    // Re-pin dependencies with Bazel
    let output = Command::new("bazel")
        .args(&["run", "@maven//:pin"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to re-pin Bazel dependencies"));
    }

    Ok(())
}

pub fn run_bazel_tests(target: &str) -> Result<bool> {
    let output = Command::new("bazel")
        .args(&["test", target])
        .output()?;

    Ok(output.status.success())
}
```

**Bazel-Specific Workflow:**
1. Update `maven_install.json` with new versions
2. Run `bazel run @maven//:pin` to regenerate lockfile
3. Run `bazel test //...` to verify fixes
4. Commit both `maven_install.json` and generated files

### 4.2.5 Test Execution & Rollback

**Safety Principle:** Never apply fixes that break tests

**Implementation:**
```rust
// crates/bazbom/src/fixes/mod.rs
pub fn apply_fixes_with_testing(fixes: Vec<Fix>, config: &FixConfig) -> Result<()> {
    // Step 1: Create backup (git or file copy)
    create_backup()?;

    // Step 2: Apply fixes
    match config.build_system {
        BuildSystem::Maven => {
            let fixer = MavenFixer::new("pom.xml");
            fixer.apply_fixes(&fixes)?;
        }
        BuildSystem::Gradle => {
            let fixer = GradleFixer::new("build.gradle");
            fixer.apply_fixes(&fixes)?;
        }
        BuildSystem::Bazel => {
            let fixer = BazelFixer::new("maven_install.json");
            fixer.apply_fixes(&fixes)?;
        }
    }

    // Step 3: Run tests
    println!("Running tests to verify fixes...");
    let test_result = run_tests(&config.build_system)?;

    if test_result.success {
        println!("✅ Tests passed. Fixes applied successfully.");

        // Step 4: Commit if requested
        if config.auto_commit {
            commit_fixes(&fixes)?;
        }

        Ok(())
    } else {
        println!("❌ Tests failed. Rolling back changes.");
        rollback_backup()?;
        Err(anyhow!("Fixes broke tests. Rolled back changes.\n\nTest output:\n{}", test_result.output))
    }
}

fn run_tests(build_system: &BuildSystem) -> Result<TestResult> {
    let (command, args) = match build_system {
        BuildSystem::Maven => ("mvn", vec!["test", "-DskipTests=false"]),
        BuildSystem::Gradle => ("gradle", vec!["test", "--no-daemon"]),
        BuildSystem::Bazel => ("bazel", vec!["test", "//..."]),
    };

    let start = Instant::now();
    let output = Command::new(command).args(&args).output()?;
    let duration = start.elapsed();

    Ok(TestResult {
        success: output.status.success(),
        output: String::from_utf8_lossy(&output.stdout).to_string(),
        duration,
    })
}
```

### 4.2.6 PR Generation (GitHub)

**File:** `crates/bazbom/src/fixes/pr.rs`

**GitHub API Integration:**
```rust
// crates/bazbom/src/fixes/pr.rs
use octocrab::Octocrab;

pub async fn create_pr(fixes: &[Fix], config: &PrConfig) -> Result<String> {
    // Step 1: Create branch
    let branch_name = format!("bazbom/fix-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    Command::new("git")
        .args(&["checkout", "-b", &branch_name])
        .status()?;

    // Step 2: Apply fixes and commit
    apply_fixes_with_testing(fixes.to_vec(), &config.fix_config)?;

    let commit_message = generate_commit_message(fixes);
    Command::new("git")
        .args(&["add", "."])
        .status()?;
    Command::new("git")
        .args(&["commit", "-m", &commit_message])
        .status()?;

    // Step 3: Push branch
    Command::new("git")
        .args(&["push", "origin", &branch_name])
        .status()?;

    // Step 4: Create PR via GitHub API
    let octocrab = Octocrab::builder()
        .personal_token(config.github_token.clone())
        .build()?;

    let pr = octocrab
        .pulls(&config.repo_owner, &config.repo_name)
        .create(&format!("🔒 Fix {} vulnerabilities", fixes.len()), &branch_name, "main")
        .body(&generate_pr_body(fixes))
        .send()
        .await?;

    Ok(pr.html_url.to_string())
}

fn generate_pr_body(fixes: &[Fix]) -> String {
    let mut body = String::from("## 🔒 Security Fixes\n\n");
    body.push_str("This PR automatically upgrades vulnerable dependencies.\n\n");
    body.push_str("### Vulnerabilities Fixed:\n\n");

    for fix in fixes {
        body.push_str(&format!(
            "- **{}**: {} → {} (fixes {})\n  - **Severity:** {}\n  - **CVSS:** {}\n  - **CISA KEV:** {}\n  - **Reachable:** {}\n\n",
            fix.package_name,
            fix.current_version,
            fix.target_version,
            fix.cve_ids.join(", "),
            fix.severity,
            fix.cvss_score.unwrap_or(0.0),
            if fix.cisa_kev { "⚠️ YES" } else { "No" },
            if fix.reachable { "⚠️ YES" } else { "No" }
        ));
    }

    body.push_str("\n### Test Results:\n\n");
    body.push_str("✅ All tests passed after applying fixes.\n\n");
    body.push_str("### How to Review:\n\n");
    body.push_str("1. Check the diff to ensure only version numbers changed\n");
    body.push_str("2. Review CVE details linked above\n");
    body.push_str("3. Merge if changes look correct\n\n");
    body.push_str("---\n");
    body.push_str("🤖 Generated with [BazBOM](https://github.com/cboyd0319/BazBOM)\n");

    body
}

fn generate_commit_message(fixes: &[Fix]) -> String {
    let cve_ids: Vec<String> = fixes.iter()
        .flat_map(|f| f.cve_ids.clone())
        .collect();

    format!(
        "fix: upgrade dependencies to fix {}\n\nFixes: {}\n\n🤖 Generated with BazBOM\nCo-Authored-By: BazBOM <noreply@bazbom.io>",
        if cve_ids.len() == 1 {
            cve_ids[0].clone()
        } else {
            format!("{} vulnerabilities", cve_ids.len())
        },
        cve_ids.join(", ")
    )
}
```

**Example PR:**
```markdown
## 🔒 Security Fixes

This PR automatically upgrades vulnerable dependencies.

### Vulnerabilities Fixed:

- **log4j-core**: 2.17.0 → 2.21.1 (fixes CVE-2021-44832)
  - **Severity:** MEDIUM
  - **CVSS:** 6.6
  - **CISA KEV:** No
  - **Reachable:** ⚠️ YES

- **spring-web**: 5.3.20 → 5.3.31 (fixes CVE-2024-xxxx)
  - **Severity:** CRITICAL
  - **CVSS:** 9.8
  - **CISA KEV:** ⚠️ YES
  - **Reachable:** ⚠️ YES

### Test Results:

✅ All tests passed after applying fixes.

### How to Review:

1. Check the diff to ensure only version numbers changed
2. Review CVE details linked above
3. Merge if changes look correct

---
🤖 Generated with [BazBOM](https://github.com/cboyd0319/BazBOM)
```

---

## 4.3 Pre-Commit Hooks - Detailed Specifications

**Goal:** Block vulnerable code from entering repository

**User Experience:**
```bash
# Install hooks
$ bazbom install-hooks

✅ Installed pre-commit hook: .git/hooks/pre-commit
✅ Configured fast scan mode (policy enforcement)

# Developer makes commit
$ git commit -m "Add new feature"

🔍 Scanning dependencies with BazBOM...
⚠️ Found 1 CRITICAL vulnerability:
  - CVE-2024-xxxx in spring-web 5.3.20 (reachable)

❌ Commit blocked by BazBOM policy.

Fix with: bazbom fix --apply --cve=CVE-2024-xxxx
Or bypass with: git commit --no-verify
```

### Implementation

**File:** `crates/bazbom/src/hooks.rs`

```rust
// crates/bazbom/src/hooks.rs
pub fn install_hooks(config: &HooksConfig) -> Result<()> {
    let git_hooks_dir = Path::new(".git/hooks");
    if !git_hooks_dir.exists() {
        return Err(anyhow!("Not a git repository"));
    }

    let pre_commit_hook = git_hooks_dir.join("pre-commit");
    let hook_script = generate_hook_script(config);

    fs::write(&pre_commit_hook, hook_script)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_hook)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_hook, perms)?;
    }

    println!("✅ Installed pre-commit hook: {}", pre_commit_hook.display());
    Ok(())
}

fn generate_hook_script(config: &HooksConfig) -> String {
    format!(r#"#!/bin/bash
# BazBOM pre-commit hook
# Auto-generated by `bazbom install-hooks`

set -e

echo "🔍 Scanning dependencies with BazBOM..."

# Fast scan mode (skip reachability for speed)
bazbom scan --fast --format json . > /tmp/bazbom-scan.json

# Check policy violations
bazbom policy check --findings /tmp/bazbom-scan.json

if [ $? -ne 0 ]; then
  echo ""
  echo "❌ Commit blocked by BazBOM policy."
  echo ""
  echo "Fix with: bazbom fix --apply"
  echo "Or bypass with: git commit --no-verify"
  exit 1
fi

echo "✅ No policy violations. Proceeding with commit."
exit 0
"#)
}
```

**Fast Scan Mode:**
```rust
// Add to crates/bazbom/src/cli.rs
#[derive(Parser)]
pub struct ScanCommand {
    // ... existing fields

    /// Fast mode: Skip reachability analysis for speed
    #[arg(long)]
    fast: bool,
}

impl ScanCommand {
    pub fn execute(&self) -> Result<()> {
        if self.fast {
            // Skip reachability (5-10 second scan instead of 30-60 seconds)
            generate_sbom_only()?;
            scan_vulnerabilities()?;
        } else {
            // Full scan with reachability
            generate_sbom()?;
            scan_vulnerabilities()?;
            analyze_reachability()?;
        }

        Ok(())
    }
}
```

---

## Success Criteria & Acceptance Testing

### Phase 4 Completion Checklist

#### 4.1 IDE Integration
- [ ] IntelliJ plugin published to JetBrains Marketplace
- [ ] 500+ plugin downloads in first month
- [ ] Dependency tree visualization works for Maven, Gradle, Bazel
- [ ] Real-time vulnerability highlighting appears in <1 second
- [ ] Quick fixes successfully upgrade dependencies
- [ ] Tests run automatically after fix application
- [ ] Zero crashes reported in first week
- [ ] 80%+ user satisfaction score (plugin ratings)

#### 4.2 Automated Remediation
- [ ] `bazbom fix --apply` works for Maven projects
- [ ] `bazbom fix --apply` works for Gradle projects
- [ ] `bazbom fix --apply` works for Bazel projects
- [ ] Tests run automatically after fix application
- [ ] Rollback works when tests fail
- [ ] PR generation creates valid GitHub PRs
- [ ] 90%+ of P0/P1 vulnerabilities auto-fixable
- [ ] Zero data loss incidents (backups work)

#### 4.3 Pre-Commit Hooks
- [ ] `bazbom install-hooks` creates working pre-commit hook
- [ ] Fast scan mode completes in <10 seconds
- [ ] Policy violations block commits
- [ ] Bypass works with `git commit --no-verify`
- [ ] Works on macOS, Linux, Windows (Git Bash)

### Performance Benchmarks

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| IntelliJ scan time (10K deps) | <1 second | TBD | ⏳ |
| VS Code scan time (10K deps) | <2 seconds | TBD | ⏳ |
| `bazbom fix --apply` (Maven) | <30 seconds | TBD | ⏳ |
| `bazbom fix --apply` (Gradle) | <45 seconds | TBD | ⏳ |
| `bazbom fix --apply` (Bazel) | <60 seconds | TBD | ⏳ |
| Pre-commit scan (fast mode) | <10 seconds | TBD | ⏳ |
| PR generation time | <2 minutes | TBD | ⏳ |

### User Acceptance Testing

**Scenario 1: New Developer Onboarding**
- Fresh IntelliJ install
- Clone repo, open in IDE
- Install BazBOM plugin from Marketplace
- Scan runs automatically
- Vulnerabilities appear in side panel
- Click "Quick Fix" to upgrade dependency
- Tests run and pass
- Commit changes

**Expected Time:** <10 minutes
**Success Criteria:** Developer never reads documentation, everything "just works"

**Scenario 2: Security Team Audit**
- Security team mandates: "No CRITICAL vulnerabilities in prod"
- Developer runs `bazbom scan`
- Finds 5 CRITICAL vulnerabilities
- Runs `bazbom fix --apply --severity=critical`
- BazBOM upgrades all 5 dependencies
- Tests pass
- Creates PR with detailed CVE info
- Security team reviews and approves

**Expected Time:** <30 minutes
**Success Criteria:** Zero manual research, automated fix + testing, PR with audit trail

**Scenario 3: CI/CD Integration**
- Add pre-commit hook to repo
- Developer tries to commit code with vulnerable dependency
- Commit is blocked
- Developer runs `bazbom fix --apply`
- Commit succeeds
- CI pipeline passes

**Expected Time:** <5 minutes
**Success Criteria:** Policy enforcement prevents vulnerable code from entering repo

---

## Dependencies & Risks

### Technical Dependencies

**Hard Dependencies:**
- Phase 0-3 complete (Rust CLI, Maven/Gradle plugins, advisory engine)
- `bazbom` CLI installed and in PATH
- Git repository (for hooks and PR generation)

**Soft Dependencies:**
- GitHub account (for PR generation)
- JetBrains account (for IntelliJ plugin testing)
- VS Code installed (for extension testing)

### Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| **IntelliJ API changes break plugin** | Medium | High | Version pinning, compatibility range testing |
| **Automated fixes break applications** | High | Critical | Always run tests, rollback on failure, dry-run mode |
| **GitHub API rate limits** | Medium | Medium | Token authentication, exponential backoff |
| **Performance regressions in IDE** | Medium | High | Caching, background tasks, debouncing |
| **Cross-platform compatibility issues** | Medium | Medium | CI testing on macOS, Linux, Windows |
| **User adoption slower than expected** | High | Medium | Better marketing, tutorial videos, case studies |

---

## Resource Requirements

### Team Composition

**IntelliJ Plugin (6 weeks):**
- 1x Senior Kotlin/Java developer (IntelliJ Platform SDK experience)
- 0.5x UI/UX designer (for tool window layout)

**VS Code Extension (3 weeks):**
- 1x Rust developer (LSP server)
- 1x TypeScript developer (VS Code extension)

**Automated Remediation (8 weeks):**
- 2x Rust developers (fix logic, testing, PR generation)

**Pre-Commit Hooks (2 weeks):**
- 1x Rust developer (hooks, fast mode)

**Total:** ~2.5 FTE for 12 weeks = ~30 person-weeks = $60K-90K (contractors at $2K-3K/week)

### Infrastructure

**Required:**
- GitHub repository (already have)
- JetBrains Marketplace account (free)
- VS Code Marketplace account (free)
- CI/CD runners (GitHub Actions, already have)

**Optional:**
- Beta testing group (recruit from Bazel Slack)
- User analytics (privacy-preserving, e.g., PostHog self-hosted)

---

## Competitive Benchmark: BazBOM vs. Snyk (Post-Phase 4)

| Feature | Snyk | BazBOM (Phase 4) | Advantage |
|---------|------|------------------|-----------|
| **IntelliJ Plugin** | ✅ Excellent | ✅ Good (v1.0) | Snyk (mature) |
| **VS Code Extension** | ✅ Excellent | ✅ Good (v1.0) | Snyk (mature) |
| **Real-Time Scanning** | ✅ <1 sec | ✅ <1 sec | **PARITY** |
| **Quick Fixes** | ✅ One-click | ✅ One-click | **PARITY** |
| **Automated Testing** | ⚠️ Manual | ✅ Automatic | **BazBOM** |
| **PR Generation** | ✅ Advanced | ✅ Basic (v1.0) | Snyk (richer UI) |
| **Pre-Commit Hooks** | ✅ Native | ✅ Native | **PARITY** |
| **Bazel Support** | ❌ None | ✅ Full | **BazBOM** |
| **Build-Time Accuracy** | ❌ Post-build | ✅ Build-native | **BazBOM** |
| **Reachability Analysis** | ⚠️ Basic | ✅ ASM call graph | **BazBOM** |
| **Privacy** | ❌ Cloud-required | ✅ Offline-capable | **BazBOM** |
| **Cost** | $99-529/dev/year | **FREE** | **BazBOM** |

**Analysis:** After Phase 4, BazBOM achieves parity with Snyk on developer experience while maintaining unique advantages (Bazel, reachability, privacy, cost).

---

## Next Steps

**Week 1:**
- Set up IntelliJ plugin project structure
- Implement basic tool window with dependency tree
- Create "Hello World" VS Code extension with LSP

**Week 2:**
- Implement real-time vulnerability highlighting
- Add quick fix actions
- Test with sample Maven project

**Week 3:**
- Polish IntelliJ plugin UI
- Publish to alpha channel
- Recruit 10 beta testers

**Week 4:**
- Start automated remediation implementation
- Maven fix logic
- Test execution framework

**Week 5-6:**
- Gradle and Bazel fix logic
- PR generation
- End-to-end testing

**Week 7-8:**
- Pre-commit hooks
- Fast scan mode
- Policy enforcement

**Week 9:**
- Beta testing
- Bug fixes
- Documentation

**Week 10:**
- Publish to stable channels (JetBrains + VS Code Marketplace)
- Blog post: "Introducing BazBOM IDE Integration"
- Demo video

**Week 11-12:**
- Monitor adoption metrics
- Respond to user feedback
- Plan Phase 5

---

## Appendix: Code Samples

### Example SBOM Cache Structure

```json
{
  "scan_time": "2025-10-30T12:00:00Z",
  "project": "my-app",
  "build_system": "Maven",
  "dependencies": [
    {
      "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0",
      "name": "log4j-core",
      "version": "2.17.0",
      "scope": "compile",
      "vulnerabilities": [
        {
          "id": "CVE-2021-44832",
          "severity": "MEDIUM",
          "cvss": 6.6,
          "summary": "RCE via JDBC Appender",
          "fixed_version": "2.21.1",
          "cisa_kev": false,
          "epss": 0.02,
          "reachable": true
        }
      ]
    }
  ]
}
```

### Example IntelliJ Plugin Descriptor

```xml
<!-- crates/bazbom-intellij-plugin/src/main/resources/META-INF/plugin.xml -->
<idea-plugin>
  <id>io.bazbom.intellij</id>
  <name>BazBOM Security Scanner</name>
  <vendor email="support@bazbom.io" url="https://bazbom.io">BazBOM</vendor>

  <description><![CDATA[
    Real-time vulnerability scanning for Java projects using Maven, Gradle, or Bazel.

    Features:
    - Dependency tree visualization with security status
    - Inline vulnerability warnings in pom.xml, build.gradle, BUILD.bazel
    - One-click quick fixes to upgrade vulnerable dependencies
    - Automated testing after fixes
    - SLSA Level 3 provenance support
  ]]></description>

  <depends>com.intellij.modules.platform</depends>
  <depends>com.intellij.modules.java</depends>
  <depends>org.jetbrains.plugins.gradle</depends>
  <depends>org.jetbrains.idea.maven</depends>

  <extensions defaultExtensionNs="com.intellij">
    <toolWindow id="BazBOM" anchor="right"
                factoryClass="io.bazbom.intellij.toolwindow.DependencyTreeWindowFactory"
                icon="/icons/bazbom-16.svg"/>

    <annotator language="XML"
               implementationClass="io.bazbom.intellij.annotator.MavenDependencyAnnotator"/>
    <annotator language="Groovy"
               implementationClass="io.bazbom.intellij.annotator.GradleDependencyAnnotator"/>

    <projectConfigurable parentId="tools"
                         instance="io.bazbom.intellij.settings.BazBomConfigurable"
                         id="io.bazbom.settings"
                         displayName="BazBOM"/>
  </extensions>

  <actions>
    <action id="BazBOM.ScanProject"
            class="io.bazbom.intellij.actions.ScanProjectAction"
            text="Scan with BazBOM"
            description="Run BazBOM security scan">
      <add-to-group group-id="ToolsMenu" anchor="last"/>
    </action>
  </actions>
</idea-plugin>
```

---

**Last Updated:** 2025-10-30
**Next Review:** After Phase 4 Sprint 3 completion
**Owner:** BazBOM Maintainers

**Ready to revolutionize developer experience?** Let's build it! 🚀
