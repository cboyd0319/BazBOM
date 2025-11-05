# IDE Marketplace Assets Guide

**Last Updated:** 2025-11-05  
**Status:** Active  
**Purpose:** Asset preparation for VS Code and JetBrains marketplace publishing

---

## Overview

This document provides detailed specifications for creating and preparing marketing assets for BazBOM IDE plugin marketplace listings.

### Required Assets

| Asset Type | VS Code | JetBrains | Purpose |
|------------|---------|-----------|---------|
| Icon/Logo | ✅ Required | ✅ Required | Plugin identification |
| Screenshots | ✅ Required | ✅ Required | Feature demonstration |
| Demo Video/GIF | ⭐ Recommended | ⭐ Recommended | Quick feature overview |
| Banner | ⭐ Optional | ❌ Not used | Marketplace header |
| README | ✅ Required | ✅ Required | Plugin documentation |
| CHANGELOG | ✅ Required | ✅ Required | Version history |

---

## Icon/Logo Requirements

### VS Code Extension Icon

**Technical Specifications:**
- **Size:** 128x128 pixels (minimum), 256x256 recommended
- **Format:** PNG (24-bit with alpha channel)
- **Background:** Transparent or solid color matching theme
- **Location:** `crates/bazbom-vscode-extension/assets/icon.png`
- **File Size:** < 1MB

**Design Guidelines:**
- Use simple, recognizable imagery
- Ensure readability at small sizes (48x48)
- Test on both light and dark themes
- Avoid text (icon should be self-explanatory)
- Use brand colors if applicable

**Suggested Design Concepts:**
1. **Security Shield** with dependency graph nodes
2. **Padlock** with JVM coffee cup
3. **Dependency Tree** with security checkmarks
4. **Radar/Scanner** icon with code brackets

**Color Palette:**
```css
Primary:   #2D7DD2 (Blue - Security)
Secondary: #E94D35 (Red - Vulnerabilities)
Accent:    #23A559 (Green - Safe)
Dark:      #1E1E1E (VS Code Dark Theme)
Light:     #FFFFFF (VS Code Light Theme)
```

### IntelliJ Plugin Icon

**Technical Specifications:**
- **Size:** 40x40 pixels (standard), 80x80 (2x retina), 16x16 (small)
- **Format:** SVG (preferred) or PNG
- **Location:** `crates/bazbom-intellij-plugin/src/main/resources/META-INF/pluginIcon.svg`
- **Variants:**
  - `pluginIcon.svg` - Default
  - `pluginIcon_dark.svg` - Dark theme (optional)

**Design Guidelines:**
- Follow JetBrains plugin icon style guide
- Use simple geometric shapes
- 2-3 colors maximum
- Must work at 16x16 size
- Test with both light (IntelliJ Light) and dark (Darcula) themes

---

## Screenshots

### General Guidelines

**Technical Requirements:**
- **Format:** PNG (24-bit color)
- **Resolution:** 1920x1080 (Full HD) or higher
- **DPI:** 72-144 DPI
- **File Size:** < 5MB per image
- **Count:** 3-5 screenshots minimum

**Screenshot Themes:**
- Use VS Code Dark+ or IntelliJ Darcula (most popular)
- Ensure high contrast and readability
- Use realistic project examples
- Hide personal information (file paths, usernames)

### VS Code Screenshots

**Required Shots:**

1. **Real-Time Scanning** (Main Feature)
   - Show `pom.xml` or `build.gradle` open
   - Display inline diagnostics (red squiggles)
   - Hover tooltip showing vulnerability details
   - Problems panel at bottom with vulnerability list

2. **Quick Fix Actions**
   - Show light bulb code action menu
   - "Upgrade to safe version" option highlighted
   - Before/after version numbers visible

3. **Settings Panel**
   - VS Code Settings UI open
   - BazBOM settings section visible
   - Show configuration options

4. **Multiple Build Systems**
   - Split view: Maven + Gradle + Bazel files
   - All showing vulnerabilities detected
   - Demonstrates cross-build-system support

5. **Command Palette**
   - Show "BazBOM: Scan Project" command
   - Extension commands listed

**Composition Tips:**
- Use **1920x1080** resolution
- **Center** important content
- Add **arrows or callouts** to highlight features (use image editor)
- **Zoom in** on specific areas for detail shots
- Use **consistent window size** across screenshots

### IntelliJ Screenshots

**Required Shots:**

1. **Tool Window** (Main Feature)
   - BazBOM tool window on right side
   - Dependency tree visible
   - Vulnerabilities highlighted in red
   - Summary statistics at top

2. **Inline Annotations**
   - `pom.xml` or `build.gradle.kts` open
   - Red/yellow highlights on vulnerable dependencies
   - Hover tooltip showing CVE details

3. **Quick Fix Dialog**
   - Alt+Enter menu open
   - "Upgrade dependency" action shown
   - Version numbers visible

4. **Settings Dialog**
   - IntelliJ Settings open
   - BazBOM settings panel visible
   - Options configured

5. **Notification System**
   - Notification balloon in bottom-right
   - "Vulnerabilities detected" or "Scan complete" message
   - Action buttons visible

**Composition Tips:**
- Use **IntelliJ IDEA 2023.3+** for screenshots
- Show **realistic project structure** in left sidebar
- **Highlight BazBOM UI elements** clearly
- Use **readable font sizes** (Editor font: 14-16pt)
- Avoid **cluttered code** - use simple examples

---

## Demo Video/GIF

### Video Specifications

**Technical Requirements:**
- **Duration:** 30-60 seconds
- **Resolution:** 1920x1080 (1080p)
- **Format:** MP4 (H.264 codec)
- **Frame Rate:** 30 or 60 fps
- **File Size:** < 50MB
- **Hosting:** YouTube or GitHub assets

**Content Structure:**

**[0-5s] Opening**
- Show BazBOM logo or title
- Text: "BazBOM Security Scanner for VS Code/IntelliJ"

**[5-15s] Problem**
- Open vulnerable project
- Show outdated dependency in build file
- Highlight security concern

**[15-35s] Solution**
- BazBOM automatically scans
- Vulnerabilities appear inline
- Quick fix action applied
- Version updated to safe version

**[35-50s] Benefits**
- Show fast scan time (<10s)
- Multiple build systems supported
- Privacy-preserving (local scanning)

**[50-60s] Call to Action**
- "Install BazBOM today"
- Link to marketplace
- GitHub star button

### GIF Specifications

**For Quick Feature Demos:**
- **Duration:** 5-10 seconds per feature
- **Resolution:** 1280x720 (720p) or smaller
- **Format:** Animated GIF or WebM
- **Frame Rate:** 15-30 fps
- **File Size:** < 5MB
- **Loop:** Yes (seamless)

**Recommended GIFs:**
1. **Inline scanning**: Save file → vulnerability appears
2. **Quick fix**: Alt+Enter → upgrade → version updated
3. **Tool window**: Expand tree → show vulnerabilities
4. **Command palette**: Run scan → results appear

**Tools for Creating GIFs:**
- **ScreenToGif** (Windows) - Free, open source
- **LICEcap** (macOS/Windows) - Free, lightweight
- **Kap** (macOS) - Free, beautiful output
- **Peek** (Linux) - Simple screen recorder

---

## Gallery Banner (VS Code Only)

### Specifications

**Technical Requirements:**
- **Size:** 960x640 pixels (exact)
- **Format:** PNG or JPEG
- **File Size:** < 500KB
- **Purpose:** Marketplace page header background

**Design Guidelines:**
- Use solid color or subtle gradient
- Avoid busy patterns (text must be readable)
- Brand colors recommended
- Test with white text overlay

**Configuration in package.json:**
```json
{
  "galleryBanner": {
    "color": "#2D7DD2",
    "theme": "dark"
  }
}
```

**Theme Options:**
- `"dark"` - Use with dark background colors (#000000-#444444)
- `"light"` - Use with light background colors (#BBBBBB-#FFFFFF)

---

## README Best Practices

### Structure

**Essential Sections:**
1. **Overview** - One-sentence description
2. **Features** - Bullet list of capabilities
3. **Requirements** - Dependencies and setup
4. **Installation** - Step-by-step
5. **Usage** - Quick start guide
6. **Configuration** - Settings documentation
7. **Troubleshooting** - Common issues
8. **License** - MIT with link
9. **Support** - Issues link

### Formatting

**Use Markdown:**
- Headers (##, ###) for sections
- Code blocks with syntax highlighting
- Screenshots/GIFs inline
- Tables for comparisons
- Badges for visual appeal (optional)

**Example Badges:**
```markdown
[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/cboyd0319.bazbom)](...)
[![Rating](https://img.shields.io/visual-studio-marketplace/r/cboyd0319.bazbom)](...)
[![Downloads](https://img.shields.io/visual-studio-marketplace/d/cboyd0319.bazbom)](...)
```

### Screenshots in README

**Embed screenshots:**
```markdown
![Feature Name](assets/screenshot-feature.png)

**Dependency Scanning**

![Scanning in action](assets/demo-scan.gif)
```

**Tips:**
- Use **relative paths** for images
- Add **alt text** for accessibility
- Keep images under 2MB each
- Use **descriptive filenames**

---

## CHANGELOG Format

### Structure

Follow **Keep a Changelog** format:

```markdown
# Changelog

All notable changes to the BazBOM extension will be documented in this file.

## [Unreleased]

### Added
- Feature X

### Changed
- Improved Y

### Fixed
- Bug Z

## [1.0.0] - 2025-11-05

### Added
- Real-time vulnerability scanning
- Support for Maven, Gradle, and Bazel
- Quick fix actions for upgrades
- Settings panel configuration

### Security
- All scanning happens locally (privacy-preserving)
```

### Versioning

Follow **Semantic Versioning (semver):**
- **MAJOR** (1.x.x): Breaking changes
- **MINOR** (x.1.x): New features (backward compatible)
- **PATCH** (x.x.1): Bug fixes

---

## Marketplace Descriptions

### VS Code Marketplace

**Short Description (80 chars max):**
```
Real-time vulnerability scanner for Java projects (Maven, Gradle, Bazel)
```

**Long Description:**
```markdown
## BazBOM Security Scanner

**World-class SBOM and vulnerability scanning for JVM projects**, now available directly in VS Code!

### ✨ Features

- **Real-Time Scanning**: Instant feedback as you edit build files
- **Privacy-First**: All scanning happens locally, no data leaves your machine
- **Multi-Build-System**: Maven, Gradle, and Bazel support
- **Fast**: <10 second scans with smart caching
- **Actionable**: Quick fix actions to upgrade vulnerable dependencies

### 🚀 Quick Start

1. Install extension
2. Open Java project
3. Edit `pom.xml` or `build.gradle`
4. Vulnerabilities appear automatically!

### 🔒 Security & Privacy

- Zero telemetry
- Offline-first operation
- MIT licensed and open source
```

### JetBrains Marketplace

**Short Description (240 chars max):**
```
BazBOM brings world-class vulnerability scanning to IntelliJ IDEA. Real-time detection for Maven, Gradle, and Bazel projects. Privacy-preserving, open source, and lightning fast.
```

**Long Description:**
```markdown
# BazBOM Security Scanner for IntelliJ IDEA

World-class SBOM and vulnerability scanning for JVM projects, integrated directly into your IDE.

## Features

### Real-Time Vulnerability Detection
Automatically scans pom.xml, build.gradle, and BUILD.bazel files as you edit. Vulnerabilities appear inline with actionable quick fixes.

### Multi-Build-System Support
Works seamlessly with:
- Maven (pom.xml)
- Gradle (build.gradle, build.gradle.kts)
- Bazel (BUILD, MODULE.bazel)

### Privacy-Preserving
All scanning happens locally on your machine. No data is sent to external servers. No telemetry or tracking.

### Dependency Tree Visualization
Browse your entire dependency graph in the BazBOM tool window. Color-coded by vulnerability severity.

### One-Click Remediation
Alt+Enter on vulnerable dependencies to upgrade to safe versions automatically.

## Requirements

- IntelliJ IDEA 2023.3 or higher
- BazBOM CLI installed (https://github.com/cboyd0319/BazBOM)

## Getting Started

1. Install plugin
2. Open Java/Kotlin/Scala project
3. Plugin auto-scans on project open
4. Review vulnerabilities in BazBOM tool window
5. Alt+Enter to apply fixes
```

---

## Asset Checklist

### Before Publishing

**VS Code:**
- [ ] Icon PNG (128x128+) created
- [ ] 3-5 screenshots captured (1920x1080)
- [ ] Demo GIF or video created (optional)
- [ ] README.md updated with screenshots
- [ ] CHANGELOG.md created with v1.0.0 entry
- [ ] package.json: publisher, repository, icon, keywords updated
- [ ] All images under 5MB each
- [ ] Test installation locally: `vsce package`

**IntelliJ:**
- [ ] Icon SVG/PNG created (40x40, 80x80)
- [ ] 3-5 screenshots captured (1920x1080)
- [ ] Demo GIF or video created (optional)
- [ ] README.md in plugin.xml description
- [ ] CHANGELOG.md in plugin.xml change-notes
- [ ] plugin.xml: name, vendor, description updated
- [ ] All images optimized
- [ ] Test build: `./gradlew buildPlugin`

---

## Asset Storage

### Directory Structure

```
crates/bazbom-vscode-extension/
├── assets/
│   ├── icon.png                    # Extension icon
│   ├── screenshot-scanning.png     # Feature screenshot 1
│   ├── screenshot-quickfix.png     # Feature screenshot 2
│   ├── screenshot-settings.png     # Feature screenshot 3
│   ├── screenshot-multibs.png      # Feature screenshot 4
│   ├── demo-scan.gif               # Demo GIF
│   └── banner.png                  # Gallery banner (optional)
├── README.md                       # Marketplace description
└── CHANGELOG.md                    # Version history

crates/bazbom-intellij-plugin/
├── src/main/resources/META-INF/
│   ├── pluginIcon.svg              # Plugin icon
│   └── pluginIcon_dark.svg         # Dark theme icon (optional)
├── screenshots/
│   ├── toolwindow.png              # Feature screenshot 1
│   ├── annotations.png             # Feature screenshot 2
│   ├── quickfix.png                # Feature screenshot 3
│   ├── settings.png                # Feature screenshot 4
│   └── demo.gif                    # Demo GIF
├── README.md                       # Plugin description
└── CHANGELOG.md                    # Version history
```

---

## Tools and Resources

### Image Editing

**Free Tools:**
- **GIMP** - https://www.gimp.org/ (Photoshop alternative)
- **Inkscape** - https://inkscape.org/ (Vector graphics)
- **Paint.NET** - https://www.getpaint.net/ (Windows)
- **Krita** - https://krita.org/ (Digital painting)

**Online Tools:**
- **Canva** - https://www.canva.com/ (Templates)
- **Figma** - https://www.figma.com/ (Design collaboration)
- **Photopea** - https://www.photopea.com/ (Browser-based Photoshop)

### Screen Recording

**Desktop:**
- **OBS Studio** - https://obsproject.com/ (Professional recording)
- **ShareX** - https://getsharex.com/ (Windows, GIF support)
- **Kap** - https://getkap.co/ (macOS, beautiful UI)

**Browser-Based:**
- **Recordit** - https://recordit.co/ (Quick GIFs)
- **CloudApp** - https://www.getcloudapp.com/ (Screen capture + hosting)

### Video Editing

**Free:**
- **DaVinci Resolve** - https://www.blackmagicdesign.com/products/davinciresolve (Professional)
- **OpenShot** - https://www.openshot.org/ (Simple editor)
- **Kdenlive** - https://kdenlive.org/ (Cross-platform)

### Image Optimization

**CLI Tools:**
```bash
# Install ImageMagick
brew install imagemagick

# Resize to 1920x1080
convert input.png -resize 1920x1080 output.png

# Optimize PNG
optipng -o7 output.png

# Convert to WebP (smaller file size)
cwebp -q 80 input.png -o output.webp
```

---

## Testing Assets

### Validation Checklist

**Visual Quality:**
- [ ] Images sharp and clear (no pixelation)
- [ ] Text readable at 100% zoom
- [ ] Colors consistent across assets
- [ ] No artifacts or compression issues

**Technical Quality:**
- [ ] File sizes within limits
- [ ] Correct dimensions for each asset type
- [ ] Proper file formats (PNG, SVG, MP4)
- [ ] Alpha channels preserved (transparency)

**Content Quality:**
- [ ] Realistic project examples used
- [ ] No personal information visible
- [ ] Screenshots show actual functionality
- [ ] Consistent UI theme (light/dark)

**Marketplace Preview:**
- [ ] Test how assets appear in marketplace listing
- [ ] Check mobile view (responsive)
- [ ] Verify links and badges work
- [ ] Proofread all text content

---

## Example Asset Workflow

### Creating Screenshots (VS Code)

1. **Setup Environment**
   ```bash
   # Start VS Code
   code .
   
   # Set window size
   # View → Appearance → Zoom In (if needed)
   ```

2. **Prepare Demo Project**
   - Create or use example Maven project
   - Add intentionally vulnerable dependencies
   - Open `pom.xml`

3. **Capture Screenshots**
   - Use OS screenshot tool:
     - macOS: Cmd+Shift+4 (select area)
     - Windows: Win+Shift+S
     - Linux: Flameshot or GNOME Screenshot
   
4. **Edit Screenshots**
   - Crop to 1920x1080
   - Add annotations if needed
   - Optimize file size
   - Save as PNG

5. **Test in Extension**
   ```bash
   cd crates/bazbom-vscode-extension
   
   # Update package.json with icon path
   # Test package build
   vsce package --no-yarn
   
   # Install locally
   code --install-extension bazbom-1.0.0.vsix
   ```

---

## Next Steps

1. **Create Assets**: Use tools above to create required assets
2. **Review Quality**: Check all assets meet specifications
3. **Update Package Files**: Add asset paths to package.json/plugin.xml
4. **Test Locally**: Package and install locally to verify
5. **Publish**: Follow [MARKETPLACE_PUBLISHING_GUIDE.md](MARKETPLACE_PUBLISHING_GUIDE.md)
6. **Monitor**: Track downloads, ratings, and user feedback

---

## Support

- **Questions?** Open an issue: https://github.com/cboyd0319/BazBOM/issues
- **Design Help?** Tag `@cboyd0319` in discussions
- **Asset Review?** Submit draft PR with assets for feedback

---

**Last Updated:** 2025-11-05  
**Maintainer:** @cboyd0319  
**Status:** Active Development
