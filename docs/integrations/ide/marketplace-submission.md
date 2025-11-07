# IDE Marketplace Submission Checklist

**Last Updated:** 2025-11-05  
**Purpose:** Step-by-step checklist for publishing BazBOM IDE plugins  
**Status:** Ready for execution

---

## Quick Links

- [Marketplace Publishing Guide](marketplace-publishing.md) - Detailed guide
- [IDE Marketplace Assets Guide](marketplace-assets.md) - Asset specifications
- [IDE Plugin Testing Guide](plugin-testing.md) - Testing procedures

---

## Pre-Submission Requirements

### Code Quality 
- [x] All tests passing (500+ tests)
- [x] Zero compilation errors
- [x] Clippy warnings addressed
- [x] Code formatted with rustfmt
- [x] Documentation up-to-date

### IDE Plugin Status

**VS Code Extension:** 95% Complete
- [x] LSP server implementation
- [x] TypeScript code compiles
- [x] Extension commands registered
- [x] Settings configured
- [ ] Marketplace assets created
- [ ] Local installation tested
- [ ] Published to marketplace

**IntelliJ Plugin:** 95% Complete
- [x] Kotlin code compiles
- [x] Gradle build successful
- [x] All features implemented
- [x] Settings panel complete
- [ ] Marketplace assets created
- [ ] Local installation tested
- [ ] Published to marketplace

---

## VS Code Marketplace Checklist

### Asset Preparation

- [ ] **Icon Created**
  - File: `crates/bazbom-vscode-extension/assets/icon.png`
  - Size: 128x128 pixels minimum (256x256 recommended)
  - Format: PNG with transparency
  - Tool: GIMP, Figma, or Canva

- [ ] **Screenshots Captured** (3-5 required)
  - [ ] Screenshot 1: Real-time scanning in action
  - [ ] Screenshot 2: Quick fix menu (Alt+Enter)
  - [ ] Screenshot 3: Settings panel
  - [ ] Screenshot 4: Multiple build systems
  - [ ] Screenshot 5: Command palette
  - Resolution: 1920x1080
  - Format: PNG
  - Location: `crates/bazbom-vscode-extension/assets/`

- [ ] **Demo Created** (Recommended)
  - [ ] GIF or video (30-60 seconds)
  - [ ] Shows key features: scan → detect → fix
  - [ ] Location: `crates/bazbom-vscode-extension/assets/demo.gif`

- [ ] **README Updated**
  - [ ] Screenshots embedded
  - [ ] Installation instructions clear
  - [ ] Feature list complete
  - [ ] Badges added (optional)

- [ ] **CHANGELOG Created**
  - [ ] Version 1.0.0 entry
  - [ ] Features documented
  - [ ] Follows Keep a Changelog format

### Publisher Setup

- [ ] **Azure DevOps Account**
  - Create at https://dev.azure.com/
  - Verify email address

- [ ] **Publisher ID Created**
  - Go to https://marketplace.visualstudio.com/manage
  - Click "Create Publisher"
  - Publisher ID: `cboyd0319` (or preferred)
  - Display Name: `BazBOM Security`

- [ ] **Personal Access Token (PAT)**
  - Generate at https://dev.azure.com/
  - Scope: Marketplace → Manage
  - Expiration: 1 year minimum
  - Store securely (GitHub Secrets)
  - Export: `export VSCE_PAT="..."`

### Package Configuration

- [ ] **package.json Updated**
  - [x] Name: `bazbom`
  - [x] Display name: `BazBOM Security Scanner`
  - [ ] Publisher: `cboyd0319` (update if needed)
  - [ ] Icon path: `assets/icon.png`
  - [x] Repository URL
  - [ ] Homepage URL
  - [ ] Bugs URL
  - [x] Keywords (security, sbom, maven, gradle, bazel)
  - [x] Categories (Linters, Security)

- [ ] **Dependencies Installed**
  ```bash
  cd crates/bazbom-vscode-extension
  npm install
  npm install -g @vscode/vsce
  ```

### Local Testing

- [ ] **Build Extension**
  ```bash
  cd crates/bazbom-vscode-extension
  npm run compile
  ```

- [ ] **Package Extension**
  ```bash
  vsce package --no-yarn
  # Creates: bazbom-1.0.0.vsix
  ```

- [ ] **Install Locally**
  ```bash
  code --install-extension bazbom-1.0.0.vsix
  ```

- [ ] **Test All Features**
  - [ ] Extension activates on Java project
  - [ ] Real-time scanning works (save file)
  - [ ] Diagnostics appear correctly
  - [ ] Commands work (scan, sync)
  - [ ] Settings are functional
  - [ ] No console errors

- [ ] **Uninstall and Reinstall**
  - Verify clean installation
  - Check all features again

### Marketplace Publishing

- [ ] **Login to VSCE**
  ```bash
  vsce login cboyd0319
  # Enter PAT when prompted
  ```

- [ ] **Publish Extension**
  ```bash
  vsce publish
  # Or with version bump:
  # vsce publish patch  # 1.0.0 → 1.0.1
  # vsce publish minor  # 1.0.0 → 1.1.0
  # vsce publish major  # 1.0.0 → 2.0.0
  ```

- [ ] **Verify Listing**
  - Go to https://marketplace.visualstudio.com/items?itemName=cboyd0319.bazbom
  - Check icon displays correctly
  - Verify screenshots appear
  - Test "Install" button

### Post-Publishing

- [ ] **Announcement**
  - [ ] GitHub Release created
  - [ ] Twitter/X post
  - [ ] Reddit: r/vscode, r/programming, r/java
  - [ ] Hacker News (Show HN)
  - [ ] Dev.to article

- [ ] **Monitor**
  - [ ] Check ratings (target: 4.5+ stars)
  - [ ] Respond to reviews
  - [ ] Track install count (target: 1000+ in month 1)
  - [ ] Monitor issues

- [ ] **Update Documentation**
  - [ ] Add marketplace badge to README.md
  - [ ] Update installation instructions
  - [ ] Link from main project README

---

## IntelliJ Marketplace Checklist

### Asset Preparation

- [ ] **Icon Created**
  - File: `crates/bazbom-intellij-plugin/src/main/resources/META-INF/pluginIcon.svg`
  - Size: 40x40 (standard), 80x80 (retina)
  - Format: SVG (preferred) or PNG
  - Optional: `pluginIcon_dark.svg` for dark theme

- [ ] **Screenshots Captured** (3-5 required)
  - [ ] Screenshot 1: Tool window with dependency tree
  - [ ] Screenshot 2: Inline annotations in pom.xml
  - [ ] Screenshot 3: Quick fix dialog (Alt+Enter)
  - [ ] Screenshot 4: Settings panel
  - [ ] Screenshot 5: Notification system
  - Resolution: 1920x1080
  - Format: PNG
  - Location: `crates/bazbom-intellij-plugin/screenshots/`

- [ ] **Demo Created** (Recommended)
  - [ ] GIF or video (30-60 seconds)
  - [ ] Shows: tool window → annotations → quick fix
  - [ ] Location: `crates/bazbom-intellij-plugin/screenshots/demo.gif`

- [ ] **plugin.xml Updated**
  - [ ] Description with HTML formatting
  - [ ] Change notes for v1.0.0
  - [ ] Vendor information
  - [ ] Plugin homepage URL

- [ ] **CHANGELOG Created**
  - [ ] Version 1.0.0 entry
  - [ ] Features documented

### Publisher Setup

- [ ] **JetBrains Account**
  - Create at https://account.jetbrains.com/
  - Verify email

- [ ] **Plugin Repository Access**
  - Go to https://plugins.jetbrains.com/
  - Sign in with JetBrains account
  - Accept developer agreement

- [ ] **Token Generated**
  - Go to https://plugins.jetbrains.com/author/me/tokens
  - Create new token: "BazBOM Publishing"
  - Store securely
  - Export: `export JB_TOKEN="..."`

### Plugin Configuration

- [x] **build.gradle.kts Configured**
  - [x] Plugin name
  - [x] Plugin version
  - [x] Vendor details
  - [x] IntelliJ version compatibility (2023.3+)
  - [x] Dependencies declared

- [x] **plugin.xml Configured**
  - [x] Plugin ID: `io.bazbom.intellij-plugin`
  - [x] Plugin name
  - [x] Vendor
  - [x] Description
  - [x] Dependencies
  - [x] Extensions registered
  - [x] Actions registered

### Local Testing

- [ ] **Build Plugin**
  ```bash
  cd crates/bazbom-intellij-plugin
  ./gradlew buildPlugin
  # Creates: build/distributions/bazbom-intellij-plugin-1.0.0.zip
  ```

- [ ] **Test in IDE**
  ```bash
  ./gradlew runIde
  # Opens test IntelliJ instance with plugin
  ```

- [ ] **Test All Features**
  - [ ] Plugin loads without errors
  - [ ] Tool window appears
  - [ ] Dependency tree displays
  - [ ] Real-time annotations work
  - [ ] Quick fixes work (Alt+Enter)
  - [ ] Settings panel functional
  - [ ] Auto-scan on project open
  - [ ] Notifications appear correctly

- [ ] **Compatibility Test**
  - [ ] IntelliJ IDEA Community 2023.3
  - [ ] IntelliJ IDEA Ultimate 2023.3
  - [ ] Android Studio (latest)

### Marketplace Publishing

- [ ] **Publish via Gradle**
  ```bash
  cd crates/bazbom-intellij-plugin
  
  # Configure token
  echo "intellijPublishToken=$JB_TOKEN" >> gradle.properties
  
  # Publish
  ./gradlew publishPlugin
  ```

- [ ] **Or Upload Manually**
  1. Go to https://plugins.jetbrains.com/plugin/add
  2. Upload ZIP: `build/distributions/bazbom-intellij-plugin-1.0.0.zip`
  3. Fill in details
  4. Submit for review

- [ ] **Await Approval**
  - Review takes 1-3 business days
  - JetBrains team reviews for:
    - Code quality
    - Performance
    - Compatibility
    - Security
  - Respond to feedback if requested

### Post-Publishing

- [ ] **Announcement**
  - [ ] GitHub Release
  - [ ] Twitter/X
  - [ ] Reddit: r/IntelliJIDEA, r/java, r/kotlin
  - [ ] JetBrains Community
  - [ ] LinkedIn

- [ ] **Monitor**
  - [ ] Check downloads (target: 500+ in month 1)
  - [ ] Respond to reviews
  - [ ] Track compatibility reports
  - [ ] Monitor issues

- [ ] **Update Documentation**
  - [ ] Add marketplace badge
  - [ ] Update installation instructions
  - [ ] Link from main README

---

## Success Metrics

### VS Code Extension

**Week 1:**
- [ ] 100+ installs
- [ ] 4.0+ star rating
- [ ] Zero critical bugs reported

**Month 1:**
- [ ] 1,000+ installs
- [ ] 4.5+ star rating
- [ ] 10+ positive reviews

**Month 3:**
- [ ] 5,000+ installs
- [ ] Featured in marketplace (if possible)
- [ ] Community contributions

### IntelliJ Plugin

**Week 1:**
- [ ] 50+ downloads
- [ ] 4.0+ star rating
- [ ] Zero critical bugs

**Month 1:**
- [ ] 500+ downloads
- [ ] 4.5+ star rating
- [ ] 5+ positive reviews

**Month 3:**
- [ ] 2,000+ downloads
- [ ] Positive community feedback
- [ ] Compatible with latest IntelliJ versions

---

## Common Issues and Solutions

### VS Code

**Issue:** "No publisher ID found"
- **Solution:** Run `vsce publish` with `--publisher cboyd0319` flag

**Issue:** "Missing icon"
- **Solution:** Add `"icon": "assets/icon.png"` to package.json

**Issue:** "Package too large"
- **Solution:** Add unnecessary files to `.vscodeignore`

### IntelliJ

**Issue:** "Plugin verification failed"
- **Solution:** Run `./gradlew verifyPlugin` and fix reported issues

**Issue:** "Incompatible with target IDE build"
- **Solution:** Update `sinceBuild` and `untilBuild` in plugin.xml

**Issue:** "Plugin causes freezes"
- **Solution:** Ensure all operations run in background threads

---

## Timeline Estimate

### VS Code Extension

- **Asset Creation:** 4-8 hours
  - Icon design: 1-2 hours
  - Screenshots: 2-3 hours
  - Demo video: 1-2 hours
  - Documentation: 1 hour

- **Publisher Setup:** 30 minutes
  - Account creation: 10 minutes
  - PAT generation: 10 minutes
  - Testing: 10 minutes

- **Testing & Publishing:** 2-3 hours
  - Local testing: 1-2 hours
  - Publishing: 30 minutes
  - Verification: 30 minutes

**Total: 7-12 hours**

### IntelliJ Plugin

- **Asset Creation:** 4-8 hours
  - Icon design: 1-2 hours
  - Screenshots: 2-3 hours
  - Demo video: 1-2 hours
  - Documentation: 1 hour

- **Publisher Setup:** 30 minutes

- **Testing & Publishing:** 3-4 hours
  - Local testing: 2-3 hours
  - Publishing: 30 minutes
  - Review wait: 1-3 business days

**Total: 8-13 hours + approval time**

---

## Next Actions

**Immediate (This Week):**
1. Create icon assets for both plugins
2. Capture screenshots in both IDEs
3. Create demo GIFs (5-10 seconds each)
4. Update package.json and plugin.xml with asset paths

**Short Term (Next Week):**
1. Set up publisher accounts (VS Code + JetBrains)
2. Test local installations thoroughly
3. Create announcement content
4. Publish to both marketplaces

**Ongoing:**
1. Monitor ratings and reviews daily
2. Respond to issues within 24 hours
3. Plan for v1.1.0 with community feedback
4. Update documentation based on user questions

---

## Resources

- **VS Code Publishing:** https://code.visualstudio.com/api/working-with-extensions/publishing-extension
- **JetBrains Publishing:** https://plugins.jetbrains.com/docs/intellij/publishing-plugin.html
- **Asset Guide:** [marketplace-assets.md](marketplace-assets.md)
- **Testing Guide:** [IDE Plugin Testing Guide](plugin-testing.md)
- **Capabilities Reference:** [../../reference/capabilities-reference.md](../../reference/capabilities-reference.md)

---

**Status:** Ready for execution - assets and publishing process documented  
**Owner:** @cboyd0319  
**Last Updated:** 2025-11-05
