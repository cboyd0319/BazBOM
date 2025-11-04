# BazBOM Implementation Roadmap
## Making BazBOM the Ultimate Easy-to-Use SBOM, SCA, and Dependency Graph Solution

**Document Version:** 1.0  
**Last Updated:** 2025-11-03  
**Status:** Active Development Plan  
**Horizon:** 8 Weeks  

> **📋 Related Planning Documents:**
> - **[Master Roadmap (../ROADMAP.md)](../ROADMAP.md)** - Complete feature tracking checklist for all phases
> - **[Strategic Roadmap (STRATEGIC_ROADMAP.md)](STRATEGIC_ROADMAP.md)** - 12-18 month market leadership vision
> - **[Phase 4 Progress (PHASE_4_PROGRESS.md)](PHASE_4_PROGRESS.md)** - Current IDE integration status
>
> This document provides detailed specifications for the 8-week UX improvement sprint.

---

## Executive Summary

This roadmap outlines the path to making BazBOM the most developer-friendly, easy-to-use SBOM, SCA, and dependency graph solution for ALL Maven, Gradle, and Bazel projects. The focus is on **developer experience**, **visual excellence**, and **team collaboration** to complement BazBOM's existing technical strengths in accuracy, security, and privacy.

### Strategic Objectives

1. **Eliminate Friction**: Make getting started with BazBOM effortless
2. **Visual Excellence**: Provide compelling visualizations for technical and non-technical audiences
3. **Team Coordination**: Enable security teams to work together efficiently
4. **One-Click Everything**: Automate repetitive tasks completely

### Success Criteria

- ✅ New users can scan a project in <60 seconds from first install
- ✅ Non-technical stakeholders understand security posture instantly
- ✅ Development teams adopt BazBOM voluntarily (not mandated)
- ✅ 95% of vulnerability remediation requires <5 minutes
- ✅ Security teams coordinate without external tools

---

## Timeline Overview

| Phase | Duration | Focus Area | Priority |
|-------|----------|------------|----------|
| **Phase 1: Quick Wins** | Weeks 1-2 | Developer onboarding & UX | 🔴 P0 |
| **Phase 2: Visual Excellence** | Weeks 3-4 | Web dashboard & reports | 🔴 P0 |
| **Phase 3: IDE Polish** | Weeks 5-6 | IDE integration maturity | 🟡 P1 |
| **Phase 4: Team Features** | Weeks 7-8 | Collaboration & coordination | 🟡 P1 |

---

## Phase 1: Quick Wins (Weeks 1-2)

### Objective
Remove all friction from getting started with BazBOM. Make the first scan experience delightful.

### 1.1 Interactive `bazbom init` Command

**Problem:** New users don't know how to configure BazBOM for their project.

**Solution:** Guided interactive setup that detects build system, creates policy, and runs first scan.

#### Features

```bash
$ bazbom init
🎉 Welcome to BazBOM! Let's get your project secured.

🔍 Detecting build system...
✅ Found: Maven project (pom.xml)

�� Choose a policy template:
  1. Development (Permissive)
  2. Corporate Standard
  3. PCI-DSS Compliance
  4. HIPAA Healthcare
  5. FedRAMP Moderate
  6. SOC 2 Type II
  7. Custom (manual configuration)

Your choice [1-7]: 2

✅ Created bazbom.yml with Corporate Standard policy

🔍 Running first scan...
⏳ Scanning dependencies... (this may take a minute)
✅ Found 127 dependencies
⚠️  Detected 3 vulnerabilities (1 CRITICAL, 2 HIGH)

📊 Summary:
  Total dependencies: 127
  Direct: 15
  Transitive: 112
  Vulnerabilities: 3
  License issues: 0

💡 Next steps:
  1. Review findings: bazbom scan . --format json
  2. Fix vulnerabilities: bazbom fix --suggest
  3. Add to git hooks: bazbom install-hooks

📖 Full documentation: https://github.com/cboyd0319/BazBOM
```

#### Implementation Details

**Location:** `crates/bazbom/src/init.rs`

**Key Components:**
- Build system detection (existing code in `detect_build_system.rs`)
- Interactive prompts using `dialoguer` crate
- Policy template selection (existing templates in `bazbom-policy`)
- First scan execution
- Summary output with actionable next steps
- Config file generation (`bazbom.yml`)

**Dependencies:**
```toml
dialoguer = "0.11"  # For interactive prompts
console = "0.15"     # For styled terminal output
indicatif = "0.17"   # For progress bars
```

**Testing:**
- Unit tests for each prompt flow
- Integration tests for full init workflow
- Test with sample Maven, Gradle, and Bazel projects

#### Acceptance Criteria
- [ ] Detects Maven, Gradle, and Bazel projects correctly
- [ ] Presents 7 policy templates with descriptions
- [ ] Generates valid `bazbom.yml` configuration
- [ ] Runs first scan automatically
- [ ] Displays clear summary with next steps
- [ ] Completes in <60 seconds for typical project
- [ ] Works offline (after initial advisory sync)

---

### 1.2 Expanded Policy Template Library

**Problem:** Current templates cover only basic compliance frameworks. Need domain-specific and framework-specific policies.

**Solution:** Comprehensive template library covering regulatory, industry, and technology-specific use cases.

#### New Templates

**Regulatory Compliance:**
- ✅ PCI-DSS v4.0 (existing)
- ✅ HIPAA Security Rule (existing)
- ✅ FedRAMP Moderate (existing)
- ✅ SOC 2 Type II (existing)
- 🆕 GDPR Data Protection
- 🆕 ISO 27001
- 🆕 NIST Cybersecurity Framework
- 🆕 CIS Benchmarks

**Industry-Specific:**
- 🆕 Financial Services (banking, fintech)
- 🆕 Healthcare Provider
- 🆕 Government/Defense (DoD, NIST 800-53)
- 🆕 SaaS/Cloud Provider
- 🆕 E-commerce/Retail

**Framework-Specific:**
- 🆕 Spring Boot Applications
- 🆕 Android Applications
- 🆕 Microservices Architecture
- 🆕 Kubernetes Deployments
- 🆕 Serverless/Lambda Functions

**Development Stages:**
- 🆕 Development (Permissive, fast feedback)
- 🆕 Staging (Moderate, test deployment)
- 🆕 Production (Strict, zero tolerance)

#### Template Structure

```yaml
# Template: spring-boot-microservices.yml
name: Spring Boot Microservices
description: Optimized for Spring Boot microservice architectures
category: framework-specific
suitable_for:
  - Spring Boot 2.x and 3.x applications
  - Microservice architectures
  - REST API services

policy:
  severity_threshold: HIGH
  
  # Spring-specific concerns
  spring_boot_vulnerabilities:
    enabled: true
    frameworks:
      - spring-core
      - spring-web
      - spring-security
    action: block
  
  # Common microservice dependencies
  allowed_licenses:
    - Apache-2.0
    - MIT
    - BSD-3-Clause
  
  # Block problematic dependencies common in Spring
  blocked_packages:
    - org.springframework:spring-core:4.*  # EOL
    - log4j:log4j:1.*                      # CVE-2021-44228
  
  # KEV filtering (CISA Known Exploited)
  kev_policy:
    action: block
    require_remediation: true
  
  # EPSS threshold for exploit probability
  epss_threshold: 0.5  # Block if >50% probability

recommendations:
  - Use Spring Boot 3.x for latest security updates
  - Enable Spring Security for all endpoints
  - Use SLF4J with Logback (not Log4J 1.x)
  - Scan containers with bazbom scan --containers=auto
```

#### Implementation Details

**Location:** `crates/bazbom-policy/src/templates/`

**Structure:**
```
templates/
├── regulatory/
│   ├── pci_dss.yml
│   ├── hipaa.yml
│   ├── fedramp_moderate.yml
│   ├── soc2.yml
│   ├── gdpr.yml (NEW)
│   ├── iso27001.yml (NEW)
│   └── nist_csf.yml (NEW)
├── industry/
│   ├── financial_services.yml (NEW)
│   ├── healthcare_provider.yml (NEW)
│   ├── government.yml (NEW)
│   └── saas_cloud.yml (NEW)
├── framework/
│   ├── spring_boot.yml (NEW)
│   ├── android.yml (NEW)
│   ├── microservices.yml (NEW)
│   └── kubernetes.yml (NEW)
└── stages/
    ├── development.yml (NEW)
    ├── staging.yml (NEW)
    └── production.yml (NEW)
```

**Template Metadata:**
- Name, description, category
- Suitable for (list of use cases)
- Recommended for (build systems, frameworks)
- Strictness level (permissive, moderate, strict)
- Documentation links

#### Acceptance Criteria
- [ ] 20+ policy templates covering all categories
- [ ] Each template has clear documentation
- [ ] Templates are tested with real projects
- [ ] `bazbom policy init --list` shows categorized templates
- [ ] Templates include recommendations and next steps
- [ ] All templates validate against schema

---

### 1.3 Terminal-Based Interactive Dependency Graph (TUI)

**Problem:** Dependency graphs in JSON/HTML are not interactive. Need real-time exploration.

**Solution:** Terminal User Interface (TUI) for exploring dependency graphs with search, filter, and drill-down.

#### Features

**Interactive Dependency Tree:**
```
┌─ BazBOM Dependency Explorer ─────────────────────────────────────────────────┐
│ Project: my-spring-app v1.0.0                          [F1: Help] [Q: Quit]  │
├───────────────────────────────────────────────────────────────────────────────┤
│ Search: [log4j____________________]  Filter: [ALL ▼]  Sort: [Severity ▼]    │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│ ▼ Direct Dependencies (15)                                                   │
│   ├─⚠️  org.springframework.boot:spring-boot-starter-web:2.7.0              │
│   │   │  2 vulnerabilities (1 HIGH, 1 MEDIUM)                               │
│   │   ├─ org.springframework:spring-web:5.3.20 ⚠️ CVE-2024-xxxx            │
│   │   ├─ org.springframework:spring-webmvc:5.3.20                           │
│   │   └─ org.apache.tomcat.embed:tomcat-embed-core:9.0.60                   │
│   │                                                                           │
│   ├─🔴 org.apache.logging.log4j:log4j-core:2.14.1 CRITICAL                  │
│   │   │  CVE-2021-44228 (Log4Shell) | CISA KEV | CVSS 10.0                 │
│   │   │  → Upgrade to: 2.21.1                                               │
│   │   │  [Press ENTER to apply fix]                                         │
│   │   │                                                                       │
│   ├─✅ com.google.guava:guava:31.1-jre                                      │
│   │   │  No known vulnerabilities                                            │
│   │   ├─ com.google.guava:failureaccess:1.0.1                               │
│   │   └─ com.google.guava:listenablefuture:9999.0                           │
│                                                                               │
│ ▶ Transitive Dependencies (112)                                              │
│                                                                               │
├───────────────────────────────────────────────────────────────────────────────┤
│ Selected: org.apache.logging.log4j:log4j-core:2.14.1                        │
│ Severity: CRITICAL | Scope: compile | License: Apache-2.0                   │
│ Vulnerabilities: 1 | Reachable: YES | Used by: 3 modules                    │
│                                                                               │
│ [Enter] Apply Fix  [Space] Mark for Batch  [/] Search  [F] Filter  [?] Help │
└───────────────────────────────────────────────────────────────────────────────┘
```

#### Implementation Details

**Location:** `crates/bazbom-tui/` (new crate)

**Technology Stack:**
- `ratatui` (formerly tui-rs): Terminal UI framework
- `crossterm`: Cross-platform terminal manipulation
- `tui-tree-widget`: Tree widget for dependencies

**Dependencies:**
```toml
[dependencies]
ratatui = "0.25"
crossterm = "0.27"
tui-tree-widget = "0.18"
unicode-width = "0.1"
```

**Key Features:**
1. **Tree Navigation**
   - Arrow keys to navigate
   - Enter to expand/collapse
   - / for search
   - Tab to switch panes

2. **Search & Filter**
   - Search by package name
   - Filter by severity (CRITICAL, HIGH, MEDIUM, LOW)
   - Filter by vulnerability status (vulnerable, clean, unknown)
   - Filter by license
   - Filter by scope (compile, runtime, test, provided)

3. **Actions**
   - Press Enter on vulnerable dependency to fix
   - Space to mark for batch fixing
   - B to run batch fix on marked items
   - E to export to JSON/SARIF
   - S to generate SBOM

4. **Visual Elements**
   - Color-coded by severity (red, yellow, blue, green)
   - Icons for vulnerability status
   - Progress bars for batch operations
   - Status bar with keyboard shortcuts

**Data Model:**
```rust
// crates/bazbom-tui/src/lib.rs
pub struct DependencyTreeState {
    pub root: Dependency,
    pub selected: Option<DependencyPath>,
    pub expanded: HashSet<DependencyPath>,
    pub marked_for_fix: HashSet<DependencyPath>,
    pub filter: Filter,
    pub search_query: String,
}

pub struct Filter {
    pub severity: Option<Severity>,
    pub vulnerability_status: Option<VulnerabilityStatus>,
    pub license: Option<String>,
    pub scope: Option<DependencyScope>,
}
```

#### Usage

```bash
# Launch TUI with current project
bazbom explore

# Launch TUI with specific SBOM
bazbom explore --sbom=sbom.spdx.json

# Launch TUI with specific findings
bazbom explore --findings=sca_findings.json
```

#### Acceptance Criteria
- [ ] Tree view displays all dependencies
- [ ] Expand/collapse works correctly
- [ ] Search finds dependencies by name
- [ ] Filters work for severity, license, scope
- [ ] Color-coding matches severity
- [ ] One-click fix launches remediation
- [ ] Batch fixing works for multiple dependencies
- [ ] Export generates valid SBOM/SARIF
- [ ] Keyboard shortcuts are discoverable (help screen)
- [ ] Works on macOS, Linux, and Windows
- [ ] Handles large projects (1000+ dependencies)

---

### 1.4 Enhanced `bazbom fix --interactive` with Smart Batch Processing

**Problem:** Fixing 10+ vulnerabilities one-by-one is tedious. Need intelligent batching.

**Solution:** Interactive mode with smart grouping, dependency conflict detection, and test verification.

#### Features

```bash
$ bazbom fix --interactive
🔍 Found 12 fixable vulnerabilities

📊 Grouping by impact analysis...
✅ Safe batch groups identified: 3

┌─ Batch 1: Low-Risk Updates (8 vulnerabilities) ──────────────────────────────┐
│ These updates are independent and safe to apply together:                    │
│                                                                               │
│  1. log4j-core: 2.14.1 → 2.21.1 (CRITICAL)                                  │
│  2. spring-web: 5.3.20 → 5.3.31 (HIGH)                                      │
│  3. jackson-databind: 2.13.0 → 2.16.0 (HIGH)                                │
│  4. guava: 30.1-jre → 32.1.3-jre (MEDIUM)                                   │
│  5. commons-io: 2.7 → 2.15.0 (MEDIUM)                                       │
│  6. commons-codec: 1.15 → 1.16.0 (LOW)                                      │
│  7. httpclient: 4.5.13 → 4.5.14 (LOW)                                       │
│  8. slf4j-api: 1.7.32 → 2.0.9 (INFO)                                        │
│                                                                               │
│ Estimated time: ~45 seconds                                                  │
│ Test coverage: 127 tests will run                                            │
│                                                                               │
│ [Enter] Apply batch  [S] Skip  [I] Individual mode  [Q] Quit                │
└───────────────────────────────────────────────────────────────────────────────┘

Apply this batch? [Y/n]: y

⏳ Applying 8 updates...
✅ 1/8 Updated log4j-core
✅ 2/8 Updated spring-web
✅ 3/8 Updated jackson-databind
✅ 4/8 Updated guava
✅ 5/8 Updated commons-io
✅ 6/8 Updated commons-codec
✅ 7/8 Updated httpclient
✅ 8/8 Updated slf4j-api

🧪 Running tests...
⏳ mvn test (127 tests)...
✅ All tests passed! (45.3 seconds)

✅ Batch 1 complete! 8 vulnerabilities fixed.

┌─ Batch 2: Moderate-Risk Updates (3 vulnerabilities) ─────────────────────────┐
│ These updates may have breaking changes:                                     │
│                                                                               │
│  9. spring-boot: 2.7.0 → 3.2.0 (HIGH) ⚠️ Major version upgrade              │
│     Breaking: Spring Boot 3 requires Java 17+                                │
│     Migration guide: https://spring.io/blog/2022/...                         │
│                                                                               │
│ 10. hibernate-core: 5.6.0 → 6.3.1 (MEDIUM) ⚠️ Major version upgrade        │
│     Breaking: API changes in Session interface                               │
│                                                                               │
│ 11. junit: 4.13.2 → 5.10.0 (LOW) ⚠️ Major version upgrade                  │
│     Breaking: Package name change (org.junit → org.junit.jupiter)           │
│                                                                               │
│ [Enter] Review individually  [S] Skip batch  [L] Learn more  [Q] Quit       │
└───────────────────────────────────────────────────────────────────────────────┘

Skip batch 2? [y/N]: y

┌─ Batch 3: Dependency Conflicts (1 vulnerability) ────────────────────────────┐
│ This update conflicts with other dependencies:                               │
│                                                                               │
│ 12. netty-codec: 4.1.70 → 4.1.100 (CRITICAL)                               │
│     ⚠️ Conflicts with:                                                       │
│       - spring-boot-starter-web requires netty-codec:4.1.70-4.1.85          │
│       - grpc-netty requires netty-codec:4.1.70                              │
│                                                                               │
│     Options:                                                                  │
│       1. Update spring-boot-starter-web to 3.2.0 (resolves conflict)        │
│       2. Override version (may cause runtime issues)                         │
│       3. Wait for dependency updates                                         │
│                                                                               │
│ [1/2/3] Choose option or [S] Skip: 3                                        │
└───────────────────────────────────────────────────────────────────────────────┘

📊 Summary:
  Fixed: 8 vulnerabilities
  Skipped: 4 vulnerabilities (requires manual review)
  
💡 Next steps:
  1. Commit changes: git commit -m "fix: upgrade vulnerable dependencies"
  2. Create PR: bazbom fix --pr
  3. Review skipped: Review Batch 2 and 3 manually

🎉 Great job! Your project is more secure.
```

#### Implementation Details

**Location:** `crates/bazbom/src/remediation.rs` (extend existing)

**Smart Grouping Algorithm:**

```rust
// crates/bazbom/src/batch_fixer.rs (NEW)
pub struct BatchFixer {
    vulnerabilities: Vec<Vulnerability>,
    dependency_graph: DependencyGraph,
}

impl BatchFixer {
    /// Group vulnerabilities into safe batches
    pub fn create_batches(&self) -> Vec<Batch> {
        let mut batches = vec![];
        
        // Batch 1: Independent updates (no shared dependencies)
        let independent = self.find_independent_updates();
        if !independent.is_empty() {
            batches.push(Batch {
                risk: RiskLevel::Low,
                updates: independent,
                conflicts: vec![],
                breaking_changes: false,
            });
        }
        
        // Batch 2: Updates with breaking changes
        let breaking = self.find_breaking_updates();
        if !breaking.is_empty() {
            batches.push(Batch {
                risk: RiskLevel::Moderate,
                updates: breaking,
                conflicts: vec![],
                breaking_changes: true,
            });
        }
        
        // Batch 3: Conflicting updates
        let conflicts = self.find_conflicting_updates();
        if !conflicts.is_empty() {
            batches.push(Batch {
                risk: RiskLevel::High,
                updates: conflicts.updates,
                conflicts: conflicts.conflicts,
                breaking_changes: false,
            });
        }
        
        batches
    }
    
    fn find_independent_updates(&self) -> Vec<Update> {
        // Use dependency graph to find updates that don't share transitive deps
        // ...
    }
    
    fn find_breaking_updates(&self) -> Vec<Update> {
        // Detect major version bumps (semver)
        // Check against known breaking change database
        // ...
    }
    
    fn find_conflicting_updates(&self) -> ConflictingUpdates {
        // Detect version conflicts using dependency resolution
        // ...
    }
}
```

**Interactive Prompts:**
- Use `dialoguer` for interactive prompts
- Show progress with `indicatif`
- Colorize output with `console`

**Breaking Change Detection:**
- Major version changes (semver)
- Known breaking changes database (JSON)
- Java version requirements
- API deprecations

**Conflict Detection:**
- Maven: Use `mvn dependency:tree -Dverbose=true`
- Gradle: Use `gradle dependencies --configuration runtime`
- Bazel: Analyze maven_install.json

#### Acceptance Criteria
- [ ] Groups vulnerabilities into safe batches
- [ ] Detects breaking changes (major version bumps)
- [ ] Identifies dependency conflicts
- [ ] Provides migration guidance for breaking changes
- [ ] Tests each batch independently
- [ ] Rolls back on test failure
- [ ] Shows clear progress indicators
- [ ] Allows skipping batches
- [ ] Generates summary report
- [ ] Works for Maven, Gradle, and Bazel

---

## Phase 2: Visual Excellence (Weeks 3-4)

### Objective
Provide compelling visual interfaces for both technical and non-technical stakeholders.

### 2.1 Embedded Web Dashboard

**Problem:** CLI output is not suitable for executives, compliance teams, or non-developers.

**Solution:** Self-hosted web dashboard with interactive visualizations, exportable reports, and shareable static HTML.

#### Features

**Dashboard Home:**
- Security score (0-100) with trend
- Vulnerability breakdown (CRITICAL, HIGH, MEDIUM, LOW)
- License compliance status
- Policy violations
- Dependency count and growth

**Interactive Dependency Graph:**
- D3.js force-directed graph
- Click to drill down into transitive dependencies
- Color-coded by vulnerability severity
- Filter by license, scope, vulnerability status
- Export as PNG/SVG

**Vulnerability Timeline:**
- Chart showing vulnerability introductions over time
- Trend analysis (improving, degrading, stable)
- Remediation velocity metrics

**SBOM Explorer:**
- Searchable table of all dependencies
- Column: Name, Version, License, Vulnerabilities, Scope
- Export to CSV, JSON, SPDX, CycloneDX

**Reports:**
- Executive summary (1-page PDF)
- Compliance report (regulatory frameworks)
- Audit trail (all scans and changes)

#### Technology Stack

**Backend:**
- Rust Axum web framework
- Tera templates (Jinja2-like)
- Tower middleware for compression, logging

**Frontend:**
- React 18 for UI components
- D3.js v7 for dependency graph
- Chart.js for trend charts
- Tailwind CSS for styling
- shadcn/ui for component library

**Data Flow:**
```
.bazbom/cache/  →  Axum Server  →  REST API  →  React Frontend
  sbom.json           (Rust)          (JSON)       (Browser)
  findings.json
  graph.json
```

#### Implementation Details

**Location:** 
- Backend: `crates/bazbom-dashboard/` (new crate)
- Frontend: `crates/bazbom-dashboard/frontend/` (React app)

**Backend Structure:**
```
crates/bazbom-dashboard/
├── Cargo.toml
├── src/
│   ├── main.rs            # Axum server entry point
│   ├── routes.rs          # API routes
│   ├── handlers/
│   │   ├── dashboard.rs   # Dashboard page
│   │   ├── vulnerabilities.rs
│   │   ├── dependencies.rs
│   │   └── sbom.rs
│   ├── models.rs          # Data models
│   └── templates/
│       ├── base.html      # Base template
│       ├── dashboard.html
│       └── graph.html
└── static/
    ├── css/
    ├── js/
    └── assets/
```

**Frontend Structure:**
```
crates/bazbom-dashboard/frontend/
├── package.json
├── vite.config.ts
├── src/
│   ├── App.tsx
│   ├── components/
│   │   ├── DependencyGraph.tsx   # D3.js visualization
│   │   ├── VulnerabilityChart.tsx
│   │   ├── SbomExplorer.tsx
│   │   └── SecurityScore.tsx
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   ├── Vulnerabilities.tsx
│   │   └── Dependencies.tsx
│   └── api/
│       └── client.ts         # API client
└── public/
```

**API Endpoints:**
```rust
// GET /api/dashboard/summary
{
  "security_score": 78,
  "total_dependencies": 127,
  "vulnerabilities": {
    "critical": 1,
    "high": 3,
    "medium": 5,
    "low": 2
  },
  "license_issues": 0,
  "policy_violations": 1
}

// GET /api/dependencies/graph
{
  "nodes": [
    {"id": "log4j-core:2.14.1", "severity": "critical", ...},
    {"id": "spring-web:5.3.20", "severity": "high", ...}
  ],
  "edges": [
    {"source": "spring-boot", "target": "spring-web", ...}
  ]
}

// GET /api/vulnerabilities
{
  "vulnerabilities": [
    {
      "cve": "CVE-2021-44228",
      "package": "log4j-core",
      "version": "2.14.1",
      "severity": "critical",
      "cvss": 10.0,
      "fixed_version": "2.21.1",
      "reachable": true
    }
  ]
}
```

**Embedded vs External:**
- Primary: Embedded React app (bundled with Rust binary)
- Alternative: Generate static HTML for offline sharing
- Shareable: Export dashboard as self-contained HTML file

#### Usage

```bash
# Start dashboard server
bazbom dashboard

# Start on custom port
bazbom dashboard --port 8080

# Generate static HTML export
bazbom dashboard --export dashboard.html

# Open in browser automatically
bazbom dashboard --open

# Output:
🚀 BazBOM Dashboard running at http://localhost:3000
📊 Security Score: 78/100
⚠️  3 HIGH vulnerabilities require attention
```

#### Acceptance Criteria
- [x] Dashboard loads in <2 seconds ✅
- [x] D3.js graph renders with force-directed layout ✅
- [x] All charts are interactive (D3.js + Chart.js) ✅
- [x] Responsive design (mobile, tablet, desktop) ✅
- [x] Accessible (WCAG 2.1 AA compliant) ✅
- [x] Works with all three build systems ✅
- [x] No external API calls (privacy-preserving) ✅
- [x] Auto-refresh capability (30s interval) ✅
- [ ] Export static HTML works offline (future)
- [ ] Generate PDF reports (future)
- [ ] Live reload on new scans (future)

---

### 2.2 Enhanced Reports for Non-Technical Stakeholders

**Problem:** CISOs and executives don't read JSON/SARIF output.

**Solution:** Executive-friendly reports with risk scoring, trends, and recommendations.

#### Report Types

**1. Executive Summary (1-page)**
- Security score with historical trend
- Top 5 risks requiring immediate attention
- Compliance status (pass/fail per framework)
- Cost of inaction (estimated breach cost)
- Recommended actions with priority

**2. Compliance Report**
- Framework-specific (PCI-DSS, HIPAA, etc.)
- Pass/fail per requirement
- Evidence citations
- Remediation roadmap
- Audit-ready formatting

**3. Developer Report**
- All vulnerabilities with fix instructions
- Dependency tree visualization
- Breaking change warnings
- Test coverage impact
- Estimated remediation time

**4. Trend Report**
- Vulnerability introductions over time
- Remediation velocity
- Mean time to fix (MTTF)
- Repeat offenders (frequently vulnerable deps)
- Team performance metrics

#### Implementation

**Location:** `crates/bazbom-reports/` (new crate)

**Dependencies:**
```toml
[dependencies]
printpdf = "0.7"        # PDF generation
plotters = "0.3"        # Charts for PDF
serde_json = "1.0"
tera = "1.19"           # Templates
```

**Report Generation:**
```rust
// crates/bazbom-reports/src/lib.rs
pub enum ReportType {
    Executive,
    Compliance(ComplianceFramework),
    Developer,
    Trend,
}

pub struct ReportGenerator {
    findings: Findings,
    sbom: Sbom,
    historical_data: Vec<Scan>,
}

impl ReportGenerator {
    pub fn generate(&self, report_type: ReportType) -> Result<Vec<u8>> {
        match report_type {
            ReportType::Executive => self.generate_executive_summary(),
            ReportType::Compliance(framework) => self.generate_compliance(framework),
            ReportType::Developer => self.generate_developer_report(),
            ReportType::Trend => self.generate_trend_report(),
        }
    }
}
```

**Usage:**
```bash
# Generate executive PDF
bazbom report --type executive --output executive.pdf

# Generate compliance report
bazbom report --type compliance --framework pci-dss --output compliance.pdf

# Generate all reports
bazbom report --all --output-dir reports/

# Email report
bazbom report --type executive --email ciso@company.com
```

#### Acceptance Criteria
- [ ] Generates professional PDF reports
- [ ] Includes charts and visualizations
- [ ] Compliance reports map to frameworks
- [ ] Executive summary fits on 1 page
- [ ] Reports include actionable recommendations
- [ ] Supports custom branding (logo, colors)
- [ ] Email integration works
- [ ] Historical trend analysis requires 3+ scans

---

## Phase 3: IDE Polish (Weeks 5-6)

### Objective
Bring IDE plugins from 95% (scaffolded) to 100% (production-ready, published).

### 3.1 VS Code Extension 1.0 Release

**Current Status:** 95% complete, needs testing and marketplace publishing

**Remaining Work:**

1. **Testing & Quality**
   - [ ] Manual testing with real projects
   - [ ] Performance profiling
   - [ ] Edge case handling
   - [ ] Error message improvements
   - [ ] Accessibility testing

2. **Marketplace Preparation**
   - [ ] README with screenshots and GIFs
   - [ ] Demo video (30-60 seconds)
   - [ ] VS Code marketplace account setup
   - [ ] Icon and banner design
   - [ ] Changelog and versioning

3. **Features Polish**
   - [ ] Improved diagnostic ranges (exact line/column)
   - [ ] Code actions for all vulnerability types
   - [ ] Settings UI improvements
   - [ ] Status bar integration
   - [ ] Output channel for logs

4. **Documentation**
   - [ ] Installation guide
   - [ ] Configuration guide
   - [ ] Troubleshooting guide
   - [ ] Keyboard shortcuts reference

#### Implementation Tasks

**Testing:**
```bash
cd crates/bazbom-vscode-extension
npm test
npm run lint
npm run package
```

**Publishing:**
```bash
# Build VSIX package
npx vsce package

# Publish to marketplace
npx vsce publish
```

#### Acceptance Criteria
- [ ] Published to VS Code Marketplace
- [ ] 4.5+ star rating (after 50+ reviews)
- [ ] 1000+ installs in first month
- [ ] <5 seconds scan time
- [ ] Works with Maven, Gradle, Bazel
- [ ] Zero crashes reported in telemetry

---

### 3.2 IntelliJ IDEA Plugin Beta Release

**Current Status:** 95% complete, needs testing and marketplace publishing

**Remaining Work:**

1. **Testing & Quality**
   - [ ] Test with IntelliJ IDEA Community & Ultimate
   - [ ] Test with Android Studio
   - [ ] Performance profiling with large projects
   - [ ] Memory leak detection
   - [ ] Thread safety audit

2. **Marketplace Preparation**
   - [ ] JetBrains Marketplace account
   - [ ] Plugin description and screenshots
   - [ ] Demo video
   - [ ] Icon design (256x256 PNG)
   - [ ] Compatibility testing (2023.3+)

3. **Features Polish**
   - [ ] Settings panel UI polish
   - [ ] Tool window layout improvements
   - [ ] Dependency tree performance optimization
   - [ ] Quick fix reliability improvements
   - [ ] Notification system refinement

4. **Documentation**
   - [ ] Installation guide
   - [ ] Configuration guide
   - [ ] Feature showcase
   - [ ] Known limitations

#### Implementation Tasks

**Testing:**
```bash
cd crates/bazbom-intellij-plugin
./gradlew test
./gradlew runIde  # Manual testing
```

**Publishing:**
```bash
# Build plugin
./gradlew buildPlugin

# Publish to marketplace
./gradlew publishPlugin
```

#### Acceptance Criteria
- [ ] Published to JetBrains Marketplace
- [ ] Beta tag (for initial release)
- [ ] 500+ downloads in first month
- [ ] <10 second scan time for 1000 deps
- [ ] Works with Maven, Gradle, Bazel
- [ ] Compatible with IntelliJ 2023.3+
- [ ] Zero critical bugs in first week

---

### 3.3 One-Click Remediation Polish

**Current Status:** Code complete, needs real-world testing

**Remaining Work:**

1. **Testing with Real Projects**
   - [ ] Test with popular Spring Boot projects
   - [ ] Test with Android projects
   - [ ] Test with Gradle multi-module projects
   - [ ] Test with Bazel monorepos
   - [ ] Test with projects that have test failures

2. **Reliability Improvements**
   - [ ] Better version conflict detection
   - [ ] Handle version properties (${log4j.version})
   - [ ] Support parent POM versions
   - [ ] Improve string replacement accuracy
   - [ ] Handle edge cases (comments, whitespace)

3. **User Experience**
   - [ ] Clearer progress indicators
   - [ ] Better error messages
   - [ ] Undo support (beyond rollback)
   - [ ] Dry-run mode improvements
   - [ ] Batch fix UI improvements

4. **GitHub Integration**
   - [ ] Test PR generation with real repos
   - [ ] Improve PR descriptions
   - [ ] Add PR labels automatically
   - [ ] Support draft PRs
   - [ ] Request reviews automatically

#### Acceptance Criteria
- [ ] 95% success rate on real projects
- [ ] Zero data loss incidents
- [ ] Tests pass after 90% of fixes
- [ ] PR generation works for public and private repos
- [ ] Rollback works in 100% of cases
- [ ] Clear error messages for failures

---

## Phase 4: Team Features (Weeks 7-8)

### Objective
Enable security teams to coordinate and collaborate efficiently.

### 4.1 Git-Based Team Coordination

**Problem:** Security teams lack tools for assignment, tracking, and coordination.

**Solution:** Leverage git for assignment tracking, team notifications, and audit trails.

#### Features

**1. Assignment System**

Track who is responsible for fixing vulnerabilities using git notes.

```bash
# Assign vulnerability to team member
bazbom assign CVE-2021-44228 --to alice@company.com

# List assignments
bazbom assign --list
CVE-2021-44228 → alice@company.com (assigned 2 days ago)
CVE-2024-xxxx  → bob@company.com (assigned 1 hour ago)

# Show my assignments
bazbom assign --me
2 vulnerabilities assigned to you:
  1. CVE-2021-44228 (CRITICAL) - log4j-core
  2. CVE-2024-yyyy (HIGH) - spring-web
```

**Implementation:**
- Use `git notes` to store assignments
- Namespace: `refs/notes/bazbom/assignments`
- Format: JSON metadata per CVE

```bash
# Internally:
git notes --ref=bazbom/assignments add -m '{"cve":"CVE-2021-44228","assignee":"alice@company.com","assigned_at":"2024-11-03T10:00:00Z"}' HEAD
```

**2. Team Notifications**

Notify team members of new vulnerabilities via git hooks.

```bash
# Configure team notifications
bazbom team-config --slack-webhook https://hooks.slack.com/...
bazbom team-config --email smtp://mail.company.com

# Enable notifications on scan
bazbom scan . --notify-team

# Output:
✅ Scan complete
📧 Notified 3 team members:
   - alice@company.com (2 CRITICAL vulnerabilities)
   - bob@company.com (1 HIGH vulnerability)
   - security-team Slack channel
```

**Notification Channels:**
- Slack webhooks
- Email (SMTP)
- Microsoft Teams webhooks
- GitHub Issues (auto-create)

**3. Audit Trail**

Track all security actions in git history.

```bash
# Show audit log
bazbom audit log

2024-11-03 10:00:00  alice@company.com  Scanned project
2024-11-03 10:05:00  alice@company.com  Fixed CVE-2021-44228 (log4j-core)
2024-11-03 10:10:00  alice@company.com  Created PR #123
2024-11-03 11:00:00  bob@company.com    Approved PR #123
2024-11-03 11:05:00  bob@company.com    Merged PR #123

# Show audit log for specific CVE
bazbom audit log --cve CVE-2021-44228

2024-11-03 10:00:00  Detected in log4j-core:2.14.1
2024-11-03 10:05:00  Fixed by alice@company.com (upgraded to 2.21.1)
2024-11-03 10:10:00  Tests passed
2024-11-03 10:15:00  PR #123 created
2024-11-03 11:05:00  PR #123 merged

# Export audit log
bazbom audit export --format csv --output audit.csv
```

**Implementation:**
- Use git commits with structured messages
- Commit message format: `[bazbom] action: details`
- Parse git log for audit trail
- Store in `.bazbom/audit.json` for performance

**4. Team Dashboard**

Web dashboard showing team metrics.

```
┌─ Team Dashboard ─────────────────────────────────────────────┐
│                                                               │
│ Team: Security Engineering                                   │
│ Members: 5                                                    │
│                                                               │
│ Open Vulnerabilities: 12                                      │
│   CRITICAL: 2  HIGH: 5  MEDIUM: 3  LOW: 2                    │
│                                                               │
│ Assigned:                                                     │
│   alice@company.com: 3 (1 CRITICAL, 2 HIGH)                  │
│   bob@company.com: 2 (1 HIGH, 1 MEDIUM)                      │
│   carol@company.com: 1 (1 MEDIUM)                            │
│   Unassigned: 6                                               │
│                                                               │
│ Metrics (Last 30 Days):                                       │
│   Mean Time to Fix (MTTF): 2.3 days                          │
│   Vulnerabilities Fixed: 24                                   │
│   Vulnerabilities Introduced: 8                               │
│   Net Security Improvement: +16                               │
│                                                               │
│ Top Contributors:                                             │
│   1. alice@company.com (12 fixes)                            │
│   2. bob@company.com (8 fixes)                               │
│   3. carol@company.com (4 fixes)                             │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

#### Implementation Details

**Location:** 
- `crates/bazbom/src/team.rs` (new module)
- `crates/bazbom-dashboard/src/handlers/team.rs`

**Git Integration:**
```rust
// crates/bazbom/src/team.rs
pub struct TeamCoordinator {
    repo: Repository,
}

impl TeamCoordinator {
    pub fn assign(&self, cve: &str, assignee: &str) -> Result<()> {
        let metadata = AssignmentMetadata {
            cve: cve.to_string(),
            assignee: assignee.to_string(),
            assigned_at: Utc::now(),
        };
        
        // Store in git notes
        self.repo.notes(&format!("bazbom/assignments/{}", cve), &serde_json::to_string(&metadata)?)?;
        
        Ok(())
    }
    
    pub fn list_assignments(&self) -> Result<Vec<Assignment>> {
        // Parse git notes
        // ...
    }
    
    pub fn audit_log(&self) -> Result<Vec<AuditEntry>> {
        // Parse git log for [bazbom] commits
        // ...
    }
}
```

**Notification System:**
```rust
// crates/bazbom/src/notifications.rs (NEW)
pub enum NotificationChannel {
    Slack { webhook_url: String },
    Email { smtp_url: String },
    Teams { webhook_url: String },
    GithubIssue { token: String, repo: String },
}

pub struct Notifier {
    channels: Vec<NotificationChannel>,
}

impl Notifier {
    pub fn send(&self, message: &Notification) -> Result<()> {
        for channel in &self.channels {
            match channel {
                NotificationChannel::Slack { webhook_url } => {
                    self.send_slack(webhook_url, message)?;
                }
                NotificationChannel::Email { smtp_url } => {
                    self.send_email(smtp_url, message)?;
                }
                // ...
            }
        }
        Ok(())
    }
}
```

#### Usage

```bash
# Configure team
bazbom team-config --name "Security Team"
bazbom team-config --add-member alice@company.com
bazbom team-config --add-member bob@company.com
bazbom team-config --slack https://hooks.slack.com/...

# Assign vulnerabilities
bazbom assign CVE-2021-44228 --to alice@company.com
bazbom assign CVE-2024-xxxx --to bob@company.com

# Auto-assign (round-robin)
bazbom assign --auto

# Send notifications
bazbom scan . --notify-team

# View team dashboard
bazbom dashboard --team

# Export audit log
bazbom audit export --output audit.csv
```

#### Acceptance Criteria
- [ ] Assignments stored in git notes
- [ ] Notifications sent to Slack/Email/Teams
- [ ] Audit log tracks all security actions
- [ ] Team dashboard shows metrics
- [ ] Round-robin auto-assignment works
- [ ] Works with git and GitHub
- [ ] Privacy-preserving (no external services required)

---

## Implementation Strategy

### Development Approach

1. **Incremental Delivery**
   - Ship features as they complete
   - Beta releases for early feedback
   - Dogfood internally before public release

2. **Testing Priority**
   - Unit tests for all new code
   - Integration tests for workflows
   - Real-world testing with sample projects
   - Performance testing for TUI and dashboard

3. **Documentation First**
   - Write docs before implementing
   - Include examples and screenshots
   - Video demos for complex features
   - Update docs with each release

4. **Community Involvement**
   - Request feedback on GitHub Discussions
   - Beta testers from community
   - Incorporate user feedback rapidly
   - Acknowledge contributors

### Risk Management

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| TUI performance issues | Medium | High | Profile early, optimize hot paths |
| Dashboard complexity | Medium | Medium | Use proven frameworks (React, D3) |
| IDE plugin bugs | High | High | Extensive testing, beta period |
| Team features adoption | High | Medium | Clear documentation, demos |
| Breaking changes | Low | High | Version carefully, migration guide |

### Success Metrics

**Phase 1 (Quick Wins):**
- ✅ 80% of new users complete `bazbom init` successfully
- ✅ 50% of users try TUI dependency explorer
- ✅ Batch fixing saves 50%+ time vs individual fixes
- ✅ Policy templates cover 90% of use cases

**Phase 2 (Visual Excellence):**
- ✅ Dashboard loads <2 seconds
- ✅ 75% of executives prefer dashboard over CLI
- ✅ Reports used in 50%+ of compliance audits
- ✅ Static HTML export used for sharing

**Phase 3 (IDE Polish):**
- ✅ 1000+ VS Code extension installs
- ✅ 500+ IntelliJ plugin downloads
- ✅ 4.5+ star rating on both marketplaces
- ✅ <10 second scan time in IDE
- ✅ 95% fix success rate

**Phase 4 (Team Features):**
- ✅ 50% of teams use assignment system
- ✅ 75% of teams configure notifications
- ✅ Audit logs used in security reviews
- ✅ Team dashboard accessed weekly

### Timeline Summary

| Week | Focus | Key Deliverables |
|------|-------|-----------------|
| 1 | Interactive init, policy templates | `bazbom init`, 20+ templates |
| 2 | TUI, batch fixing | `bazbom explore`, `bazbom fix --interactive` |
| 3 | Dashboard backend | Axum server, API routes |
| 4 | Dashboard frontend | React app, D3 graphs, reports |
| 5 | VS Code polish | Testing, marketplace publish |
| 6 | IntelliJ polish | Testing, marketplace publish |
| 7 | Team coordination | Assignments, notifications |
| 8 | Team dashboard, audit | Team metrics, audit log |

---

## Next Steps

### Immediate Actions (This Week)

1. **Review & Approve**
   - [ ] Maintainers review this roadmap
   - [ ] Validate priorities and timeline
   - [ ] Approve scope and features

2. **Setup**
   - [ ] Create GitHub Project board
   - [ ] Create issues for each feature
   - [ ] Assign owners to each phase
   - [ ] Set up milestones

3. **Start Development**
   - [ ] Begin Phase 1.1 (Interactive init)
   - [ ] Setup TUI crate structure
   - [ ] Create policy template files
   - [ ] Write tests for new features

### For Contributors

- Pick issues labeled "good first issue"
- Review and test features as they ship
- Provide feedback on GitHub Discussions
- Help with documentation and examples
- Test beta releases with your projects

### For Users

- Stay tuned for beta releases
- Provide feedback on new features
- Share your workflows and use cases
- Help spread the word

---

## Conclusion

This roadmap will transform BazBOM from a powerful CLI tool into the ultimate easy-to-use SBOM, SCA, and dependency graph solution. By focusing on developer experience, visual excellence, and team coordination, we'll make BazBOM the tool that developers WANT to use, not just what security teams mandate.

**Timeline:** 8 weeks  
**Deliverables:** 15+ major features  
**Impact:** 10x improvement in developer productivity  

Let's build something amazing. 🚀

---

**Document Version:** 1.0  
**Next Review:** After Phase 1 completion (Week 3)  

**Feedback:** Please provide feedback via GitHub Discussions or Issues.
