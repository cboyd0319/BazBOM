# BazBOM v6.8 - Jira Integration Implementation Roadmap

**Version:** 6.8
**Timeline:** 20 weeks (Q1-Q2 2025)
**Status:** Planning

## Overview

This roadmap outlines the implementation plan for comprehensive Jira bidirectional integration in BazBOM v6.8. The project is divided into 7 phases over 20 weeks, with each phase delivering incrementally valuable features.

---

## Timeline

```
Q1 2025                                    Q2 2025
┌────────────┬────────────┬────────────┬────────────┬────────────┐
│  Weeks 1-3 │  Weeks 4-6 │  Weeks 7-9 │ Weeks 10-12│ Weeks 13-15│
│  Phase 1   │  Phase 2   │  Phase 3   │  Phase 4   │  Phase 5   │
│ Foundation │Auto-Create │ Bi-Sync    │  CI/CD     │ Dashboard  │
└────────────┴────────────┴────────────┴────────────┴────────────┘
               Weeks 16-18│ Weeks 19-20│
               Phase 6    │  Phase 7   │
               Advanced   │  Testing   │
               └───────────┴────────────┘
```

---

## Phase 1: Foundation (Weeks 1-3)

### Goals
- Establish core Jira integration infrastructure
- Create `bazbom-jira` crate with REST API client
- Implement authentication and basic CRUD operations
- CLI commands for manual ticket management

### Deliverables

**Week 1:**
- [ ] Create `crates/bazbom-jira/` directory structure
- [ ] Define data models (`models.rs`)
- [ ] Implement Jira REST API client skeleton (`client.rs`)
- [ ] Authentication: API token, PAT, OAuth 2.0
- [ ] Unit tests with `wiremock`

**Week 2:**
- [ ] Implement issue CRUD operations:
  - `create_issue()`
  - `get_issue()`
  - `update_issue()`
  - `add_comment()`
- [ ] Rate limiting with `governor` crate
- [ ] Retry logic with exponential backoff
- [ ] Error handling (`error.rs`)

**Week 3:**
- [ ] CLI command: `bazbom jira init` (interactive setup)
- [ ] CLI command: `bazbom jira create` (manual ticket creation)
- [ ] CLI command: `bazbom jira get` (fetch ticket details)
- [ ] Configuration file support (`.bazbom/jira.yml`)
- [ ] Documentation: Quick start guide
- [ ] Integration tests with Jira Cloud sandbox

**Dependencies:**
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
reqwest-middleware = "0.4"
reqwest-retry = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
governor = "0.7"
```

**Success Criteria:**
- ✅ Can authenticate to Jira Cloud and Server
- ✅ Can create, read, and update issues via CLI
- ✅ Rate limiting prevents API overload
- ✅ All unit tests passing (>90% coverage)

---

## Phase 2: Automatic Ticket Creation (Weeks 4-6)

### Goals
- Auto-create Jira tickets during BazBOM scans
- Template-based ticket formatting with custom fields
- Component-based routing and team assignment
- Bulk operations for large vulnerability sets

### Deliverables

**Week 4:**
- [ ] Ticket template engine (`templates.rs`)
  - Markdown → Jira Atlassian Document Format (ADF)
  - Variable substitution (`{cve_id}`, `{package}`, etc.)
  - Support for headings, lists, code blocks, links
- [ ] Custom field mapping configuration
- [ ] Priority mapping (P0-P4 → Jira priorities)

**Week 5:**
- [ ] Component-based routing (`routing.rs`)
  - Regex pattern matching for package names
  - Team/component assignment rules
  - Label auto-tagging
- [ ] CLI flag: `bazbom scan --jira-create`
- [ ] Bulk issue creation (up to 50 issues per batch)
- [ ] Duplicate detection (avoid re-creating tickets)

**Week 6:**
- [ ] Integration with BazBOM policy engine
  - Only create tickets for policy violations
  - Configurable severity thresholds
  - Reachability filter (only reachable CVEs)
- [ ] Dry-run mode: `--jira-dry-run`
- [ ] Progress indicators for bulk operations
- [ ] Documentation: Automatic ticket creation guide

**Configuration Example:**
```yaml
jira:
  auto_create:
    enabled: true
    min_priority: P2
    only_reachable: true

  routing:
    - pattern: "^org\\.springframework\\..*"
      component: Backend
      assignee: backend-team
      labels: [spring, critical]
```

**Success Criteria:**
- ✅ `bazbom scan --jira-create` creates tickets automatically
- ✅ Correct routing based on package patterns
- ✅ Custom fields populated correctly
- ✅ Bulk operations handle 100+ vulnerabilities efficiently (<5 min)

---

## Phase 3: Bidirectional Synchronization (Weeks 7-9)

### Goals
- Webhook receiver for Jira event notifications
- Sync Jira updates back to BazBOM database
- Auto-close tickets when vulnerabilities are fixed
- VEX generation from rejected Jira tickets

### Deliverables

**Week 7:**
- [ ] Webhook server implementation (`webhook.rs`)
  - Axum-based HTTP server
  - HMAC signature verification
  - Event parsing (issue_updated, comment_created)
- [ ] SQLite database for tracking Jira tickets
  - Table: `jira_issues`
  - Table: `jira_sync_log`
- [ ] CLI command: `bazbom jira webhook-server`

**Week 8:**
- [ ] Bidirectional sync engine (`sync.rs`)
  - Import Jira status updates to BazBOM
  - Detect fixed vulnerabilities and close tickets
  - Respect manual ticket closures
- [ ] CLI command: `bazbom jira sync` (manual sync trigger)
- [ ] Auto-close logic:
  - Re-scan after fix
  - Compare CVE lists
  - Transition ticket to "Done"
  - Add comment with fix confirmation

**Week 9:**
- [ ] VEX generation from rejected tickets
  - Query Jira for status="Rejected"
  - Extract justification from comments
  - Generate CSAF VEX entries
  - Link VEX to Jira ticket URL
- [ ] CLI command: `bazbom jira export-vex`
- [ ] Documentation: Bidirectional sync guide

**Webhook Events Supported:**
- `jira:issue_updated` → Sync status, priority, assignee
- `comment_created` → Parse remediation notes
- `worklog_updated` → Track time spent

**Success Criteria:**
- ✅ Webhook server receives and processes events correctly
- ✅ Status changes in Jira reflect in BazBOM dashboard
- ✅ Fixed vulnerabilities auto-close tickets
- ✅ VEX entries generated with Jira references

---

## Phase 4: CI/CD Integration (Weeks 10-12)

### Goals
- Seamless integration with GitHub Actions, GitLab CI, Jenkins
- Auto-comment on PRs with Jira ticket links
- Workflow automation (PR → Jira transition → PR merge → Close)

### Deliverables

**Week 10:**
- [ ] GitHub Actions integration
  - Action input: `jira-auto-create`
  - Action input: `jira-project`
  - Action input: `jira-epic`
  - Auto-comment on PRs with ticket links
- [ ] Example workflows for common scenarios
  - PR scan with ticket creation
  - Main branch scan with ticket updates
  - Container scan with layer-based tickets

**Week 11:**
- [ ] GitLab CI integration
  - `.gitlab-ci.yml` templates
  - Auto-link merge requests to Jira tickets
  - Transition tickets on MR merge
- [ ] Jenkins integration examples
  - Jenkinsfile templates
  - Post-build Jira updates
  - Integration with Jira plugin

**Week 12:**
- [ ] PR workflow automation
  - Detect PR context (GitHub/GitLab)
  - Create tickets linked to PR
  - Transition on PR events (opened → in_progress, merged → done)
- [ ] Commit message parsing (e.g., "Closes SEC-567")
- [ ] Documentation: CI/CD integration guide

**GitHub Actions Example:**
```yaml
- uses: cboyd0319/BazBOM@main
  with:
    jira-auto-create: true
    jira-project: SEC
    jira-comment-on-pr: true
```

**Success Criteria:**
- ✅ GitHub Actions workflow creates tickets automatically
- ✅ PR comments include Jira ticket links
- ✅ Ticket transitions tracked through PR lifecycle
- ✅ Works on GitLab and Jenkins with minimal config

---

## Phase 5: Dashboard & Reporting (Weeks 13-15)

### Goals
- Jira integration in BazBOM web dashboard
- Enhanced HTML/PDF reports with Jira metadata
- IDE plugin updates (IntelliJ, VS Code)
- Jira dashboard widgets

### Deliverables

**Week 13:**
- [ ] BazBOM dashboard UI enhancements
  - Embedded Jira ticket cards
  - "Create Jira Ticket" button per CVE
  - Inline ticket status badges
  - Quick transitions (To Do → In Progress → Done)
- [ ] Dashboard API endpoints:
  - `GET /api/jira/tickets?cve={cve_id}`
  - `POST /api/jira/tickets` (create)
  - `PUT /api/jira/tickets/{key}` (update)

**Week 14:**
- [ ] Enhanced reporting
  - HTML reports: Clickable Jira ticket links
  - PDF reports: Jira ticket references
  - CSV exports: Jira issue keys column
  - SARIF: Custom property for Jira tracking
- [ ] CLI flag: `bazbom report --jira-links`
- [ ] Report templates with Jira metadata

**Week 15:**
- [ ] IDE plugin updates
  - **IntelliJ:** Show Jira ticket status in tooltips
  - **VS Code:** "Open in Jira" code action
  - LSP: Hover info includes Jira links
- [ ] Jira dashboard gadget (optional)
  - Gadget showing BazBOM vulnerability stats
  - Embed in Jira dashboards
- [ ] Documentation: Dashboard and reporting guide

**Success Criteria:**
- ✅ Dashboard displays Jira ticket status for each CVE
- ✅ One-click ticket creation from dashboard
- ✅ Reports include Jira references
- ✅ IDE plugins show Jira context

---

## Phase 6: Advanced Features (Weeks 16-18)

### Goals
- SLA tracking and automation
- Sprint planning integration
- Advanced routing rules
- Performance optimizations

### Deliverables

**Week 16:**
- [ ] SLA configuration and tracking
  - Configurable SLA per priority (P0: 24h, P1: 7d, etc.)
  - Auto-set Jira due dates
  - SLA breach alerts
  - SLA compliance metrics
- [ ] CLI command: `bazbom jira sla-report`

**Week 17:**
- [ ] Sprint integration
  - Auto-add tickets to current sprint
  - Epic linking for themed remediation
  - Story point estimation (based on remediation effort)
  - CLI command: `bazbom jira add-to-sprint`
  - CLI command: `bazbom jira create-epic`

**Week 18:**
- [ ] Advanced routing and assignment
  - CODEOWNERS file integration
  - Round-robin assignment
  - Escalation rules (KEV → security team)
  - Fallback assignee configuration
- [ ] Performance optimizations
  - Caching of Jira metadata
  - Batching of API calls
  - Async processing queues
  - Connection pooling

**Configuration Example:**
```yaml
jira:
  sla:
    P0: 24h
    P1: 7d
    P2: 30d

  sprint:
    auto_add: true
    current_sprint_jql: "sprint in openSprints()"
```

**Success Criteria:**
- ✅ SLA tracking automated with due dates
- ✅ Tickets added to sprints automatically
- ✅ Smart assignment based on CODEOWNERS
- ✅ Bulk operations optimized (<3 min for 500 tickets)

---

## Phase 7: Testing & Documentation (Weeks 19-20)

### Goals
- Comprehensive testing (unit, integration, E2E)
- Performance and load testing
- Complete user and developer documentation
- Migration guides

### Deliverables

**Week 19:**
- [ ] Testing
  - Unit test coverage >95%
  - Integration tests with Jira Cloud sandbox
  - Integration tests with Jira Server/Data Center
  - E2E tests for all workflows
  - Load testing: 1000 tickets creation
  - Performance profiling and optimization

**Week 20:**
- [ ] Documentation
  - User guide: Getting started with Jira integration
  - User guide: Advanced features and workflows
  - Developer guide: Extending Jira integration
  - API reference documentation
  - Troubleshooting guide
  - FAQ
- [ ] Migration guides
  - Migrating from manual workflow
  - Importing existing Jira tickets
  - Custom field setup wizard
- [ ] Video tutorials (optional)
  - 5-minute quick start
  - 15-minute deep dive
  - Use case walkthroughs

**Testing Checklist:**
- [ ] ✅ Unit tests (models, client, templates, routing)
- [ ] ✅ Integration tests (Jira API, webhook server, sync engine)
- [ ] ✅ E2E tests (scan → create → update → close)
- [ ] ✅ Performance tests (bulk operations, rate limiting)
- [ ] ✅ Security tests (auth, HMAC verification, input sanitization)
- [ ] ✅ Compatibility tests (Jira Cloud, Server, Data Center)

**Documentation Checklist:**
- [ ] ✅ README with overview and quick start
- [ ] ✅ Installation and setup guide
- [ ] ✅ Configuration reference
- [ ] ✅ CLI command reference
- [ ] ✅ API documentation (Rust docs)
- [ ] ✅ Use case examples
- [ ] ✅ Troubleshooting guide
- [ ] ✅ Migration guide

**Success Criteria:**
- ✅ All tests passing (700+ tests)
- ✅ Performance targets met (see technical specs)
- ✅ Complete documentation published
- ✅ Ready for beta release

---

## Milestones

### M1: Alpha Release (End of Phase 3, Week 9)
- Core functionality: Create, update, sync tickets
- Webhook support
- VEX generation
- Limited to internal testing

### M2: Beta Release (End of Phase 5, Week 15)
- Full feature set except advanced features
- Dashboard integration
- CI/CD templates
- Open to early adopters

### M3: Release Candidate (End of Phase 6, Week 18)
- All features implemented
- Performance optimizations complete
- Ready for QA and user acceptance testing

### M4: General Availability (End of Phase 7, Week 20)
- Full v6.8 release
- Complete documentation
- Production-ready
- Public announcement

---

## Resource Requirements

### Team

- **Lead Developer:** Full-time (20 weeks)
- **Backend Developer:** Full-time (Weeks 1-12)
- **Frontend Developer:** Part-time (Weeks 13-15)
- **QA Engineer:** Part-time (Weeks 7-20)
- **Technical Writer:** Part-time (Weeks 16-20)

### Infrastructure

- **Jira Cloud Sandbox:** For testing (free tier)
- **Jira Server Trial:** For compatibility testing
- **CI/CD Credits:** GitHub Actions, GitLab CI
- **Test VMs:** For multi-platform testing

### External Dependencies

- Jira REST API access (documented, stable)
- Jira Cloud free tier (testing)
- Jira Marketplace approval (for gadget, optional)

---

## Risk Management

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Jira API changes | High | Version pinning, fallback to v2 API |
| Rate limiting issues | Medium | Batching, caching, exponential backoff |
| Webhook reliability | Medium | Retry mechanism, manual sync fallback |
| Custom field complexity | Medium | Configuration wizard, validation |

### Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep | High | Strict phase gates, MVP focus |
| Testing delays | Medium | Start testing early (Phase 3) |
| Documentation lag | Low | Continuous documentation updates |

### Operational Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| User adoption | Medium | Early beta, gather feedback |
| Support burden | Medium | Comprehensive docs, FAQ |
| Compatibility issues | Low | Test all Jira versions |

---

## Success Metrics

### Phase 1-3 (Foundation + Sync)
- **Code Coverage:** >90%
- **API Error Rate:** <2%
- **Performance:** Ticket creation <2s (p95)

### Phase 4-5 (CI/CD + Dashboard)
- **CI Integration:** Works on GitHub, GitLab, Jenkins
- **Dashboard Load Time:** <3s
- **Report Generation:** <5s with Jira links

### Phase 6-7 (Advanced + Testing)
- **Test Coverage:** >95%
- **Load Test:** 1000 tickets in <10 min
- **Documentation:** 100% feature coverage

### Post-Release (v6.8 GA)
- **Adoption:** 50% of users enable Jira integration (6 months)
- **User Satisfaction:** NPS >40
- **Support Tickets:** <5% related to Jira integration

---

## Dependencies on Other Work

### BazBOM Core
- Stable policy engine (v6.5+)
- Reachability analysis API (v6.5+)
- Dashboard API (v6.5+)

### External Systems
- Jira REST API v3 (Cloud)
- Jira REST API v2 (Server/Data Center)
- GitHub API v3 (for PR comments)
- GitLab API v4 (for MR integration)

### Tools & Libraries
- `reqwest` for HTTP client
- `axum` for webhook server
- `serde_json` for JSON (de)serialization
- `governor` for rate limiting

---

## Review & Approval

### Phase Gates

Each phase requires approval before proceeding:

1. **Phase 1 → 2:** Core API client functional, tests passing
2. **Phase 2 → 3:** Auto-create working, routing correct
3. **Phase 3 → 4:** Bidirectional sync verified, VEX generation tested
4. **Phase 4 → 5:** CI/CD templates validated on all platforms
5. **Phase 5 → 6:** Dashboard complete, reports enhanced
6. **Phase 6 → 7:** Advanced features working, performance acceptable
7. **Phase 7 → GA:** All tests passing, docs complete, stakeholder sign-off

### Stakeholder Reviews

- **Weekly:** Development team sync
- **Bi-weekly:** Product owner review
- **Monthly:** Security and compliance review
- **End of Phase:** Stakeholder demo and approval

---

## Post-Release Plan

### v6.8.1 (Patch Release, +2 weeks)
- Bug fixes from user feedback
- Performance tuning
- Documentation updates

### v6.9 (Future Enhancements, +12 weeks)
- Advanced JQL query builder in dashboard
- Multi-project support (route to different projects)
- Jira Service Management integration
- Advanced analytics and dashboards
- AI-powered ticket triage

---

## Appendices

### A. Implementation Checklist

See [implementation-checklist.md](implementation-checklist.md) (to be created)

### B. Test Plan

See [test-plan.md](test-plan.md) (to be created)

### C. API Design Review

See [technical-specifications.md](technical-specifications.md)

---

**Next Steps:**
1. Review and approve roadmap
2. Allocate resources (developers, QA, tech writer)
3. Set up Jira Cloud sandbox for testing
4. Kick off Phase 1 (Weeks 1-3)

---

**Document Owner:** cboyd0319
**Last Updated:** 2025-11-16
**Status:** Draft - Pending Approval
