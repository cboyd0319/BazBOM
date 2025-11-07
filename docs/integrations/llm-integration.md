# LLM Integration for AI-Powered Features

**Status:** Phase 10 - In Development  
**Privacy Level:**  **100% LOCAL BY DEFAULT** (External APIs are **OPT-IN ONLY**)

---

## Overview

BazBOM supports AI-powered fix generation and policy recommendations using Large Language Models (LLMs). **BazBOM is privacy-first**, which means:

-  **Local LLMs are the default and recommended option**
-  **No data is sent externally without explicit opt-in**
-  **All external API calls require explicit API key configuration**
-  **Users are warned when data is being sent to external services**

---

## Privacy-First Architecture

### Default Behavior

BazBOM **NEVER** sends data to external services without explicit configuration. The default behavior is:

1. **Check for local Ollama** (recommended, privacy-safe)
2. **Fall back to mock provider** (no AI features, but functional)
3. **Never use external APIs unless explicitly configured**

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

### 3. Batch Fix Planning

Optimize fixing multiple vulnerabilities with intelligent batching.

```bash
# Get batch fix plan from LLM
bazbom fix --llm --interactive

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

### 4. Natural Language Policy Queries (Future)

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

## Usage Examples

### Local Ollama (Privacy-Safe)

```bash
# Setup (one-time)
brew install ollama
ollama serve
ollama pull llama2

# Configure
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=llama2

# Use
bazbom fix --llm --suggest
#  Using local Ollama at http://localhost:11434 (privacy-preserving)
```

### External API (Opt-In)

```bash
# Configure (choose one)
export ANTHROPIC_API_KEY=sk-ant-...
# OR
export OPENAI_API_KEY=sk-...

# Use
bazbom fix --llm --suggest
#  Using Anthropic Claude API (OPT-IN: data sent to external service)
```

### Checking Privacy Level

```bash
# Check which provider will be used
bazbom fix --llm --dry-run

# Example output:
# LLM Provider: Ollama (llama2)
# Privacy Level: LOCAL (privacy-safe, no data leaves your machine)
# Status: Ready
```

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

## Token Usage and Cost Tracking

BazBOM tracks token usage when using external APIs:

```bash
# View token usage after scan
bazbom fix --llm --suggest --verbose

# Example output:
# Token Usage:
#   Prompt tokens: 1,234
#   Completion tokens: 567
#   Total tokens: 1,801
#   Estimated cost: $0.05 (GPT-4)
```

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
```

### Ollama Connection Error

```bash
# Make sure Ollama is running
ollama serve

# Or specify custom URL
export OLLAMA_BASE_URL=http://192.168.1.100:11434
```

### External API Rate Limits

External APIs have rate limits. If you hit them:

1. Wait and retry
2. Switch to a different provider
3. Use Ollama (no rate limits)

---

## Future Features (Phase 10)

- [ ] Custom exploit prediction models
- [ ] Code change impact analysis
- [ ] False positive prediction
- [ ] Semantic dependency search
- [ ] Privacy-preserving ML (local models)
- [ ] Integration with GitHub Copilot

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

---

## Related Documentation

- [ML Features](../reference/ml-features.md)
- [Usage Guide](../user-guide/usage.md)
- [Threat Detection](../security/threat-detection.md)

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Status:** Phase 10 Implementation (In Progress)
