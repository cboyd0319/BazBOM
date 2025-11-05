# BazBOM LLM Integration Usage Guide

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Status:** Complete

---

## Overview

BazBOM now includes AI-powered vulnerability fix generation and policy recommendations using Large Language Models (LLMs). This feature is designed with **privacy first** - all AI processing happens locally by default using Ollama, with external APIs available only as opt-in features.

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
curl https://ollama.ai/install.sh | sh

# Start Ollama server
ollama serve

# Pull a model (recommended: codellama for code fixes)
ollama pull codellama
```

### 2. Configure BazBOM

Set environment variables for your chosen provider:

```bash
# Option 1: Local Ollama (RECOMMENDED - privacy-preserving)
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=codellama

# Option 2: OpenAI (OPT-IN - sends data externally)
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-4

# Option 3: Anthropic Claude (OPT-IN - sends data externally)
export ANTHROPIC_API_KEY=sk-ant-...
export ANTHROPIC_MODEL=claude-3-sonnet-20240229
```

### 3. Use LLM Features

```bash
# Generate AI-powered fix suggestions
bazbom fix --llm

# Interactive LLM-assisted remediation
bazbom fix --llm --interactive

# Query policy recommendations
bazbom policy query "What severity threshold should I use for production?"
```

---

## Provider Comparison

| Provider | Privacy | Cost | Speed | Quality | Recommended For |
|----------|---------|------|-------|---------|----------------|
| **Ollama (Local)** | ✅ 100% Private | 🆓 Free | ⚡ Fast | ⭐⭐⭐⭐ Good | **Everyone (Default)** |
| **OpenAI GPT-4** | ⚠️ External | 💰 $0.03-0.06/1K tokens | 🐢 Slow | ⭐⭐⭐⭐⭐ Excellent | Complex fixes, high budget |
| **OpenAI GPT-3.5** | ⚠️ External | 💵 $0.0015-0.002/1K tokens | ⚡ Fast | ⭐⭐⭐⭐ Good | Simple fixes, cost-conscious |
| **Claude 3 Opus** | ⚠️ External | 💰 $15-75/1M tokens | 🐢 Slow | ⭐⭐⭐⭐⭐ Excellent | Complex analysis |
| **Claude 3 Sonnet** | ⚠️ External | 💵 $3-15/1M tokens | ⚡ Fast | ⭐⭐⭐⭐ Good | Balanced performance |
| **Claude 3 Haiku** | ⚠️ External | 💸 $0.25-1.25/1M tokens | ⚡⚡ Very Fast | ⭐⭐⭐ Decent | High volume, low cost |

### Privacy Levels

- ✅ **100% Private**: Ollama (data never leaves your machine)
- ⚠️ **External**: OpenAI/Anthropic (data sent to external servers)

---

## Use Cases

### 1. AI-Powered Vulnerability Fixes

Generate detailed migration guides with code examples:

```bash
# Scan and generate AI-powered fixes
bazbom scan . --ml-risk
bazbom fix --llm

# Example output:
# 
# 🤖 AI-Powered Fix Recommendation
# 
# Vulnerability: CVE-2021-44228 (Log4Shell)
# Current Version: log4j-core 2.14.1
# Fixed Version: 2.21.1
# 
# Migration Steps:
# 1. Update pom.xml dependency version
# 2. Check for Log4j-specific configuration files
# 3. Update any custom appenders
# 
# Breaking Changes:
# - Configuration file format changed in 2.17+
# - Some deprecated methods removed
# 
# Code Changes Required:
# ```java
# // Before
# Logger logger = LogManager.getLogger(MyClass.class);
# 
# // After (no change needed - API compatible)
# Logger logger = LogManager.getLogger(MyClass.class);
# ```
# 
# Estimated Effort: 1-2 hours
```

### 2. Interactive Fix Mode

Let AI guide you through complex migrations:

```bash
bazbom fix --llm --interactive

# Example interaction:
# 
# 🤖 I found 3 vulnerabilities. Let's fix them together.
# 
# [1/3] CVE-2021-44228 in log4j-core:2.14.1
# 
# AI Recommendation:
# Upgrade to 2.21.1. This is a CRITICAL vulnerability (CVSS 10.0).
# 
# Breaking Changes:
# - Configuration format updated (automatic migration available)
# 
# Options:
#   [A] Apply fix automatically
#   [G] Generate detailed migration guide
#   [S] Skip this fix
#   [Q] Quit
# 
# Your choice: G
# 
# [Generating detailed migration guide...]
```

### 3. Policy Recommendations

Query AI for policy advice:

```bash
bazbom policy query "What severity threshold should I use for a Spring Boot microservice?"

# Example response:
# 
# 🤖 AI Policy Recommendation
# 
# For Spring Boot microservices, I recommend:
# 
# 1. Severity Threshold: HIGH or CRITICAL only
#    - Spring Boot has frequent security updates
#    - Medium/Low can create alert fatigue
# 
# 2. KEV Policy: BLOCK
#    - Always block CISA Known Exploited Vulnerabilities
#    - These are actively being exploited in the wild
# 
# 3. EPSS Threshold: 0.5 (50%)
#    - Block vulnerabilities with >50% exploit probability
# 
# 4. Spring-Specific:
#    - Enable Spring Security for all endpoints
#    - Use Spring Boot 3.x for latest patches
#    - Monitor Spring Security advisories
# 
# Sample Policy:
# ```yaml
# policy:
#   severity_threshold: HIGH
#   kev_policy: block
#   epss_threshold: 0.5
#   frameworks:
#     spring_boot:
#       min_version: "3.0.0"
#       security_required: true
# ```
```

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

# Optional (defaults shown)
export OPENAI_MODEL=gpt-4                 # or gpt-3.5-turbo
```

#### Anthropic Claude (External - Opt-In)

```bash
# Required
export ANTHROPIC_API_KEY=sk-ant-...

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
# 🤖 AI Fix Generation Complete
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

## Privacy & Security

### Data Handling

| Provider | Data Sent | Data Retention | Privacy Level |
|----------|-----------|----------------|---------------|
| **Ollama** | None (local only) | Local only | ✅ 100% Private |
| **OpenAI** | Vulnerability info, code snippets | 30 days | ⚠️ External API |
| **Anthropic** | Vulnerability info, code snippets | Per policy | ⚠️ External API |

### Privacy Best Practices

1. **Default to Ollama** - BazBOM prioritizes local processing
2. **Explicit Opt-In** - External APIs require explicit configuration
3. **Warnings** - BazBOM warns when sending data externally
4. **Sensitive Data** - Never send credentials or secrets
5. **Audit Trail** - All LLM calls are logged

### Privacy Warnings

BazBOM displays clear warnings when using external APIs:

```
⚠ WARNING: Using OpenAI API (external service)
⚠ Data will be sent to OpenAI servers
⚠ For privacy-preserving AI, use Ollama (local)
```

---

## Advanced Usage

### Custom Prompts

Customize prompts for your organization:

```rust
use bazbom_ml::llm::{FixPromptBuilder, LlmClient};

let builder = FixPromptBuilder::new(
    "CVE-2021-44228".to_string(),
    "2.14.1".to_string(),
    "2.21.1".to_string(),
    "Maven".to_string(),
)
.with_project_context("Spring Boot microservice with high traffic")
.with_breaking_changes(vec![
    "Configuration format changed".to_string(),
])
.with_custom_instructions("Focus on zero-downtime migration");

let client = LlmClient::from_env()?;
let response = client.chat_completion(builder.build())?;
```

### Batch Processing

Process multiple vulnerabilities efficiently:

```bash
# Generate fixes for all vulnerabilities
bazbom fix --llm --batch

# Example:
# 
# 🤖 Batch Fix Generation (10 vulnerabilities)
# 
# Estimated cost:
#   OpenAI GPT-4: $1.20
#   Ollama (Local): $0.00 (FREE)
# 
# Proceed? [y/N]: y
# 
# [1/10] Generating fix for CVE-2021-44228...
# [2/10] Generating fix for CVE-2024-1234...
# ...
```

---

## Troubleshooting

### Ollama Not Running

```
Error: Ollama API error: Connection refused

Solution:
1. Check Ollama is running: ollama list
2. Start Ollama: ollama serve
3. Pull a model: ollama pull codellama
```

### API Key Invalid

```
Error: OpenAI API error: 401 Unauthorized

Solution:
1. Check API key is set: echo $OPENAI_API_KEY
2. Verify key is valid on OpenAI dashboard
3. Check for typos or extra spaces
```

### Model Not Found

```
Error: Model 'gpt-5' not found

Solution:
1. Use valid model names:
   - OpenAI: gpt-4, gpt-3.5-turbo
   - Claude: claude-3-opus-20240229, claude-3-sonnet-20240229
   - Ollama: llama2, codellama, mistral
2. Check Ollama models: ollama list
```

### Slow Response Times

```
Problem: LLM responses take 30+ seconds

Solutions:
1. Use faster models:
   - OpenAI: Switch from gpt-4 to gpt-3.5-turbo
   - Claude: Switch from Opus to Sonnet or Haiku
   - Ollama: Use smaller models like llama2-7b
2. Reduce max_tokens in config
3. Use local Ollama for fastest responses
```

---

## Examples

### Complete Workflow

```bash
# 1. Setup Ollama (one-time)
brew install ollama
ollama serve &
ollama pull codellama

# 2. Configure BazBOM
export OLLAMA_BASE_URL=http://localhost:11434
export OLLAMA_MODEL=codellama

# 3. Scan project
bazbom scan . --ml-risk

# 4. Review ML-enhanced risk scores
bazbom scan . --ml-risk --format json | jq '.findings[] | select(.ml_risk.level == "Critical")'

# 5. Generate AI-powered fixes
bazbom fix --llm --interactive

# 6. Apply fixes and test
bazbom fix --apply --test

# 7. Create PR with fixes
bazbom fix --pr
```

---

## Integration with CI/CD

### GitHub Actions

```yaml
name: BazBOM LLM Security Check

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Ollama
        run: |
          curl https://ollama.ai/install.sh | sh
          ollama serve &
          ollama pull codellama
      
      - name: Install BazBOM
        run: |
          curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
      
      - name: Scan with AI
        env:
          OLLAMA_BASE_URL: http://localhost:11434
          OLLAMA_MODEL: codellama
        run: |
          bazbom scan . --ml-risk --with-semgrep
      
      - name: Generate AI fixes
        run: |
          bazbom fix --llm --suggest > ai-fixes.md
      
      - name: Upload fixes
        uses: actions/upload-artifact@v3
        with:
          name: ai-fixes
          path: ai-fixes.md
```

---

## FAQ

### Q: Is my data sent to external servers?

**A:** Only if you explicitly configure an external API (OpenAI or Anthropic). By default, BazBOM uses local Ollama which processes everything on your machine.

### Q: Can I use multiple providers?

**A:** BazBOM uses one provider at a time based on priority: Ollama → Anthropic → OpenAI. Configure your preferred provider via environment variables.

### Q: How accurate are AI-generated fixes?

**A:** AI-generated fixes are suggestions that should be reviewed. For critical vulnerabilities, we recommend:
1. Review AI suggestions carefully
2. Test thoroughly in dev/staging
3. Have a rollback plan
4. Consult official migration guides

### Q: Can I use this offline?

**A:** Yes! Ollama runs completely offline once models are downloaded.

### Q: How much does this cost?

**A:** Ollama is free. External APIs:
- **OpenAI GPT-4**: $0.03-0.06 per 1K tokens (~$0.10-0.30 per fix)
- **OpenAI GPT-3.5**: $0.0015-0.002 per 1K tokens (~$0.005-0.01 per fix)  
- **Claude**: $0.25-75 per 1M tokens (varies by model)

### Q: Can I customize prompts?

**A:** Yes, via custom configuration or programmatically using the Rust API.

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

## Resources

- **Ollama**: https://ollama.ai
- **OpenAI API**: https://platform.openai.com/docs/api-reference
- **Anthropic API**: https://docs.anthropic.com/claude/reference/getting-started
- **BazBOM Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs

---

## Feedback

Have suggestions for improving LLM integration? Open an issue:
https://github.com/cboyd0319/BazBOM/issues

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-05  
**Maintained By:** @cboyd0319
