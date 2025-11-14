# LLM Integration Guide

**Status:** Beta
**Privacy Level:** **100% LOCAL BY DEFAULT** (External APIs are **OPT-IN ONLY**)
**Document Version:** 2.0
**Last Updated:** 2025-11-14

---

## Overview

BazBOM supports AI-powered fix generation and policy recommendations using Large Language Models (LLMs). **BazBOM is privacy-first**, which means:

-  **Local LLMs are the default and recommended option**
-  **No data is sent externally without explicit opt-in**
-  **All external API calls require explicit API key configuration**
-  **Users are warned when data is being sent to external services**

### Key Features

- **Privacy-First Design**: Local Ollama processing by default
- **Multi-Provider Support**: OpenAI, Anthropic Claude, or Ollama
- **Smart Fix Generation**: LLM-powered migration guides for vulnerabilities
- **Policy Recommendations**: Natural language policy queries
- **Cost Tracking**: Automatic token usage and cost estimation

---

## Quick Start

### 1. Install Ollama (Recommended - Privacy-Safe)

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama server
ollama serve

# Pull a model (recommended: codellama for code fixes)
ollama pull codellama
```

### 2. Configure BazBOM

```bash
# Option 1: Local Ollama (RECOMMENDED - privacy-preserving)
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=codellama

# Option 2: OpenAI (OPT-IN - sends data externally)
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-4
export BAZBOM_ALLOW_EXTERNAL_API=1

# Option 3: Anthropic Claude (OPT-IN - sends data externally)
export ANTHROPIC_API_KEY=sk-ant-...
export ANTHROPIC_MODEL=claude-3-sonnet-20240229
export BAZBOM_ALLOW_EXTERNAL_API=1
```

### 3. Use LLM Features

```bash
# Generate AI-powered fix suggestions (uses Ollama by default)
bazbom fix --llm

# Interactive LLM-assisted remediation
bazbom fix --llm --interactive

# Combine ML prioritization with LLM fix generation
bazbom fix --ml-prioritize --llm

# Apply fixes with LLM guidance
bazbom fix --llm --apply

# Create PR with LLM-generated descriptions
bazbom fix --llm --pr
```

---

## Privacy-First Architecture

### Default Behavior

BazBOM **NEVER** sends data to external services without explicit configuration. The default behavior is:

1. **Check for local Ollama** (recommended, privacy-safe)
2. **Require BAZBOM_ALLOW_EXTERNAL_API=1** for external APIs
3. **Display warnings** when sending data externally

### Priority Order

When you run `bazbom fix --llm`, BazBOM checks for LLM providers in this order:

1. **OLLAMA_BASE_URL** → Local Ollama ( Privacy-safe, recommended)
2. **ANTHROPIC_API_KEY** → Anthropic Claude ( External, opt-in)
3. **OPENAI_API_KEY** → OpenAI GPT ( External, opt-in)
4. **Error** → No provider configured

---

## Supported LLM Providers

### 1. Ollama (RECOMMENDED)

**Privacy Level:**  **LOCAL - No data leaves your machine**

Ollama runs LLMs locally on your machine. This is the **recommended** option for privacy-conscious users.

#### Installation

```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Or via Homebrew (macOS)
brew install ollama

# Start Ollama server
ollama serve

# Pull a model (first time only)
ollama pull llama2
```

#### Configuration

```bash
# Set environment variables
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=llama2  # or mistral, codellama, etc.

# Use BazBOM with local LLM
bazbom fix --llm --suggest
```

#### Supported Models

- `llama2` - General purpose (recommended)
- `codellama` - Code-focused (good for fix generation)
- `mistral` - Fast and efficient
- `mixtral` - More powerful, slower
- See [Ollama Library](https://ollama.ai/library) for more models

---

### 2. Anthropic Claude (OPT-IN)

**Privacy Level:**  **EXTERNAL - Data sent to Anthropic servers**

Anthropic Claude provides high-quality responses but requires sending data to external servers.

#### When to Use

- Your organization has an Anthropic contract
- You need the highest quality responses
- You've reviewed Anthropic's privacy policy
- You're okay with sending vulnerability data externally

#### Configuration

```bash
# Get API key from https://console.anthropic.com/
export ANTHROPIC_API_KEY=sk-ant-...

# Required for external APIs
export BAZBOM_ALLOW_EXTERNAL_API=1

# Optional: specify model (default: claude-3-sonnet-20240229)
export ANTHROPIC_MODEL=claude-3-opus-20240229

# BazBOM will warn you that data is being sent externally
bazbom fix --llm --suggest
```

**Output:**
```
 Using Anthropic Claude API (OPT-IN: data sent to external service)
```

#### Available Models

- `claude-3-haiku-20240307` - Fast, cost-effective
- `claude-3-sonnet-20240229` - Balanced (default)
- `claude-3-opus-20240229` - Most capable, expensive

#### Pricing (as of 2024)

| Model | Input (per 1M tokens) | Output (per 1M tokens) |
|-------|----------------------|------------------------|
| Haiku | $0.25 | $1.25 |
| Sonnet | $3.00 | $15.00 |
| Opus | $15.00 | $75.00 |

---

### 3. OpenAI GPT (OPT-IN)

**Privacy Level:**  **EXTERNAL - Data sent to OpenAI servers**

OpenAI GPT models provide high-quality responses but require sending data to external servers.

#### When to Use

- Your organization has an OpenAI contract
- You're already using OpenAI services
- You've reviewed OpenAI's privacy policy
- You're okay with sending vulnerability data externally

#### Configuration

```bash
# Get API key from https://platform.openai.com/
export OPENAI_API_KEY=sk-...

# Required for external APIs
export BAZBOM_ALLOW_EXTERNAL_API=1

# Optional: specify model (default: gpt-4)
export OPENAI_MODEL=gpt-4-turbo-preview

# BazBOM will warn you that data is being sent externally
bazbom fix --llm --suggest
```

**Output:**
```
 Using OpenAI API (OPT-IN: data sent to external service)
```

#### Available Models

- `gpt-3.5-turbo` - Fast, cost-effective
- `gpt-4` - Balanced (default)
- `gpt-4-turbo-preview` - More capable, faster

#### Pricing (as of 2024)

| Model | Input (per 1K tokens) | Output (per 1K tokens) |
|-------|----------------------|------------------------|
| GPT-3.5-turbo | $0.0015 | $0.002 |
| GPT-4 | $0.03 | $0.06 |

---

## CLI Reference

### `bazbom fix --llm`

Generate LLM-powered fix guides for vulnerabilities.

**Flags:**
- `--llm`: Enable LLM-powered fix generation
- `--llm-provider <PROVIDER>`: Choose provider (ollama, anthropic, openai). Default: ollama
- `--llm-model <MODEL>`: Specify model (e.g., codellama, gpt-4, claude-3-opus)
- `--suggest`: Show suggestions without applying (default with --llm)
- `--apply`: Apply fixes automatically with LLM guidance
- `--pr`: Create GitHub PR with LLM-generated description
- `--interactive`: Interactive mode with LLM assistance
- `--ml-prioritize`: Combine with ML risk scoring
- `--batch`: Process multiple vulnerabilities efficiently

**Examples:**

```bash
# Use local Ollama (privacy-first, recommended)
bazbom fix --llm

# Use specific Ollama model
bazbom fix --llm --llm-model codellama:latest

# Use OpenAI GPT-4 (requires OPENAI_API_KEY and BAZBOM_ALLOW_EXTERNAL_API=1)
export OPENAI_API_KEY=sk-...
export BAZBOM_ALLOW_EXTERNAL_API=1
bazbom fix --llm --llm-provider openai --llm-model gpt-4

# Use Anthropic Claude
export ANTHROPIC_API_KEY=sk-ant-...
export BAZBOM_ALLOW_EXTERNAL_API=1
bazbom fix --llm --llm-provider anthropic --llm-model claude-3-5-sonnet-20241022

# Combine ML prioritization with LLM guidance
bazbom fix --ml-prioritize --llm

# Interactive mode with LLM assistance
bazbom fix --llm --interactive

# Batch processing
bazbom fix --llm --batch
```

**Output:**

The command generates:
1. **Console output**: Detailed fix guides with steps, code changes, and testing recommendations
2. **File**: `llm_fix_guides.json` - Structured JSON with all generated guides
3. **File**: `remediation_suggestions.json` - Traditional remediation suggestions

---

## Features

### 1. LLM-Powered Fix Generation

Generate detailed migration guides for vulnerability fixes, including breaking changes.

```bash
# Generate fix suggestions with LLM
bazbom fix --llm --suggest

# Example output:
# CVE-2021-44228 in log4j-core 2.14.1 → 2.21.1
#
# Migration Guide:
# 1. Update pom.xml dependency version
# 2. No breaking changes in this upgrade
# 3. Test logging functionality thoroughly
# 4. Verify log output format hasn't changed
#
# Estimated effort: 0.5 hours
```

### 2. Breaking Change Analysis

Get detailed guidance for upgrades with breaking changes.

```bash
# For major version upgrades
bazbom fix --llm --cve CVE-2024-1234

# Example output:
# Spring Boot 2.7.0 → 3.2.0 (MAJOR UPGRADE)
#
# Breaking Changes:
# 1. Java 17 required (was Java 8+)
# 2. Jakarta EE namespaces (javax → jakarta)
# 3. Spring Security configuration changes
# 4. Actuator endpoint changes
#
# Migration Steps:
# 1. Upgrade to Java 17
# 2. Update import statements (javax.* → jakarta.*)
# 3. Update SecurityConfig class...
#
# Estimated effort: 8 hours
```

### 3. Interactive Fix Mode

Let AI guide you through complex migrations:

```bash
bazbom fix --llm --interactive

# Example interaction:
#
#  I found 3 vulnerabilities. Let's fix them together.
#
# [1/3] CVE-2021-44228 in log4j-core:2.14.1
#
# AI Recommendation:
# Upgrade to 2.21.1. This is a CRITICAL vulnerability (CVSS 10.0).
#
# Options:
#   [A] Apply fix automatically
#   [G] Generate detailed migration guide
#   [S] Skip this fix
#   [Q] Quit
#
# Your choice: G
```

### 4. Batch Fix Planning

Optimize fixing multiple vulnerabilities with intelligent batching.

```bash
# Get batch fix plan from LLM
bazbom fix --llm --batch

# Example output:
# Batch 1: No Breaking Changes (8 vulnerabilities)
#   Estimated time: 2 hours
#   Safe to apply together
#
# Batch 2: Minor Breaking Changes (3 vulnerabilities)
#   Estimated time: 4 hours
#   Review recommended
#
# Batch 3: Major Breaking Changes (1 vulnerability)
#   Estimated time: 8 hours
#   Requires code changes
```

### 5. Natural Language Policy Queries

Ask policy questions in natural language.

```bash
# Query policy recommendations
bazbom policy query "What severity threshold for production?"

# Example output:
# For production environments, recommended settings:
# - severity_threshold: HIGH (block CRITICAL and HIGH)
# - kev_policy: block (always block Known Exploited)
# - epss_threshold: 0.3 (30% exploit probability)
# - require_tests: true
```

---

## Provider Comparison

| Provider | Privacy | Cost | Speed | Quality | Recommended For |
|----------|---------|------|-------|---------|----------------|
| **Ollama (Local)** |  100% Private | 🆓 Free |  Fast |  Good | **Everyone (Default)** |
| **OpenAI GPT-4** |  External |  $0.03-0.06/1K tokens |  Slow |  Excellent | Complex fixes, high budget |
| **OpenAI GPT-3.5** |  External |  $0.0015-0.002/1K tokens |  Fast |  Good | Simple fixes, cost-conscious |
| **Claude 3 Opus** |  External |  $15-75/1M tokens |  Slow |  Excellent | Complex analysis |
| **Claude 3 Sonnet** |  External |  $3-15/1M tokens |  Fast |  Good | Balanced performance |
| **Claude 3 Haiku** |  External |  $0.25-1.25/1M tokens |  Very Fast |  Decent | High volume, low cost |

---

## Privacy Considerations

### What Data is Sent?

When using external APIs, BazBOM sends:

- Vulnerability CVE IDs
- Package names and versions
- Build system type (Maven, Gradle, Bazel)
- Breaking change descriptions (if available)

BazBOM **NEVER** sends:

- Your source code
- Your project structure
- Your team information
- Your security policies
- Any PII (Personally Identifiable Information)

### Recommendations

1. **Use Ollama for maximum privacy** - All processing happens locally
2. **Review your organization's policy** - Check if external AI services are allowed
3. **Consider air-gapped environments** - Use Ollama in offline mode
4. **Be aware of data retention** - External providers may retain data per their policies

---

## Configuration

### Environment Variables

#### Ollama (Local - Recommended)

```bash
# Required
export OLLAMA_BASE_URL=http://localhost:11434

# Optional (defaults shown)
export OLLAMA_MODEL=llama2                # Model name
```

#### OpenAI (External - Opt-In)

```bash
# Required
export OPENAI_API_KEY=sk-...
export BAZBOM_ALLOW_EXTERNAL_API=1

# Optional (defaults shown)
export OPENAI_MODEL=gpt-4                 # or gpt-3.5-turbo
```

#### Anthropic Claude (External - Opt-In)

```bash
# Required
export ANTHROPIC_API_KEY=sk-ant-...
export BAZBOM_ALLOW_EXTERNAL_API=1

# Optional (defaults shown)
export ANTHROPIC_MODEL=claude-3-sonnet-20240229  # or opus/haiku
```

### Configuration File

Create `~/.bazbom/llm.toml`:

```toml
[llm]
# Provider: ollama, openai, or anthropic
provider = "ollama"

# Ollama settings
[llm.ollama]
base_url = "http://localhost:11434"
model = "codellama"

# OpenAI settings (optional)
[llm.openai]
model = "gpt-4"
# api_key from env var OPENAI_API_KEY

# Anthropic settings (optional)
[llm.anthropic]
model = "claude-3-sonnet-20240229"
# api_key from env var ANTHROPIC_API_KEY

# Common settings
[llm.settings]
max_tokens = 2000
temperature = 0.7
timeout_seconds = 30
```

---

## Cost Estimation

BazBOM tracks token usage and estimates costs automatically.

### Example Cost Report

```bash
bazbom fix --llm

# Output includes:
#
#  AI Fix Generation Complete
#
# Token Usage:
#   Prompt tokens: 1,234
#   Completion tokens: 2,456
#   Total tokens: 3,690
#
# Estimated Cost:
#   OpenAI GPT-4: $0.18
#   OpenAI GPT-3.5: $0.007
#   Anthropic Opus: $0.20
#   Anthropic Sonnet: $0.04
#   Ollama (Local): $0.00 (FREE)
```

### Cost-Saving Tips

1. **Use Ollama for Most Tasks** - Free and privacy-preserving
2. **Use GPT-3.5 for Simple Fixes** - 95% cheaper than GPT-4
3. **Use Claude Haiku for High Volume** - Cheapest external option
4. **Cache Results** - BazBOM caches LLM responses to avoid duplicate costs

---

## Troubleshooting

### "No LLM provider configured" Error

```bash
# This error means no provider is configured
# Solution: Install and configure Ollama (recommended)

brew install ollama
ollama serve
export OLLAMA_BASE_URL=http://localhost:11434

# Or configure external API (opt-in)
export ANTHROPIC_API_KEY=sk-ant-...
export BAZBOM_ALLOW_EXTERNAL_API=1
```

### Ollama Connection Error

```bash
# Make sure Ollama is running
ollama serve

# Check Ollama is responding
curl http://localhost:11434/api/version

# Or specify custom URL
export OLLAMA_BASE_URL=http://192.168.1.100:11434
```

### External API Rate Limits

External APIs have rate limits. If you hit them:

1. Wait and retry
2. Switch to a different provider
3. Use Ollama (no rate limits)

### API Key Invalid

```
Error: OpenAI API error: 401 Unauthorized

Solution:
1. Check API key is set: echo $OPENAI_API_KEY
2. Verify key is valid on OpenAI dashboard
3. Check for typos or extra spaces
4. Ensure BAZBOM_ALLOW_EXTERNAL_API=1 is set
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: BazBOM LLM Security Check

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - name: Install Ollama
        run: |
          curl https://ollama.ai/install.sh | sh
          ollama serve &
          sleep 5
          ollama pull codellama

      - uses: dtolnay/rust-toolchain@stable

      - name: Install BazBOM
        run: |
          git clone --depth 1 https://github.com/cboyd0319/BazBOM.git /tmp/bazbom
          cd /tmp/bazbom
          cargo build --release -p bazbom
          sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom

      - name: Scan with AI
        env:
          OLLAMA_BASE_URL: http://localhost:11434
          OLLAMA_MODEL: codellama
        run: |
          bazbom scan . --ml-risk

      - name: Generate AI fixes
        run: |
          bazbom fix --llm --suggest > ai-fixes.md

      - name: Upload fixes
        uses: actions/upload-artifact@v4
        with:
          name: ai-fixes
          path: ai-fixes.md
```

---

## FAQ

**Q: Is LLM integration required?**
A: No. All BazBOM features work without LLM integration. LLM features are optional enhancements.

**Q: Can I use BazBOM offline?**
A: Yes. Use Ollama for local LLM features, or disable LLM features entirely.

**Q: Does BazBOM send telemetry?**
A: No. BazBOM has **zero telemetry**. Privacy is a core principle.

**Q: Which LLM provider should I use?**
A: **Ollama** (local) is recommended for privacy. Use external APIs only if your organization approves.

**Q: How much do external APIs cost?**
A: See pricing tables above. Typical usage: $0.01-$0.10 per scan with GPT-4, less with Claude Sonnet.

**Q: Can I use custom Ollama models?**
A: Yes. Set `OLLAMA_MODEL=your-model-name` for any Ollama-compatible model.

**Q: Is my data sent to external servers?**
A: Only if you explicitly configure an external API and set `BAZBOM_ALLOW_EXTERNAL_API=1`. By default, BazBOM uses local Ollama.

**Q: How accurate are AI-generated fixes?**
A: AI-generated fixes are suggestions that should be reviewed. Always test thoroughly in dev/staging before production.

---

## Best Practices

1. **Start Local**: Always start with Ollama for privacy and zero cost
2. **Review Suggestions**: AI is a tool, not a replacement for human judgment
3. **Test Thoroughly**: Always test AI-generated fixes in dev/staging
4. **Track Costs**: Monitor token usage if using paid APIs
5. **Keep Models Updated**: Regularly update Ollama models for best results
6. **Use Appropriate Models**: Match model complexity to task complexity
7. **Cache Results**: BazBOM caches responses to avoid duplicate work

---

## Related Documentation

- [ML Features](../reference/ml-features.md)
- [Usage Guide](../user-guide/usage.md)
- [Threat Detection](../security/threat-detection.md)
- [Container Scanning](../features/container-scanning.md)

---

## Resources

- **Ollama**: https://ollama.ai
- **OpenAI API**: https://platform.openai.com/docs/api-reference
- **Anthropic API**: https://docs.anthropic.com/claude/reference/getting-started
- **BazBOM Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs

---

**Document Version:** 2.0
**Last Updated:** 2025-11-14
**Status:** Beta
