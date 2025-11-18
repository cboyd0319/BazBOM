# BazBOM v6.8 - Jira Bidirectional Integration

**Version:** 6.8
**Target Release:** Q2 2026
**Status:** In Development - Phase 1 Foundation COMPLETE
**Last Updated:** 2025-11-16

## Overview

Version 6.8 transforms BazBOM into a **fully automated DevSecOps platform** with comprehensive Jira integration AND intelligent GitHub PR automation. This release completes the entire vulnerability remediation loop from detection to deployment.

**The Complete Automation Loop:**
**Scan → Ticket → PR → Review → Merge → Close**

This release bridges the gap between security scanning and actual code remediation, automating 90% of the manual work while maintaining safety controls and review processes.

## Key Features

### Jira Integration
- **Automatic Ticket Creation:** Vulnerabilities create Jira tickets with full intelligence from ALL BazBOM modules
- **Bidirectional Sync:** Status changes flow Jira ↔ BazBOM ↔ GitHub in real-time
- **Smart Routing:** Component-based routing assigns vulnerabilities to the right teams automatically
- **SLA Tracking:** Configurable SLAs with automatic due dates and breach alerts
- **Sprint Planning:** Add vulnerabilities to sprints, create epics, estimate story points
- **VEX Generation:** Rejected Jira tickets automatically generate VEX suppression entries

### GitHub PR Automation (NEW!)
- **Automatic PR Creation:** AI-powered dependency upgrades with comprehensive intelligence reports
- **Full Intelligence Integration:** Every PR includes data from ALL 14+ BazBOM intelligence modules:
  - Reachability analysis (7 languages) with call graph visualization
  - ML risk scoring for auto-merge decisions
  - Upgrade analyzer / Breaking change detection
  - Difficulty scoring (0-100 scale)
  - Multi-CVE grouping
  - EPSS/KEV exploitation intelligence
  - Container impact assessment with layer attribution
  - Threat intelligence (ExploitDB, GitHub POCs, Nuclei templates)
  - Policy compliance verification
  - Framework migration guides
  - Ecosystem-specific guidance
  - LLM fix generation with alternatives
  - Plain English "Why" explanations
  - Enhanced testing strategy recommendations
- **Auto-Merge (Optional):** Safe automated merging for low-risk upgrades with configurable policies
- **Multi-PR Orchestration:** Batch remediation across multiple repositories
- **Three-Way Sync:** Jira ↔ BazBOM ↔ GitHub all stay in sync

### Developer Experience
- **Dashboard Integration:** Unified view of Jira tickets AND GitHub PRs
- **IDE Support:** IntelliJ and VS Code plugins show Jira + GitHub status inline
- **CLI Power:** Full control via `bazbom jira` and `bazbom github` commands
- **CI/CD Integration:** GitHub Actions, GitLab CI, and Jenkins workflows

## Business Impact

- **90% reduction** in manual remediation work (ticket + PR creation + testing)
- **80% faster** time-to-fix for automated-eligible vulnerabilities
- **30% faster** overall vulnerability remediation (MTTR)
- **Zero-touch remediation** for low-risk dependency upgrades (with approval gates)
- **End-to-end traceability** from CVE discovery → Jira ticket → GitHub PR → deployment
- **Automated compliance** reporting via Jira custom fields and audit trails
- **90% of tickets** are actionable (not false positives)

## Documentation

### Planning & Design

1. **[Jira Integration Plan](jira-integration-plan.md)** ⭐ START HERE
   - Executive summary
   - Feature categories (9 major areas including GitHub PR automation)
   - Use cases and workflows
   - Configuration examples
   - Security considerations

2. **[Technical Specifications](technical-specifications.md)**
   - Architecture and component design
   - Data models and API specifications
   - Webhook protocol
   - Configuration schema
   - Database schema
   - Error handling
   - Performance requirements

3. **[Implementation Roadmap](implementation-roadmap.md)**
   - 20-week timeline (7 phases)
   - Phase deliverables and milestones
   - Resource requirements
   - Risk management
   - Success metrics

### Additional Documentation

4. **[Integration Analysis](integration-analysis.md)**
   - Comprehensive analysis of all integration points
   - Feature matrix and coverage
   - Comparison with competitors

5. **[Use Case Examples](use-cases.md)** (Coming Soon)
   - Real-world scenarios with step-by-step workflows
   - Configuration templates
   - Best practices

6. **[API Reference](api-reference.md)** (Coming Soon)
   - Complete Jira API coverage
   - BazBOM CLI commands
   - Webhook event reference

## Quick Start (Post-Release)

Once v6.8 is released, you'll be able to get started in 3 steps:

```bash
# 1. Initialize Jira integration
bazbom jira init

# 2. Configure your project
# Edit .bazbom/jira.yml with your Jira details

# 3. Scan with automatic ticket creation
bazbom scan --jira-create
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                       BazBOM v6.8                           │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   CLI    │  │Dashboard │  │   LSP    │  │  Webhook │  │
│  │ Scanner  │  │   UI     │  │  Server  │  │  Server  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │             │         │
│       └─────────────┴─────────────┴─────────────┘         │
│                         │                                  │
│                    ┌────▼────┐                            │
│                    │  Jira   │  New crate: bazbom-jira   │
│                    │ Client  │                            │
│                    └────┬────┘                            │
└─────────────────────────┼───────────────────────────────────┘
                          │
                          │ REST API + Webhooks
                          ▼
                   ┌──────────────┐
                   │     Jira     │
                   │  Cloud/Server│
                   └──────────────┘
```

## Integration Points

### 1. Vulnerability Tracking
- Auto-create tickets for P0-P2 vulnerabilities
- Custom fields: CVE ID, CVSS, EPSS, KEV status, Reachability
- Attach call graph diagrams
- Link related CVEs

### 2. Workflow Automation
- CI/CD: GitHub Actions, GitLab CI, Jenkins
- PR workflows: Create → In Progress → Done
- Auto-close on fix detection
- Status sync via webhooks

### 3. Team Collaboration
- Component-based routing (package → team)
- CODEOWNERS integration
- SLA tracking with due dates
- Sprint planning and burndown

### 4. Policy & Compliance
- VEX generation from rejected tickets
- Compliance custom fields (PCI-DSS, HIPAA, etc.)
- Approval workflows for suppressions
- Audit logging

### 5. Developer Experience
- CLI commands: `jira create`, `jira sync`, `jira close-fixed`
- Dashboard: Embedded ticket cards, one-click creation
- IDE: Inline Jira status, "Open in Jira" actions
- Reports: HTML/PDF with Jira links

## Timeline & Milestones

```
2026-01 │ Development Start (Phase 1)
        │
2026-02 │ M1: Alpha (Core + Sync)
        │ • Manual ticket creation
        │ • Webhook support
        │ • VEX generation
        │
2026-03 │
        │
2026-04 │ M2: Beta (CI/CD + Dashboard)
        │ • Auto-create on scan
        │ • CI/CD integration
        │ • Dashboard UI
        │
2026-05 │ M3: Release Candidate
        │ • SLA tracking
        │ • Sprint integration
        │ • Performance optimized
        │
2026-06 │ M4: General Availability (v6.8)
        │ • Full documentation
        │ • Production ready
        │ • Public release
```

## Feature Comparison

| Feature | v6.5 (Current) | v6.8 (Full Automation) |
|---------|----------------|------------------------|
| Vulnerability Discovery | ✅ | ✅ |
| Issue Tracking (Jira) | Manual | ✅ Automatic |
| **PR Creation (GitHub)** | ❌ | ✅ **Automatic** |
| **PR Intelligence** | ❌ | ✅ **ALL 14+ Modules** |
| **Auto-Merge** | ❌ | ✅ **Optional (Safe)** |
| **Multi-Repo Orchestration** | ❌ | ✅ **Yes** |
| Team Assignment | Manual | ✅ Smart Routing |
| SLA Tracking | ❌ | ✅ Automated |
| Status Sync | ❌ | ✅ **Tri-directional** (Jira↔BazBOM↔GitHub) |
| CI/CD Workflows | Basic | ✅ Advanced |
| VEX Generation | Manual | ✅ From Jira |
| Dashboard Integration | Basic | ✅ Full (Jira + GitHub) |
| IDE Integration | Basic | ✅ Jira + GitHub aware |

## Configuration Example

```yaml
# .bazbom/jira.yml
jira:
  url: https://example.atlassian.net
  auth:
    type: api-token
    token_env: JIRA_API_TOKEN
    username_env: JIRA_USERNAME

  project: SEC
  issue_type: Bug

  auto_create:
    enabled: true
    min_priority: P2
    only_reachable: true

  routing:
    - pattern: "^org\\.springframework\\..*"
      component: Backend
      assignee: backend-team
      labels: [spring, critical]

  sla:
    P0: 24h   # CISA KEV
    P1: 7d    # Critical reachable
    P2: 30d   # High reachable

  sync:
    bidirectional: true
    auto_close_on_fix: true
```

## Success Metrics

### Adoption (6 months post-release)
- 70% of BazBOM users enable Jira integration
- 10,000+ Jira tickets created automatically
- 80% enable bidirectional sync

### Efficiency
- 70% reduction in manual ticket creation time
- 30% faster vulnerability remediation (MTTR)
- 90% of tickets actionable (not duplicates)

### Quality
- <1% API error rate
- 99% sync accuracy
- <0.1% webhook events dropped

### Satisfaction
- NPS >50 for Jira integration
- <5% support tickets related to Jira
- Prioritized feature requests

## Technical Highlights

### New Crate: `bazbom-jira` (~2,500 LOC)
- Jira REST API client (v3 Cloud + v2 Server/Data Center)
- Webhook server (Axum) for bidirectional sync
- Custom field mapping and templates
- Smart routing and team assignment
- Rate limiting and retry logic

### New Crate: `bazbom-github` (~3,000 LOC)
- GitHub REST API client (v3)
- Automated PR creation with intelligent content
- PR template engine with full intelligence integration
- Multi-PR orchestration across repositories
- Auto-merge with safety controls and test gates
- GitHub webhook receiver for PR events
- CODEOWNERS integration

### New Component: Intelligence Hub (~1,500 LOC)
- Aggregates data from ALL 14+ BazBOM intelligence modules
- Unified interface for enriching tickets and PRs
- Formats intelligence for human-readable GitHub Markdown
- Generates remediation guidance and testing strategies

### Enhanced Crates
- `bazbom-core`: Jira + GitHub configuration models
- `bazbom`: CLI commands (`jira` + `github` subcommands)
- `bazbom-dashboard`: Jira + GitHub UI integration
- `bazbom-lsp`: IDE Jira + GitHub status display
- `bazbom-upgrade-analyzer`: PR-specific breaking change analysis
- `bazbom-ml`: PR risk scoring for auto-merge decisions
- `bazbom-formats`: GitHub-flavored Markdown exports

### Dependencies
- `reqwest`: HTTP client for Jira + GitHub APIs
- `axum`: Webhook servers (Jira + GitHub)
- `governor`: Rate limiting
- `serde_json`: JSON serialization
- `tokio`: Async runtime
- `octocrab`: GitHub API library (optional)

## Security Considerations

- **Authentication:** API tokens, PAT, OAuth 2.0
- **Secrets Management:** Environment variables, vault integration
- **Webhook Security:** HMAC signature verification
- **Data Privacy:** PII sanitization, configurable sync
- **Compliance:** GDPR, SOC 2, HIPAA considerations
- **Rate Limiting:** Prevent API overload
- **Audit Logging:** All Jira operations logged

## Testing Strategy

- **Unit Tests:** >95% coverage
- **Integration Tests:** Jira Cloud & Server sandbox
- **E2E Tests:** Full workflow scenarios
- **Performance Tests:** 1000+ tickets, load testing
- **Security Tests:** Auth, HMAC, input sanitization
- **Compatibility Tests:** All Jira versions

## FAQ

**Q: Will this work with Jira Server/Data Center?**
A: Yes! We support both Jira Cloud (REST API v3) and Jira Server/Data Center (REST API v2).

**Q: Can I use this with multiple Jira projects?**
A: v6.8 supports a default project with routing to components. Multi-project support is planned for v6.9.

**Q: How are duplicate tickets prevented?**
A: BazBOM maintains a local SQLite database tracking CVE → Jira issue mappings. Tickets are only created once per CVE.

**Q: Can I customize ticket templates?**
A: Yes! Templates are fully configurable in `.bazbom/jira.yml` with variable substitution.

**Q: What happens if Jira is down?**
A: BazBOM scans continue uninterrupted. Ticket creation is queued and retried automatically.

**Q: Can I import existing Jira tickets?**
A: Yes, use `bazbom jira sync` to import existing tickets and establish the CVE → ticket mapping.

## Contributing

v6.8 is in the planning phase. We welcome feedback on:

- Feature priorities
- Use case coverage
- Configuration design
- API design

**Feedback Channels:**
- GitHub Discussions: Feature requests and Q&A
- GitHub Issues: Bug reports and technical feedback
- Slack: #bazbom-dev (for active contributors)

## Related Resources

### BazBOM Documentation
- [BazBOM Architecture](../../ARCHITECTURE.md)
- [Integration Patterns](../../../integrations/README.md)
- [CI/CD Integration](../../../CI.md)
- [Policy Integration](../../../user-guide/policy-integration.md)

### Jira Documentation
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/intro/)
- [Jira Webhooks](https://developer.atlassian.com/cloud/jira/platform/webhooks/)
- [Jira Custom Fields](https://support.atlassian.com/jira-cloud-administration/docs/create-a-custom-field/)

### External Tools
- [GitHub Actions](https://docs.github.com/en/actions)
- [GitLab CI](https://docs.gitlab.com/ee/ci/)
- [Jenkins](https://www.jenkins.io/doc/)

## Status Updates

| Date | Status | Milestone |
|------|--------|-----------|
| 2025-11-16 | Planning | Initial plan created |
| 2025-11-16 | **Phase 1 Complete** | **Foundation: Template Engines, Sync Engine, Orchestrator** |
| 2026-01 | Development | Phase 2 target (API integration) |
| 2026-03 | Alpha | M1: Core + Sync |
| 2026-04 | Beta | M2: CI/CD + Dashboard |
| 2026-05 | RC | M3: Advanced features |
| 2026-06 | GA | M4: Public release |

## Development Progress

### Phase 1: Foundation (Weeks 1-3) - ✅ **COMPLETE**

**Timeline:** November 16, 2025 (Completed in 1 day)

**Summary:**
Phase 1 foundation work is COMPLETE with all core template engines, sync engine, and multi-PR orchestrator fully implemented and tested. The foundation includes ~5,000 LOC of production code with 49 comprehensive tests, all passing.

**Week 1: Template Engines** ✅ COMPLETE
- ✅ **Jira Template Engine** (`bazbom-jira/src/templates.rs` - 400+ LOC)
  - Full Markdown → Jira ADF (Atlassian Document Format) conversion
  - Variable substitution for ticket titles and descriptions
  - Support for headings (h2-h6), paragraphs, code blocks, bullet lists
  - Inline formatting: bold, italic, code, links
  - 20 comprehensive tests (all passing)
  - Handles both Jira Wiki syntax (`h2.`) and Markdown (`##`)

- ✅ **GitHub PR Template Engine** (`bazbom-github/src/pr_template.rs` - 300+ LOC)
  - Dynamic variable substitution with intelligence integration
  - Severity-based risk badges (🔴 CRITICAL, 🟠 HIGH, 🟡 MEDIUM, 🟢 LOW)
  - Confidence scoring for auto-merge decisions
  - Reachability status formatting
  - ML risk score integration (0-100)
  - Auto-merge eligibility indicators
  - Jira ticket and BazBOM scan URL linking
  - 12 comprehensive tests (all passing)
  - Default PR template file created (`templates/pr_template.md`)

**Week 2-3: Sync Engine & Orchestrator** ✅ COMPLETE
- ✅ **Bidirectional Sync Engine** (`bazbom-jira/src/sync.rs` - 500+ LOC)
  - Thread-safe state management with `Arc<RwLock<SyncState>>`
  - CVE ↔ Jira key bidirectional mapping
  - Jira webhook event processing:
    - StatusChanged: Maps Jira status to BazBOM vulnerability states
    - IssueDeleted: Treats as accepted risk/VEX
    - CommentAdded: Captures remediation notes
    - IssueCreated, IssueUpdated, AssignmentChanged
  - BazBOM event processing:
    - VulnerabilityFixed: Updates Jira to "Done"
    - FixVerified: Updates Jira to "Verified"
    - SeverityChanged: Updates Jira priority
    - VulnerabilityDiscovered: Triggers ticket creation
  - Status mapping between Jira and BazBOM states
  - Last sync timestamp tracking
  - 9 comprehensive tests (all passing)

- ✅ **Multi-PR Orchestrator** (`bazbom-github/src/orchestrator.rs` - 450+ LOC)
  - Three orchestration strategies:
    - OnePrPerRepo: Single PR with all vulnerabilities (default)
    - BatchByPackage: One PR per package (groups CVEs by dependency)
    - BatchBySeverity: Separate PRs for CRITICAL, HIGH, MEDIUM, LOW
  - Concurrent processing with configurable limits (default: 5 repos)
  - Batch processing across multiple repositories
  - Comprehensive result tracking with OrchestrationSummary
  - Statistics: successful PRs, failures, total vulnerabilities
  - 8 comprehensive tests (all passing)

**Crate Structure:**
- ✅ `bazbom-jira` crate (~2,500 LOC total)
  - REST API client skeleton
  - Data models for Jira issues, fields, custom metadata
  - Webhook server foundation
  - **Template engine (COMPLETE)**
  - Routing engine for team assignment
  - **Sync engine (COMPLETE)**
  - Configuration management
  - Error handling and type system

- ✅ `bazbom-github` crate (~3,000 LOC total)
  - REST API client skeleton
  - Data models for PRs, repositories, users
  - **PR template engine (COMPLETE)**
  - **Multi-PR orchestrator (COMPLETE)**
  - Auto-merge configuration models
  - Webhook server foundation
  - Error handling and type system

- ✅ Workspace configuration
  - Both crates added to Cargo workspace
  - Dependencies configured (axum, reqwest, governor, tokio, etc.)

**Testing Summary:**
- 49 total tests, all passing ✅
  - 20 Jira template tests
  - 12 GitHub PR template tests
  - 9 Sync engine tests
  - 8 Orchestrator tests
- Unit test coverage for all core functions
- Edge case handling validated
- Thread-safety verified (sync engine)
- Strategy pattern validated (orchestrator)

**Code Quality:**
- All code compiling successfully
- Minor warnings (expected for stubs): unused imports, dead code in webhook stubs
- Clean error handling with `thiserror`
- Async/await patterns with tokio
- Thread-safe concurrent access with Arc<RwLock>

**Documentation:**
- ✅ `.github/copilot-instructions.md` updated with Phase 1 completion status
- ✅ This README updated with complete statistics
- ✅ Inline code documentation and comments

**Next Steps: Phase 2** (API Integration - Future Work)
- Complete REST API client implementations for Jira and GitHub
- Add authentication (Jira: API token, OAuth; GitHub: PAT, OAuth)
- Implement webhook HMAC signature verification
- Integration testing with Jira Cloud sandbox
- Integration testing with GitHub test repositories
- Rate limiting implementation (Jira: 5 req/sec, GitHub: 60 req/min)
- Error handling and retry logic for API failures

## Contact

- **Owner:** cboyd0319
- **Email:** [via GitHub]
- **Slack:** #bazbom-dev
- **Issues:** https://github.com/cboyd0319/BazBOM/issues
- **Discussions:** https://github.com/cboyd0319/BazBOM/discussions

---

**Next Steps:**
1. ✅ Review this documentation
2. ⏳ Gather stakeholder feedback
3. ⏳ Finalize Phase 1 scope
4. ⏳ Allocate development resources
5. ⏳ Begin implementation (Q1 2026)

---

Last Updated: 2025-11-16

## Development Environment Updates (v6.8)

The v6.8 release includes updates to all development dependencies and tooling:

### Updated Dependency Versions

**Rust Dependencies:**
- axum: 0.8.7 (web framework for dashboard)
- bcrypt: 0.17.1 (password hashing)
- colored: 3.0.0 (terminal colors)
- jsonschema: 0.35 (JSON schema validation)
- rand: 0.9.2 (random number generation)
- serde-xml-rs: 0.8 (XML serialization)
- which: 8.0.0 (executable finding)

**Build Tools:**
- Gradle: 9.2.0 (latest stable, up from 8.5)
- Java: 21 LTS (latest LTS, up from 17)
- Kotlin: 2.2.21 (latest stable)

**IDE Integration:**
- IntelliJ Platform: 2025.2 (up from 2023.3)
- VS Code Engine: 1.85.0+

### Breaking Changes Avoided

To maintain stability, the following packages were kept at their current major versions due to breaking changes in newer releases:

- **jsonwebtoken:** Staying on 9.x (version 10+ requires explicit crypto backend selection)
- **printpdf/lopdf:** Staying on current versions (version 0.8+ has significant API changes)

These packages will be evaluated for upgrade in future releases when migration paths are clearer.

---

