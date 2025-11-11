# Marketplace Publishing Guide

**Last Updated:** 2025-11-05  
**Status:** Active  
**Target:** Maintainers publishing to VS Code and JetBrains marketplaces

---

## Overview

This guide provides step-by-step instructions for publishing BazBOM IDE plugins to their respective marketplaces.

**Marketplaces:**
1. VS Code Marketplace (Microsoft)
2. JetBrains Marketplace (IntelliJ IDEA, Android Studio, etc.)

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [VS Code Marketplace Publishing](#vs-code-marketplace-publishing)
3. [JetBrains Marketplace Publishing](#jetbrains-marketplace-publishing)
4. [Marketing Materials](#marketing-materials)
5. [Post-Publishing](#post-publishing)
6. [Maintenance and Updates](#maintenance-and-updates)

---

## Prerequisites

### Required Accounts

**VS Code Marketplace:**
- Azure DevOps account (required for publisher access)
- GitHub account (for repository linking)
- Publisher ID (created in step 1 below)

**JetBrains Marketplace:**
- JetBrains Account (create at https://account.jetbrains.com/)
- Plugin repository access (for uploading)

### Required Tools

```bash
# Install VS Code Extension Manager
npm install -g @vscode/vsce

# Verify installation
vsce --version
```

### Required Assets

Before publishing, ensure you have:

- [ ] Plugin builds successfully
- [ ] All tests pass
- [ ] README.md with screenshots
- [ ] CHANGELOG.md with version history
- [ ] LICENSE file (MIT)
- [ ] Icon assets (multiple sizes)
- [ ] Demo video or GIFs (optional but recommended)

---

## VS Code Marketplace Publishing

### Step 1: Create Publisher Account

1. Go to https://marketplace.visualstudio.com/manage
2. Sign in with Microsoft account
3. Click "Create Publisher"
4. Fill in details:
   - **Publisher ID:** `cboyd0319` (or your preferred ID, lowercase, alphanumeric + hyphens)
   - **Display Name:** `BazBOM Security`
   - **Description:** `Security scanning tools for JVM projects`
   - **Website:** `https://github.com/cboyd0319/BazBOM`

5. Click "Create"

### Step 2: Generate Personal Access Token (PAT)

1. Go to https://dev.azure.com/
2. Click user icon (top right) → "Personal access tokens"
3. Click "+ New Token"
4. Configure token:
   - **Name:** `BazBOM VS Code Publishing`
   - **Organization:** All accessible organizations
   - **Expiration:** 1 year (or custom)
   - **Scopes:** Custom defined → **Marketplace** → **Manage** (check this box)
5. Click "Create"
6. **IMPORTANT:** Copy token immediately (you won't see it again)
7. Store securely (GitHub Secrets recommended)

```bash
# Store token in environment
export VSCE_PAT="your-token-here"
```

### Step 3: Update package.json

Edit `crates/bazbom-vscode-extension/package.json`:

```json
{
  "publisher": "cboyd0319",
  "repository": {
    "type": "git",
    "url": "https://github.com/cboyd0319/BazBOM.git"
  },
  "bugs": {
    "url": "https://github.com/cboyd0319/BazBOM/issues"
  },
  "homepage": "https://github.com/cboyd0319/BazBOM",
  "icon": "assets/icon.png",
  "galleryBanner": {
    "color": "#1e1e1e",
    "theme": "dark"
  },
  "keywords": [
    "security",
    "sbom",
    "vulnerability",
    "maven",
    "gradle",
    "bazel",
    "java",
    "kotlin",
    "scala"
  ],
  "categories": [
    "Linters",
    "Programming Languages",
    "Other"
  ]
}
```

### Step 4: Prepare Assets

**Icon Requirements:**
- Size: 128x128 pixels (minimum)
- Format: PNG
- Transparent background recommended
- Square aspect ratio

```bash
# Create assets directory
mkdir -p crates/bazbom-vscode-extension/assets

# Add icon (create/place icon file here)
# crates/bazbom-vscode-extension/assets/icon.png
```

### Step 5: Create CHANGELOG.md

```markdown
# Change Log

## [1.0.0] - 2025-11-05

### Added
- Initial release
- Real-time vulnerability scanning
- Support for Maven, Gradle, and Bazel
- Inline diagnostics with severity indicators
- Fast mode scanning (<10 seconds)
- Privacy-first architecture (100% local)

### Features
- LSP-based architecture
- Automatic scanning on file save
- Manual scan command
- Advisory database sync
- Configurable severity thresholds
```

### Step 6: Build and Package

```bash
cd crates/bazbom-vscode-extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Run tests (if any)
npm test

# Package extension
npx vsce package

# Output: bazbom-1.0.0.vsix
```

### Step 7: Test Package Locally

```bash
# Install locally for testing
code --install-extension bazbom-1.0.0.vsix

# Or in VS Code:
# Extensions → ... → Install from VSIX
```

### Step 8: Publish to Marketplace

**Option A: Using CLI (Recommended)**

```bash
# Login with PAT
npx vsce login cboyd0319
# Paste your PAT when prompted

# Publish (will build automatically)
npx vsce publish

# Or specify version bump
npx vsce publish patch  # 1.0.0 → 1.0.1
npx vsce publish minor  # 1.0.0 → 1.1.0
npx vsce publish major  # 1.0.0 → 2.0.0
```

**Option B: Manual Upload**

1. Go to https://marketplace.visualstudio.com/manage/publishers/cboyd0319
2. Click "+ New Extension" → "Visual Studio Code"
3. Upload `.vsix` file
4. Fill in marketplace details
5. Click "Upload"

### Step 9: Verify Publication

1. Go to https://marketplace.visualstudio.com/items?itemName=cboyd0319.bazbom
2. Verify:
   - Extension appears correctly
   - Screenshots display
   - README renders properly
   - Install button works
3. Test installation:
   ```bash
   code --install-extension cboyd0319.bazbom
   ```

---

## JetBrains Marketplace Publishing

### Step 1: Create JetBrains Account

1. Go to https://account.jetbrains.com/
2. Sign up or log in
3. Verify email address

### Step 2: Configure Plugin Descriptor

Edit `crates/bazbom-intellij-plugin/src/main/resources/META-INF/plugin.xml`:

```xml
<idea-plugin>
  <id>io.bazbom.intellij-plugin</id>
  <name>BazBOM Security Scanner</name>
  <vendor email="support@bazbom.io" url="https://github.com/cboyd0319/BazBOM">BazBOM</vendor>
  
  <description><![CDATA[
    Real-time vulnerability scanning for Java projects using Maven, Gradle, or Bazel.
    
    <h2>Features</h2>
    <ul>
      <li>Dependency tree visualization with security indicators</li>
      <li>Real-time vulnerability highlighting in build files</li>
      <li>One-click fixes with automated testing</li>
      <li>Support for Maven, Gradle, and Bazel</li>
      <li>Privacy-first: 100% local scanning</li>
    </ul>
    
    <h2>Requirements</h2>
    <ul>
      <li>BazBOM CLI installed and in PATH</li>
      <li>Advisory database synced (bazbom db sync)</li>
    </ul>
    
    <h2>Quick Start</h2>
    <ol>
      <li>Open a Java project</li>
      <li>Click BazBOM in the right sidebar</li>
      <li>Click "Scan" button</li>
      <li>View vulnerabilities in dependency tree</li>
      <li>Use Alt+Enter on vulnerable dependencies for quick fixes</li>
    </ol>
  ]]></description>
  
  <change-notes><![CDATA[
    <h3>Version 1.0.0</h3>
    <ul>
      <li>Initial release</li>
      <li>Dependency tree visualization</li>
      <li>Real-time vulnerability highlighting</li>
      <li>One-click quick fixes</li>
      <li>Automated testing after fixes</li>
      <li>Support for Maven, Gradle, and Bazel</li>
    </ul>
  ]]></change-notes>
  
  <!-- ... rest of plugin.xml ... -->
</idea-plugin>
```

### Step 3: Update build.gradle.kts

Edit `crates/bazbom-intellij-plugin/build.gradle.kts`:

```kotlin
plugins {
    id("org.jetbrains.kotlin.jvm") version "1.9.21"
    id("org.jetbrains.intellij") version "1.17.4"
}

group = "io.bazbom"
version = "6.0.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2023.3")
    type.set("IC") // IC = Community, IU = Ultimate
    plugins.set(listOf("java", "Kotlin", "Gradle", "maven"))
}

tasks {
    patchPluginXml {
        sinceBuild.set("233")  // 2023.3
        untilBuild.set("242.*")  // 2024.2
    }
    
    publishPlugin {
        token.set(System.getenv("JETBRAINS_TOKEN"))
        channels.set(listOf("stable"))  // or "beta", "alpha"
    }
}
```

### Step 4: Create Plugin Icon

**Icon Requirements:**
- Size: 40x40 pixels (pluginIcon.svg) and 80x80 pixels (pluginIcon@2x.svg)
- Format: SVG preferred, PNG acceptable
- Transparent background
- Square aspect ratio

```bash
# Create icons
mkdir -p crates/bazbom-intellij-plugin/src/main/resources/META-INF

# Add icons:
# - pluginIcon.svg (40x40)
# - pluginIcon@2x.svg (80x80)
```

### Step 5: Build Plugin

```bash
cd crates/bazbom-intellij-plugin

# Clean and build
./gradlew clean buildPlugin

# Output: build/distributions/bazbom-intellij-plugin-1.0.0.zip
```

### Step 6: Test Plugin Locally

```bash
# Run in development IDE
./gradlew runIde

# Or manually install in IntelliJ:
# Settings → Plugins →  → Install Plugin from Disk
# Select build/distributions/bazbom-intellij-plugin-1.0.0.zip
```

### Step 7: Generate JetBrains Token

1. Go to https://plugins.jetbrains.com/author/me/tokens
2. Click "+ Generate New Token"
3. Configure:
   - **Token Name:** `BazBOM Plugin Publishing`
   - **Scope:** Plugin Upload
   - **Expiration:** 1 year
4. Click "Generate"
5. Copy token
6. Store securely:
   ```bash
   export JETBRAINS_TOKEN="your-token-here"
   ```

### Step 8: Publish to Marketplace

**Option A: Using Gradle (Recommended)**

```bash
# Set token
export JETBRAINS_TOKEN="perm:..."

# Publish
./gradlew publishPlugin

# Publish to beta channel
./gradlew publishPlugin --channel=beta
```

**Option B: Manual Upload**

1. Go to https://plugins.jetbrains.com/
2. Click "Upload plugin"
3. Sign in with JetBrains Account
4. Upload `.zip` file from `build/distributions/`
5. Fill in plugin details:
   - **Plugin Name:** BazBOM Security Scanner
   - **Category:** Code tools
   - **License:** MIT
   - **Tags:** security, vulnerability, sbom, maven, gradle, bazel
6. Add screenshots (at least 2)
7. Add description (copied from plugin.xml)
8. Submit for review

### Step 9: Await Approval

- JetBrains reviews all new plugins (1-3 business days)
- You'll receive email notification when approved
- Plugin will appear in marketplace after approval

### Step 10: Verify Publication

1. Go to https://plugins.jetbrains.com/plugin/YOUR-PLUGIN-ID/bazbom-security-scanner
2. Verify:
   - Plugin details correct
   - Screenshots display
   - Description renders
   - Download button works
3. Test installation in IntelliJ:
   - Settings → Plugins → Marketplace
   - Search "BazBOM"
   - Click Install

---

## Marketing Materials

### Screenshots

**VS Code:**
1. Vulnerability detection in pom.xml (inline warning)
2. Problems panel showing multiple vulnerabilities
3. Settings page
4. Command palette showing BazBOM commands

**IntelliJ:**
1. Dependency tree tool window
2. Inline annotation in build.gradle
3. Quick fix action (Alt+Enter menu)
4. Success notification after fix
5. Settings panel

**Screenshot Guidelines:**
- Resolution: 1920x1080 or higher
- Format: PNG
- Clean workspace (close unnecessary panels)
- Light theme preferred (better visibility)
- Highlight key features (use red boxes/arrows sparingly)

### Demo Video (Optional but Recommended)

**Content:**
1. Opening a vulnerable project (5 sec)
2. Showing vulnerability detection (10 sec)
3. Using quick fix (Alt+Enter) (10 sec)
4. Tests running and passing (10 sec)
5. Final success (5 sec)

**Total:** 30-60 seconds

**Tools:**
- macOS: QuickTime Screen Recording
- Windows: Xbox Game Bar (Win+G)
- Linux: OBS Studio
- Editing: iMovie, DaVinci Resolve (free)

**Upload to:**
- YouTube (unlisted or public)
- Add link to README and marketplace listing

### Description Template

```markdown
# BazBOM Security Scanner

Real-time vulnerability scanning for Java projects. Detect security issues in your Maven, Gradle, and Bazel dependencies instantly.

##  Key Features

-  **Real-time Scanning:** Instant feedback as you code
-  **Privacy-First:** 100% local, no data sent to external servers
-  **Fast:** Scans complete in <10 seconds
-  **Smart Fixes:** One-click upgrades with automated testing
-  **Multi-Build Support:** Maven, Gradle, and Bazel

##  Quick Start

1. Install BazBOM CLI: `brew install cboyd0319/tap/bazbom`
2. Sync database: `bazbom db sync`
3. Open a Java project
4. See vulnerabilities highlighted in build files
5. Press Alt+Enter to fix

##  Documentation

- [Getting Started Guide](https://github.com/cboyd0319/BazBOM/blob/main/docs/user-guide/usage.md)
- [Configuration Options](https://github.com/cboyd0319/BazBOM/blob/main/docs/CONFIGURATION.md)
- [FAQ](https://github.com/cboyd0319/BazBOM/blob/main/docs/FAQ.md)

##  Support

- [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
- [Documentation](https://github.com/cboyd0319/BazBOM/tree/main/docs)

##  License

MIT License - Free for personal and commercial use
```

---

## Post-Publishing

### Initial Launch Checklist

- [ ] Plugin published and live on marketplace
- [ ] Installation tested from marketplace
- [ ] All features working in fresh install
- [ ] Documentation links all working
- [ ] GitHub README updated with installation links
- [ ] CHANGELOG.md updated

### Announcement Plan

**GitHub:**
1. Create release: https://github.com/cboyd0319/BazBOM/releases/new
   - Tag: `v1.0.0`
   - Title: "BazBOM 1.0 - IDE Plugins Released"
   - Description: Features, installation links, screenshots
   - Attach extension/plugin files

2. Post in Discussions:
   - Share release notes
   - Ask for feedback
   - Invite beta testers

**Social Media:**
- Twitter/X: Short announcement with screenshot
- LinkedIn: Longer post with use case
- Reddit: r/java, r/Bazel, r/Kotlin (check self-promotion rules)
- Hacker News: Show HN post (if significant interest)

**Community:**
- Bazel Slack (#general, #java)
- Maven Users mailing list
- Gradle Community Forum

**Sample Tweet:**
```
 Just launched BazBOM IDE plugins for VS Code & IntelliJ!

 Real-time vulnerability scanning
 <10 second scans
 100% local & private
 One-click fixes

Maven • Gradle • Bazel support

VS Code: [link]
IntelliJ: [link]

#security #java #bazel
```

### Monitoring

**First 24 Hours:**
- Check for installation errors
- Monitor GitHub issues
- Respond to marketplace reviews
- Track download numbers

**First Week:**
- Gather user feedback
- Fix critical bugs immediately
- Plan first patch release if needed
- Document common questions

**First Month:**
- Analyze usage patterns
- Plan feature improvements
- Collect feature requests
- Prepare 1.1.0 roadmap

---

## Maintenance and Updates

### Versioning Strategy

Follow semantic versioning (semver):

- **Patch (1.0.1):** Bug fixes, no new features
- **Minor (1.1.0):** New features, backward compatible
- **Major (2.0.0):** Breaking changes

### Release Process

1. **Update Version:**
   ```json
   // package.json (VS Code)
   "version": "1.0.1"
   ```
   ```kotlin
   // build.gradle.kts (IntelliJ)
   version = "1.0.1"
   ```

2. **Update CHANGELOG.md:**
   ```markdown
   ## [1.0.1] - 2025-11-12
   
   ### Fixed
   - Fixed crash on Windows when scanning large projects
   - Improved error messages for missing BazBOM CLI
   ```

3. **Commit and Tag:**
   ```bash
   git add .
   git commit -m "chore: bump version to 1.0.1"
   git tag v1.0.1
   git push origin main --tags
   ```

4. **Rebuild and Publish:**
   ```bash
   # VS Code
   cd crates/bazbom-vscode-extension
   npx vsce publish patch
   
   # IntelliJ
   cd crates/bazbom-intellij-plugin
   ./gradlew publishPlugin
   ```

5. **Create GitHub Release:**
   - Go to Releases → Draft new release
   - Select tag v1.0.1
   - Copy CHANGELOG entry
   - Publish

### Handling Bug Reports

**Triage Priority:**
- **P0 (Critical):** Crashes, data loss → Fix within 24 hours
- **P1 (High):** Major feature broken → Fix within 1 week
- **P2 (Medium):** Minor issues → Fix in next minor release
- **P3 (Low):** Enhancements → Backlog

**Response Template:**
```markdown
Thanks for reporting! I can reproduce this issue.

**Workaround:** [if available]

**Fix:** Will be included in version 1.0.1 (releasing this week)

In the meantime, you can [temporary solution]
```

### Deprecation Policy

When deprecating features:

1. **Announce early:** At least one major version in advance
2. **Provide migration path:** Clear instructions
3. **Keep docs updated:** Mark as deprecated
4. **Remove in next major:** Only in major version bumps

---

## Success Metrics

### Initial Targets

**VS Code (First Month):**
- 1,000+ installs
- 4.0+ star rating
- <5 critical bugs
- 80% positive reviews

**IntelliJ (First Month):**
- 500+ downloads
- 4.0+ star rating
- <5 critical bugs
- 80% positive reviews

### Long-Term Goals (6 Months)

- 10,000+ VS Code installs
- 5,000+ IntelliJ downloads
- Featured in marketplace ("Trending", "Popular")
- Community contributions (PRs, translations)
- Integration requests from organizations

---

## Checklist

### Pre-Publishing

- [ ] Plugin builds successfully
- [ ] All tests pass
- [ ] Manual testing complete (see ../../development/testing-guide.md)
- [ ] README.md updated with screenshots
- [ ] CHANGELOG.md created
- [ ] LICENSE file present
- [ ] Icon assets created
- [ ] Publisher accounts created
- [ ] Access tokens generated and stored securely

### Publishing

- [ ] VS Code marketplace account ready
- [ ] JetBrains marketplace account ready
- [ ] package.json configured
- [ ] plugin.xml configured
- [ ] Extension/plugin packaged
- [ ] Local testing completed
- [ ] Published to marketplaces
- [ ] Verified installation from marketplace

### Post-Publishing

- [ ] GitHub release created
- [ ] README updated with installation links
- [ ] Announcement posted (GitHub, social media)
- [ ] Monitoring set up (issues, reviews)
- [ ] Support channels ready
- [ ] First-week feedback collected

---

## Troubleshooting

### Common Issues

**Issue: "Publisher not found" error**

**Solution:**
```bash
# Re-login to vsce
npx vsce logout
npx vsce login your-publisher-id
```

---

**Issue: "Invalid token" error**

**Solution:**
- Check token expiration in Azure DevOps
- Ensure token has "Marketplace (Manage)" scope
- Generate new token if expired

---

**Issue: Plugin rejected by JetBrains**

**Common Reasons:**
- Missing plugin.xml details
- Inadequate description
- No screenshots
- Copyright violations
- Security concerns

**Solution:** Address feedback and resubmit

---

**Issue: Extension not activating in VS Code**

**Debug Steps:**
1. Check activation events in package.json
2. View Output panel → BazBOM Language Server
3. Check for error messages
4. Verify bazbom-lsp binary exists

---

## Resources

**VS Code:**
- [Publishing Extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- [Extension Manifest](https://code.visualstudio.com/api/references/extension-manifest)
- [Marketplace FAQ](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#marketplace-faq)

**JetBrains:**
- [Plugin Publication](https://plugins.jetbrains.com/docs/intellij/publishing-plugin.html)
- [Plugin Deployment](https://plugins.jetbrains.com/docs/intellij/deployment.html)
- [Marketplace Guidelines](https://plugins.jetbrains.com/docs/marketplace/plugin-overview-page.html)

**Tools:**
- [vsce (VS Code Extension Manager)](https://github.com/microsoft/vscode-vsce)
- [IntelliJ Platform Gradle Plugin](https://github.com/JetBrains/gradle-intellij-plugin)

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Maintained By:** BazBOM Development Team  
**Contact:** support@bazbom.io
