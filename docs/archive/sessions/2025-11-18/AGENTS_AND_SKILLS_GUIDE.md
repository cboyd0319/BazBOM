---
name: agents-and-skills-guide
description: Complete guide to BazBOM's Claude Code subagents and skills - what they are, when to use them, and how they improve development workflow
---

# BazBOM Agents and Skills Guide

Comprehensive guide to the automated assistants and capabilities available in the BazBOM project.

---

## Overview

**Subagents** are specialized AI assistants that handle specific tasks with their own context windows.
**Skills** are automated capabilities that Claude invokes based on your needs.

Together, they make BazBOM development faster, more consistent, and less error-prone.

---

## Project Subagents

Location: `.claude/agents/`

### 1. Bazel Expert (`bazel-expert.md`)

**Purpose:** Deep expertise in Bazel build system integration

**Use when:**
- Debugging Bazel dependency detection
- Adding Bazel features
- Investigating "0 packages detected" issues
- Understanding maven_install.json parsing

**Expertise:**
- `bazel.rs` internals (lines 104-285)
- Build system detection
- Both scan paths (legacy + orchestrator)
- maven_install.json structure

**Example invocation:**
```
"Use the bazel-expert agent to investigate why maven_install.json isn't being parsed"
```

**Key capabilities:**
- ✅ Traces Bazel detection flow
- ✅ Validates maven_install.json structure
- ✅ Ensures both scan paths are consistent
- ✅ Provides specific file/line references
- ✅ Tests on real repositories

---

### 2. Test Runner (`test-runner.md`)

**Purpose:** Automated testing and validation specialist

**Use when:**
- Running comprehensive test suites
- Validating fixes across multiple repos
- Checking for regressions
- Performance testing

**Capabilities:**
- Rust unit tests (cargo test)
- Integration tests (test scripts)
- Manual validation
- Performance profiling
- Result reporting

**Example invocation:**
```
"Use the test-runner agent to validate this fix against all test repositories"
```

**Test coverage:**
- ✅ 45+ Rust unit tests
- ✅ 5 integration test repos
- ✅ Performance benchmarks
- ✅ Regression detection

---

### 3. Code Reviewer (`code-reviewer.md`)

**Purpose:** Rust code quality and BazBOM pattern enforcement

**Use when:**
- Reviewing pull requests
- Checking code quality
- Ensuring pattern consistency
- Security auditing

**Review checklist:**
- Rust best practices
- BazBOM coding standards
- Security vulnerabilities
- Test coverage
- Documentation quality

**Example invocation:**
```
"Use the code-reviewer agent to review my changes"
```

**Focus areas:**
- ✅ Proper error handling
- ✅ Logging standards (tracing not eprintln)
- ✅ Both scan paths updated
- ✅ Security vulnerabilities
- ✅ Test completeness

---

### 4. Reachability Expert (`reachability-expert.md`)

**Purpose:** 7-language reachability analysis specialist (BazBOM's 70-90% noise reduction killer feature)

**Use when:**
- Debugging reachability false positives/negatives
- Investigating why vulnerabilities are marked reachable/unreachable
- Adding framework support (Express, Flask, Django, Rails, etc.)
- Understanding language-specific accuracy limitations

**Language coverage:**
- Rust (>98% accuracy) - syn parser, trait tracking
- Go (~90%) - tree-sitter, reflection, goroutines
- JVM (~85%) - OPAL bytecode analysis
- JavaScript/TypeScript (~85%) - SWC AST, dynamic code
- Python (~80%) - RustPython, framework detection
- Ruby (~75%) - Rails/RSpec, metaprogramming
- PHP (~70%) - Laravel/Symfony, variable functions

**Example invocation:**
```
"Use reachability-expert to investigate why this CVE is marked unreachable"
"Have reachability-expert analyze false positive rate for Python"
```

**Key capabilities:**
- ✅ Entrypoint detection (main, routes, tests)
- ✅ Call graph construction
- ✅ Dynamic code handling (eval, reflection)
- ✅ Framework-aware analysis
- ✅ Conservative over-approximation

---

### 5. Container Expert (`container-expert.md`)

**Purpose:** Container scanning with layer attribution and multi-language reachability

**Use when:**
- Debugging container scans
- Investigating layer attribution issues
- Understanding P0-P4 prioritization
- Adding support for new base images

**Container features:**
- Layer attribution (maps vulns to Dockerfile layers)
- EPSS/KEV enrichment
- P0-P4 intelligent prioritization
- Quick wins analysis
- Baseline comparison
- Multi-language remediation (copy-paste Dockerfile fixes)

**Example invocation:**
```
"Use container-expert to debug why layer attribution is missing"
"Have container-expert explain P0 prioritization for this container"
```

**Key capabilities:**
- ✅ OCI image analysis
- ✅ Dockerfile layer mapping
- ✅ Container reachability (6 languages)
- ✅ EPSS/KEV real-time data
- ✅ Exploit intelligence integration

---

### 6. Security Analyst (`security-analyst.md`)

**Purpose:** Vulnerability enrichment, threat intelligence, policy enforcement, compliance specialist

**Use when:**
- Debugging EPSS/KEV integration
- Investigating policy violations
- Generating compliance reports (7 frameworks)
- Understanding threat intelligence (malicious packages, typosquatting)

**Security coverage:**
- OSV, NVD, GHSA, CISA KEV advisory sources
- EPSS exploit prediction scoring
- Malicious package detection
- Typosquatting alerts (Levenshtein distance)
- Policy engines (Rego/YAML/CUE)
- Compliance frameworks (PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST)

**Example invocation:**
```
"Use security-analyst to explain why this CVE is P0"
"Have security-analyst generate PCI-DSS compliance report"
```

**Key capabilities:**
- ✅ CVSS 3.1 scoring
- ✅ EPSS probability analysis
- ✅ KEV catalog integration
- ✅ Policy rule evaluation
- ✅ Audit-ready reports

---

### 7. Polyglot Expert (`polyglot-expert.md`)

**Purpose:** Multi-language/multi-build-system specialist (13 systems, unified SBOM)

**Use when:**
- Debugging lockfile parsing (npm, Yarn, pnpm, Poetry, Pipenv, etc.)
- Ecosystem detection issues
- Workspace/monorepo problems
- Universal auto-fix across package managers

**Ecosystem coverage:**
- JVM: Maven, Gradle, SBT, Ant+Ivy, Buildr, Android
- JavaScript: npm, Yarn, pnpm (full lockfile parsing)
- Python: pip, Poetry, Pipenv
- Go: go.mod/go.sum
- Rust: Cargo
- Ruby: Bundler
- PHP: Composer

**Example invocation:**
```
"Use polyglot-expert to debug why npm packages aren't detected"
"Have polyglot-expert investigate workspace resolution"
```

**Key capabilities:**
- ✅ Auto-detection (zero config)
- ✅ Lockfile parsing (all formats)
- ✅ Unified SBOM generation
- ✅ Universal auto-fix (9 package managers)
- ✅ Workspace/monorepo support

---

### 8. Upgrade Intelligence Expert (`upgrade-intelligence-expert.md`)

**Purpose:** Recursive transitive breaking change analysis and upgrade impact prediction

**Use when:**
- Investigating upgrade impacts
- Debugging breaking change detection
- Understanding effort estimates (0-100 score)
- Adding support for new package ecosystems

**Upgrade intelligence:**
- Recursive transitive dependency analysis
- Breaking change detection (GitHub releases, semver, bytecode)
- JAR bytecode comparison (JVM packages)
- Config migration detection (Spring Boot, Log4j)
- Community upgrade data
- Effort estimation (hours-based, not vague)

**Example invocation:**
```
"Use upgrade-intelligence-expert to analyze Spring Boot 2→3 upgrade impact"
"Have upgrade-intelligence-expert explain why effort estimate is 6 hours"
```

**Key capabilities:**
- ✅ Multi-source intelligence (deps.dev, GitHub, bytecode)
- ✅ Transitive breaking changes
- ✅ Migration guide discovery
- ✅ Multi-CVE grouping
- ✅ Effort scoring (0-100)

---

## Skills

Location: `.claude/skills/`

### 1. SBOM Validator (`sbom-validator`)

**Purpose:** Validates generated SBOM files for correctness

**Automatically activates when you ask:**
- "Is this SBOM valid?"
- "Check the generated SBOM"
- "How many packages in the SBOM?"

**Validation checks:**
- ✅ JSON format validity
- ✅ SPDX structure compliance
- ✅ Package completeness
- ✅ Relationship integrity
- ✅ Content quality

**Quick commands:**
```bash
# Count packages
jq '.packages | length' sbom.spdx.json

# Validate structure
jq -e '.spdxVersion' sbom.spdx.json

# Check PURLs
jq -r '.packages[].externalRefs[] | select(.referenceType == "purl")' sbom.spdx.json
```

**Success criteria:**
- All required SPDX fields present
- Package count > 0 (for repos with deps)
- All packages have names, versions, PURLs
- Relationships reference existing packages

---

### 2. Performance Profiler (`performance-profiler`)

**Purpose:** Analyzes scan performance and identifies bottlenecks

**Automatically activates when you ask:**
- "Why is this scan slow?"
- "Performance test needed"
- "How much memory is this using?"
- "Optimize the scan"

**Performance metrics:**
```
Small repos:  <1s, ~50MB
Medium repos: 1-3s, ~100MB
Large repos:  3-10s, ~150MB
Huge repos:   10-30s, ~200MB
```

**Profiling commands:**
```bash
# Basic timing
/usr/bin/time -l bazbom scan .

# Detailed profiling
RUST_LOG=debug /usr/bin/time -l bazbom scan .

# Progressive testing
for limit in 10 50 100; do
    /usr/bin/time -l bazbom scan --limit $limit .
done
```

**Bottleneck detection:**
- maven_install.json parsing
- SBOM generation
- Polyglot scanning
- Reachability analysis

---

### 3. Reachability Validator (`reachability-validator`)

**Purpose:** Validates reachability analysis accuracy and identifies common issues

**Automatically activates when you ask:**
- "Is this reachability analysis correct?"
- "Validate call graph"
- "Check reachability accuracy"
- "Why is this marked unreachable?"
- "Reachability false positive/negative?"

**Validation checks:**
- ✅ Entrypoint detection completeness
- ✅ Call graph construction correctness
- ✅ Noise reduction within expected range (45-90% by language)
- ✅ Dynamic code handling (eval, reflection)
- ✅ Framework pattern detection
- ✅ False negative risk assessment

**Expected reduction rates by language:**
- Rust: 70-90% (highly static)
- Go: 60-85%
- JavaScript/TypeScript: 60-80%
- Python: 55-75%
- Ruby: 50-70%
- PHP: 45-65%

---

### 4. Vulnerability Reporter (`vulnerability-reporter`)

**Purpose:** Provides deep-dive CVE analysis with actionable remediation guidance

**Automatically activates when you ask:**
- "Explain this CVE"
- "Why is this P0/P1/P2?"
- "Show exploit details"
- "What is EPSS score for CVE-X?"
- "Is this actively exploited?"
- "How do I fix CVE-X?"

**Deep-dive analysis includes:**
- ✅ CVSS 3.1 scoring breakdown
- ✅ EPSS probability (0.0-1.0)
- ✅ CISA KEV status
- ✅ Exploit availability (ExploitDB, Metasploit, GitHub POCs, Nuclei)
- ✅ Prioritization rationale (P0-P4 logic)
- ✅ Remediation guidance (copy-paste commands)
- ✅ Effort estimation
- ✅ Migration guides

**P0-P4 Criteria:**
- P0: CVSS ≥8.0 AND (KEV=true OR EPSS≥0.7) - Fix <24h
- P1: CVSS ≥7.0 AND EPSS≥0.3 - Fix <1 week
- P2: CVSS ≥7.0 AND EPSS<0.3 - Fix <30 days
- P3: CVSS 4.0-6.9 - Fix <90 days
- P4: CVSS <4.0 - Informational

---

### 5. Compliance Checker (`compliance-checker`)

**Purpose:** Validates compliance against 7 security/regulatory frameworks

**Automatically activates when you ask:**
- "Check PCI-DSS compliance"
- "Generate HIPAA report"
- "Validate policy"
- "Are we SOC2 compliant?"
- "FedRAMP compliance status"

**Supported frameworks:**
- PCI-DSS 3.2.1 (Payment Card Industry)
- HIPAA Security Rule (Healthcare)
- FedRAMP Moderate (Federal)
- SOC 2 Type II (Trust Services)
- GDPR Article 32 (EU Privacy)
- ISO 27001:2013 (Info Security)
- NIST Cybersecurity Framework

**Compliance validation:**
- ✅ Requirement mapping to BazBOM checks
- ✅ Pass/fail criteria per requirement
- ✅ Evidence documentation
- ✅ Remediation action items
- ✅ Audit-ready reports (HTML)
- ✅ Policy rule evaluation

**Example checks:**
- PCI-DSS 6.2: 0 critical vulns, high vulns <30 days
- HIPAA 164.308: Risk analysis completed (vulnerability scan)
- SOC2 CC7.1: Security monitoring (continuous scanning)

---

## How They Work Together

### Example Workflow: Fixing a Bug

1. **Investigation** - Bazel Expert agent analyzes the issue
   ```
   "Use bazel-expert to investigate why packages aren't being detected"
   ```

2. **Implementation** - Main Claude implements the fix
   ```
   (You make code changes)
   ```

3. **Code Review** - Code Reviewer checks quality
   ```
   "Have code-reviewer review my changes"
   ```

4. **Testing** - Test Runner validates across repos
   ```
   "Use test-runner to validate this fix"
   ```

5. **Validation** - SBOM Validator checks output
   ```
   (Automatically activates when checking SBOM)
   "Is the generated SBOM valid?"
   ```

6. **Performance** - Performance Profiler ensures no regression
   ```
   (Automatically activates on performance questions)
   "Is this faster than before?"
   ```

---

## Usage Patterns

### Explicit Agent Invocation
```
"Use the [agent-name] agent to [task]"
"Have [agent-name] investigate [issue]"
"Let [agent-name] handle [work]"
```

### Automatic Skill Activation

Skills activate based on your questions:
- Ask about SBOM validity → sbom-validator activates
- Ask about performance → performance-profiler activates

### Chaining Agents
```
"First use bazel-expert to analyze the issue,
then use test-runner to validate the fix"
```

---

## Best Practices

### When to Use Subagents

✅ **Use subagents when:**
- Task requires deep domain expertise
- Need isolated context for complex work
- Want specialized tool access
- Following established workflows

❌ **Don't use subagents when:**
- Simple questions
- General coding
- Quick lookups
- Already in flow

### When Skills Activate

**Skills activate automatically** based on natural language:
- No need to invoke explicitly
- Just ask the question
- Claude decides which skill fits

### Maintaining Agents and Skills

**Update when:**
- Patterns change
- New best practices emerge
- Tools or commands updated
- Historical lessons learned

**Review quarterly:**
- Verify commands still work
- Update documentation links
- Refresh examples
- Add new capabilities

---

## Configuration Reference

### Subagent Format
```markdown
---
name: agent-name
description: When to use this agent and what it does
tools: Read, Edit, Bash, Grep
model: sonnet
---

# Agent Title

System prompt and instructions...
```

### Skill Format
```markdown
---
name: skill-name
description: What this skill does and when it activates
---

# Skill Title

Instructions for Claude when skill is active...
```

---

## File Structure

```
BazBOM/
├── .claude/
│   ├── CLAUDE.md                      # Project memory
│   ├── agents/
│   │   ├── bazel-expert.md           # Bazel specialist
│   │   ├── test-runner.md            # Testing specialist
│   │   └── code-reviewer.md          # Review specialist
│   └── skills/
│       ├── sbom-validator/
│       │   └── SKILL.md              # SBOM validation
│       └── performance-profiler/
│           └── SKILL.md              # Performance analysis
└── docs/
    └── AGENTS_AND_SKILLS_GUIDE.md    # This file
```

---

## Quick Start Examples

### Debug Bazel Issue
```
Me: "Bazel projects are returning 0 packages"
Claude: [Invokes bazel-expert agent]
Bazel Expert: "Let me trace the detection flow..."
[Analyzes code, finds issue, provides solution]
```

### Validate SBOM
```
Me: "Is this SBOM valid?"
Claude: [Activates sbom-validator skill]
[Runs validation checks, provides report]
```

### Performance Check
```
Me: "Why is this scan taking 30 seconds?"
Claude: [Activates performance-profiler skill]
[Profiles execution, identifies bottleneck]
```

### Code Review
```
Me: "Review my PR for the Bazel fix"
Claude: [Invokes code-reviewer agent]
Code Reviewer: "Checking code quality..."
[Provides detailed review with specific feedback]
```

### Full Test Suite
```
Me: "Test this fix against all repos"
Claude: [Invokes test-runner agent]
Test Runner: "Running comprehensive test suite..."
[Executes tests, reports results]
```

---

## Advanced Features

### Tool Restrictions

Agents can limit tool access:
```yaml
tools: Read, Grep  # No Bash or Edit
```

This enables read-only analysis agents.

### Model Selection

Choose optimal model for task:
```yaml
model: haiku   # Fast, simple tasks
model: sonnet  # Complex reasoning (default)
model: opus    # Most capable, slower
```

### Skill Import

Skills can reference additional files:
```markdown
See validation-rules.md for detailed checks
```

### Permission Modes

Control permission behavior:
```yaml
permissionMode: auto  # Never ask for permissions
```

---

## Performance Impact

### Subagents
- **Latency:** +2-5 seconds (context loading)
- **Context:** Independent 200K token budget
- **Benefit:** Main chat stays focused

### Skills
- **Latency:** Minimal (progressive loading)
- **Context:** Shares main conversation context
- **Benefit:** Automatic activation

**Rule of thumb:** Use subagents for complex tasks, skills for quick capabilities.

---

## Troubleshooting

### Agent Not Found
```bash
# Check agent exists
ls .claude/agents/

# Verify syntax
cat .claude/agents/agent-name.md
```

### Skill Not Activating
- Description too vague?
- File name correct?
- YAML frontmatter valid?
- Try explicit phrasing

### Agent Errors
```bash
# View agent output
# (Check conversation for errors)

# Update agent
code .claude/agents/agent-name.md

# Test changes
"Use the updated agent-name"
```

---

## Future Enhancements

### Planned Agents
- **docs-updater** - Keep documentation in sync
- **release-manager** - Handle release process
- **security-auditor** - Deep security analysis

### Planned Skills
- **changelog-updater** - Automate CHANGELOG.md
- **dependency-updater** - Manage Cargo.toml updates
- **benchmark-runner** - Performance regression testing

---

## Contributing

### Adding New Agents

1. Identify clear domain/responsibility
2. Create agent file in `.claude/agents/`
3. Write system prompt with examples
4. Test with real scenarios
5. Update this guide

### Adding New Skills

1. Define specific trigger phrases
2. Create skill directory and SKILL.md
3. Write clear activation description
4. Test automatic activation
5. Update this guide

---

## Reference

- **Claude Code Docs:** https://code.claude.com/docs/en/sub-agents
- **Skills Documentation:** https://code.claude.com/docs/en/skills
- **Project Memory:** `.claude/CLAUDE.md`
- **Memory Guide:** `docs/MEMORY_GUIDE.md`

---

## Quick Command Reference

```bash
# List available agents
ls .claude/agents/

# List available skills
ls .claude/skills/

# View agent details
cat .claude/agents/bazel-expert.md

# Invoke agent explicitly
# (in chat): "Use the bazel-expert agent to..."

# Check if skill activated
# (Look for skill-specific output in response)
```

---

**Status:** ✅ Production Ready
**Agent Count:** 8 specialized agents
**Skill Count:** 5 automated skills
**Coverage:** Bazel, Reachability (7 langs), Container Security, Vulnerability Analysis, Polyglot (13 systems), Upgrade Intelligence, Testing, Code Review, SBOM/Performance Validation, Compliance (7 frameworks)

**Benefits:**
- Faster development
- Consistent patterns
- Better code quality
- Automated validation
- Expert assistance on demand

**Maintained By:** BazBOM Development Team
**Last Updated:** 2025-11-18

