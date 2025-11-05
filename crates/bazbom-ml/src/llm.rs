// LLM client for AI-powered fix generation and policy queries
//
// **PRIVACY FIRST**: BazBOM is 100% local by default. All external API calls are OPT-IN only.
//
// Supported providers:
// - **Local Ollama (RECOMMENDED)**: Privacy-preserving, no data leaves your machine
// - OpenAI API (OPT-IN): Requires explicit API key configuration
// - Anthropic Claude API (OPT-IN): Requires explicit API key configuration
// - Mock (for testing)
//
// Default behavior: NO external calls. Users must explicitly configure API keys.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// LLM provider type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProvider {
    /// OpenAI API (GPT-4, GPT-3.5-turbo)
    OpenAI { api_key: String, model: String },
    /// Anthropic Claude API (Claude 3 Opus, Sonnet, Haiku)
    Anthropic { api_key: String, model: String },
    /// Local Ollama server
    Ollama { base_url: String, model: String },
    /// Mock provider for testing
    Mock,
}

/// LLM client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_seconds: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        // PRIVACY FIRST: Default to local Ollama (if available) or Mock
        // Never default to external APIs
        let provider = if let Ok(base_url) = std::env::var("OLLAMA_BASE_URL") {
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());
            LlmProvider::Ollama { base_url, model }
        } else {
            // No local provider available - use mock
            LlmProvider::Mock
        };

        Self {
            provider,
            max_tokens: 2000,
            temperature: 0.7,
            timeout_seconds: 30,
        }
    }
}

/// LLM request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

/// LLM chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub messages: Vec<LlmMessage>,
    pub max_tokens: usize,
    pub temperature: f32,
}

/// LLM chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub tokens_used: usize,
    pub model: String,
}

/// Token usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost_usd: f64,
}

impl TokenUsage {
    /// Estimate cost for OpenAI GPT-4
    pub fn estimate_gpt4_cost(&self) -> f64 {
        // GPT-4 pricing: $0.03/1K prompt tokens, $0.06/1K completion tokens
        let prompt_cost = (self.prompt_tokens as f64 / 1000.0) * 0.03;
        let completion_cost = (self.completion_tokens as f64 / 1000.0) * 0.06;
        prompt_cost + completion_cost
    }

    /// Estimate cost for OpenAI GPT-3.5-turbo
    pub fn estimate_gpt35_cost(&self) -> f64 {
        // GPT-3.5-turbo pricing: $0.0015/1K prompt tokens, $0.002/1K completion tokens
        let prompt_cost = (self.prompt_tokens as f64 / 1000.0) * 0.0015;
        let completion_cost = (self.completion_tokens as f64 / 1000.0) * 0.002;
        prompt_cost + completion_cost
    }

    /// Estimate cost for Anthropic Claude 3 Opus
    pub fn estimate_claude_opus_cost(&self) -> f64 {
        // Claude 3 Opus pricing: $15/1M input tokens, $75/1M output tokens
        let prompt_cost = (self.prompt_tokens as f64 / 1_000_000.0) * 15.0;
        let completion_cost = (self.completion_tokens as f64 / 1_000_000.0) * 75.0;
        prompt_cost + completion_cost
    }

    /// Estimate cost for Anthropic Claude 3 Sonnet
    pub fn estimate_claude_sonnet_cost(&self) -> f64 {
        // Claude 3 Sonnet pricing: $3/1M input tokens, $15/1M output tokens
        let prompt_cost = (self.prompt_tokens as f64 / 1_000_000.0) * 3.0;
        let completion_cost = (self.completion_tokens as f64 / 1_000_000.0) * 15.0;
        prompt_cost + completion_cost
    }

    /// Estimate cost for Anthropic Claude 3 Haiku
    pub fn estimate_claude_haiku_cost(&self) -> f64 {
        // Claude 3 Haiku pricing: $0.25/1M input tokens, $1.25/1M output tokens
        let prompt_cost = (self.prompt_tokens as f64 / 1_000_000.0) * 0.25;
        let completion_cost = (self.completion_tokens as f64 / 1_000_000.0) * 1.25;
        prompt_cost + completion_cost
    }
}

/// LLM client for generating fixes and answering queries
pub struct LlmClient {
    config: LlmConfig,
    usage: TokenUsage,
}

impl LlmClient {
    /// Create new LLM client with configuration
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            usage: TokenUsage::default(),
        }
    }

    /// Create client from environment variables
    ///
    /// **PRIVACY FIRST**: Prioritizes local providers over external APIs
    ///
    /// Priority order:
    /// 1. OLLAMA_BASE_URL (local, privacy-preserving)
    /// 2. ANTHROPIC_API_KEY (opt-in, external)
    /// 3. OPENAI_API_KEY (opt-in, external)
    /// 4. Error if no provider configured
    pub fn from_env() -> Result<Self> {
        // PRIVACY FIRST: Check for local Ollama first
        let provider = if let Ok(base_url) = std::env::var("OLLAMA_BASE_URL") {
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());
            eprintln!(
                "[+] Using local Ollama at {} (privacy-preserving)",
                base_url
            );
            LlmProvider::Ollama { base_url, model }
        } else if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            let model = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-sonnet-20240229".to_string());
            eprintln!("[!] Using Anthropic Claude API (OPT-IN: data sent to external service)");
            LlmProvider::Anthropic { api_key, model }
        } else if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
            eprintln!("[!] Using OpenAI API (OPT-IN: data sent to external service)");
            LlmProvider::OpenAI { api_key, model }
        } else {
            anyhow::bail!(
                "No LLM provider configured.\n\n\
                 PRIVACY FIRST: BazBOM recommends local LLMs.\n\n\
                 Options:\n\
                 1. Local (RECOMMENDED): Set OLLAMA_BASE_URL=http://localhost:11434\n\
                 2. External (OPT-IN): Set ANTHROPIC_API_KEY or OPENAI_API_KEY\n\n\
                 For privacy-preserving AI features, install Ollama: https://ollama.ai"
            );
        };

        Ok(Self::new(LlmConfig {
            provider,
            ..Default::default()
        }))
    }

    /// Send chat completion request
    pub fn chat_completion(&mut self, request: LlmRequest) -> Result<LlmResponse> {
        match &self.config.provider {
            LlmProvider::OpenAI { api_key, model } => {
                let api_key = api_key.clone();
                let model = model.clone();
                self.openai_chat_completion(&api_key, &model, request)
            }
            LlmProvider::Anthropic { api_key, model } => {
                let api_key = api_key.clone();
                let model = model.clone();
                self.anthropic_chat_completion(&api_key, &model, request)
            }
            LlmProvider::Ollama { base_url, model } => {
                let base_url = base_url.clone();
                let model = model.clone();
                self.ollama_chat_completion(&base_url, &model, request)
            }
            LlmProvider::Mock => {
                // Mock response for testing
                let prompt_tokens = request
                    .messages
                    .iter()
                    .map(|m| m.content.split_whitespace().count())
                    .sum::<usize>();
                let completion_tokens = 20; // Mock completion

                // Update usage tracking even for Mock
                self.usage.prompt_tokens += prompt_tokens;
                self.usage.completion_tokens += completion_tokens;
                self.usage.total_tokens += prompt_tokens + completion_tokens;
                self.usage.estimated_cost_usd = 0.0; // Mock is free

                Ok(LlmResponse {
                    content: "Mock LLM response".to_string(),
                    tokens_used: prompt_tokens + completion_tokens,
                    model: "mock".to_string(),
                })
            }
        }
    }

    /// OpenAI API chat completion
    ///
    /// **PRIVACY WARNING**: This sends data to external OpenAI servers.
    /// Only use if you have explicitly opted in via OPENAI_API_KEY environment variable.
    fn openai_chat_completion(
        &mut self,
        api_key: &str,
        model: &str,
        request: LlmRequest,
    ) -> Result<LlmResponse> {
        // PRIVACY: Warn user that data is being sent externally
        eprintln!("[!] Sending data to OpenAI API (external service)");

        // Build OpenAI API request
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()?;

        // Convert messages to OpenAI format
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        // Call OpenAI API
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            anyhow::bail!("OpenAI API error: {}", response.status());
        }

        let response_json: serde_json::Value = response.json()?;

        // Extract response content
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract token usage
        let prompt_tokens = response_json["usage"]["prompt_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;
        let completion_tokens = response_json["usage"]["completion_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;

        // Update usage tracking
        self.usage.prompt_tokens += prompt_tokens;
        self.usage.completion_tokens += completion_tokens;
        self.usage.total_tokens += prompt_tokens + completion_tokens;
        self.usage.estimated_cost_usd = if model.contains("gpt-4") {
            self.usage.estimate_gpt4_cost()
        } else {
            self.usage.estimate_gpt35_cost()
        };

        Ok(LlmResponse {
            content,
            tokens_used: prompt_tokens + completion_tokens,
            model: model.to_string(),
        })
    }

    /// Anthropic Claude API chat completion
    ///
    /// **PRIVACY WARNING**: This sends data to external Anthropic servers.
    /// Only use if you have explicitly opted in via ANTHROPIC_API_KEY environment variable.
    fn anthropic_chat_completion(
        &mut self,
        api_key: &str,
        model: &str,
        request: LlmRequest,
    ) -> Result<LlmResponse> {
        // PRIVACY: Warn user that data is being sent externally
        eprintln!("[!] Sending data to Anthropic API (external service)");

        // Build Anthropic API request
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()?;

        // Convert messages to Anthropic format
        // Anthropic expects system message separate from conversation
        let mut system_content = String::new();
        let mut conversation_messages = Vec::new();

        for msg in &request.messages {
            if msg.role == "system" {
                system_content = msg.content.clone();
            } else {
                conversation_messages.push(serde_json::json!({
                    "role": msg.role,
                    "content": msg.content,
                }));
            }
        }

        let mut body = serde_json::json!({
            "model": model,
            "messages": conversation_messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        if !system_content.is_empty() {
            body["system"] = serde_json::json!(system_content);
        }

        // Call Anthropic API
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            anyhow::bail!("Anthropic API error: {}", response.status());
        }

        let response_json: serde_json::Value = response.json()?;

        // Extract response content
        let content = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract token usage
        let prompt_tokens = response_json["usage"]["input_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response_json["usage"]["output_tokens"]
            .as_u64()
            .unwrap_or(0) as usize;

        // Update usage tracking
        self.usage.prompt_tokens += prompt_tokens;
        self.usage.completion_tokens += completion_tokens;
        self.usage.total_tokens += prompt_tokens + completion_tokens;
        self.usage.estimated_cost_usd = if model.contains("opus") {
            self.usage.estimate_claude_opus_cost()
        } else if model.contains("sonnet") {
            self.usage.estimate_claude_sonnet_cost()
        } else {
            self.usage.estimate_claude_haiku_cost()
        };

        Ok(LlmResponse {
            content,
            tokens_used: prompt_tokens + completion_tokens,
            model: model.to_string(),
        })
    }

    /// Ollama API chat completion
    ///
    /// **PRIVACY SAFE**: This uses local Ollama server. No data leaves your machine.
    /// This is the RECOMMENDED provider for BazBOM's privacy-first approach.
    fn ollama_chat_completion(
        &mut self,
        base_url: &str,
        model: &str,
        request: LlmRequest,
    ) -> Result<LlmResponse> {
        // PRIVACY: Ollama runs locally - no external data transmission
        eprintln!("[+] Using local Ollama (privacy-preserving)");

        // Build Ollama API request
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds))
            .build()?;

        // Convert messages to Ollama format
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "num_predict": request.max_tokens,
            }
        });

        // Call Ollama API
        let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Ollama API error: {} (Is Ollama running at {}?)",
                response.status(),
                base_url
            );
        }

        let response_json: serde_json::Value = response.json()?;

        // Extract response content
        let content = response_json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract token usage (Ollama provides these in eval metrics)
        let prompt_tokens = response_json["prompt_eval_count"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = response_json["eval_count"].as_u64().unwrap_or(0) as usize;

        // Update usage tracking
        self.usage.prompt_tokens += prompt_tokens;
        self.usage.completion_tokens += completion_tokens;
        self.usage.total_tokens += prompt_tokens + completion_tokens;
        // Ollama is free (local)
        self.usage.estimated_cost_usd = 0.0;

        Ok(LlmResponse {
            content,
            tokens_used: prompt_tokens + completion_tokens,
            model: model.to_string(),
        })
    }

    /// Get current token usage
    pub fn token_usage(&self) -> &TokenUsage {
        &self.usage
    }

    /// Reset token usage tracking
    pub fn reset_usage(&mut self) {
        self.usage = TokenUsage::default();
    }

    /// Check if the current provider is privacy-safe (local only)
    pub fn is_privacy_safe(&self) -> bool {
        matches!(
            self.config.provider,
            LlmProvider::Ollama { .. } | LlmProvider::Mock
        )
    }

    /// Check if the current provider sends data externally
    pub fn is_external(&self) -> bool {
        matches!(
            self.config.provider,
            LlmProvider::OpenAI { .. } | LlmProvider::Anthropic { .. }
        )
    }

    /// Get a description of the provider's privacy level
    pub fn privacy_level(&self) -> &str {
        match &self.config.provider {
            LlmProvider::Ollama { .. } => "LOCAL (privacy-safe, no data leaves your machine)",
            LlmProvider::Mock => "MOCK (testing only)",
            LlmProvider::OpenAI { .. } => "EXTERNAL (data sent to OpenAI servers)",
            LlmProvider::Anthropic { .. } => "EXTERNAL (data sent to Anthropic servers)",
        }
    }
}

/// Fix generation prompt builder
pub struct FixPromptBuilder {
    vulnerability: String,
    current_version: String,
    target_version: String,
    build_system: String,
    breaking_changes: Vec<String>,
}

impl FixPromptBuilder {
    pub fn new(
        vulnerability: String,
        current_version: String,
        target_version: String,
        build_system: String,
    ) -> Self {
        Self {
            vulnerability,
            current_version,
            target_version,
            build_system,
            breaking_changes: Vec::new(),
        }
    }

    pub fn with_breaking_changes(mut self, changes: Vec<String>) -> Self {
        self.breaking_changes = changes;
        self
    }

    pub fn build(&self) -> LlmRequest {
        let system_message = LlmMessage {
            role: "system".to_string(),
            content: "You are a security expert helping developers fix vulnerabilities in JVM projects. \
                      Provide clear, actionable migration guides focusing on Maven, Gradle, and Bazel build systems. \
                      Be concise and practical.".to_string(),
        };

        let user_content = if self.breaking_changes.is_empty() {
            format!(
                "I need to fix {} by upgrading from version {} to {}.\n\
                 Build system: {}\n\n\
                 Please provide:\n\
                 1. Steps to upgrade the dependency\n\
                 2. Any configuration changes needed\n\
                 3. Testing recommendations",
                self.vulnerability, self.current_version, self.target_version, self.build_system
            )
        } else {
            format!(
                "I need to fix {} by upgrading from version {} to {}.\n\
                 Build system: {}\n\n\
                 Known breaking changes:\n{}\n\n\
                 Please provide:\n\
                 1. Steps to upgrade the dependency\n\
                 2. Code changes needed for each breaking change\n\
                 3. Configuration changes\n\
                 4. Testing recommendations",
                self.vulnerability,
                self.current_version,
                self.target_version,
                self.build_system,
                self.breaking_changes.join("\n")
            )
        };

        let user_message = LlmMessage {
            role: "user".to_string(),
            content: user_content,
        };

        LlmRequest {
            messages: vec![system_message, user_message],
            max_tokens: 2000,
            temperature: 0.7,
        }
    }
}

/// Policy query prompt builder
pub struct PolicyQueryBuilder {
    query: String,
    project_type: Option<String>,
    compliance_requirements: Vec<String>,
}

impl PolicyQueryBuilder {
    pub fn new(query: String) -> Self {
        Self {
            query,
            project_type: None,
            compliance_requirements: Vec::new(),
        }
    }

    pub fn with_project_type(mut self, project_type: String) -> Self {
        self.project_type = Some(project_type);
        self
    }

    pub fn with_compliance(mut self, requirements: Vec<String>) -> Self {
        self.compliance_requirements = requirements;
        self
    }

    pub fn build(&self) -> LlmRequest {
        let system_message = LlmMessage {
            role: "system".to_string(),
            content: "You are a security policy expert helping configure BazBOM policies. \
                      Provide clear recommendations based on industry best practices."
                .to_string(),
        };

        let mut user_content = format!("Policy query: {}\n\n", self.query);

        if let Some(ref project_type) = self.project_type {
            user_content.push_str(&format!("Project type: {}\n", project_type));
        }

        if !self.compliance_requirements.is_empty() {
            user_content.push_str(&format!(
                "Compliance requirements: {}\n",
                self.compliance_requirements.join(", ")
            ));
        }

        user_content.push_str("\nProvide specific BazBOM policy recommendations.");

        let user_message = LlmMessage {
            role: "user".to_string(),
            content: user_content,
        };

        LlmRequest {
            messages: vec![system_message, user_message],
            max_tokens: 1500,
            temperature: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_creation() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            max_tokens: 1000,
            temperature: 0.7,
            timeout_seconds: 30,
        };
        let client = LlmClient::new(config);
        assert_eq!(client.token_usage().total_tokens, 0);
    }

    #[test]
    fn test_mock_chat_completion() {
        let mut client = LlmClient::new(LlmConfig::default());

        let request = LlmRequest {
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: 100,
            temperature: 0.7,
        };

        let response = client.chat_completion(request).unwrap();
        assert!(!response.content.is_empty());
        assert!(response.tokens_used > 0);
    }

    #[test]
    fn test_token_usage_tracking() {
        // Use Mock provider for testing since we don't have real API access
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let mut client = LlmClient::new(config);

        let request = LlmRequest {
            messages: vec![LlmMessage {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            max_tokens: 100,
            temperature: 0.7,
        };

        client.chat_completion(request).unwrap();
        assert!(client.token_usage().total_tokens > 0);

        client.reset_usage();
        assert_eq!(client.token_usage().total_tokens, 0);
    }

    #[test]
    fn test_openai_provider_structure() {
        // Test that OpenAI provider can be created with correct structure
        let config = LlmConfig {
            provider: LlmProvider::OpenAI {
                api_key: "test_key".to_string(),
                model: "gpt-4".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);

        // Verify provider is external
        assert!(client.is_external());
        assert!(!client.is_privacy_safe());

        // Note: Actual API calls require valid credentials
        // Integration tests should be run separately with OPENAI_API_KEY env var
    }

    #[test]
    fn test_anthropic_provider_structure() {
        // Test that Anthropic provider can be created with correct structure
        let config = LlmConfig {
            provider: LlmProvider::Anthropic {
                api_key: "test_key".to_string(),
                model: "claude-3-sonnet-20240229".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);

        // Verify provider is external
        assert!(client.is_external());
        assert!(!client.is_privacy_safe());

        // Note: Actual API calls require valid credentials
        // Integration tests should be run separately with ANTHROPIC_API_KEY env var
    }

    #[test]
    fn test_ollama_provider_structure() {
        // Test that Ollama provider can be created with correct structure
        let config = LlmConfig {
            provider: LlmProvider::Ollama {
                base_url: "http://localhost:11434".to_string(),
                model: "llama2".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);

        // Verify provider is local and privacy-safe
        assert!(!client.is_external());
        assert!(client.is_privacy_safe());

        // Note: Actual API calls require Ollama to be running
        // Integration tests should be run separately with Ollama installed
    }

    #[test]
    fn test_fix_prompt_builder() {
        let builder = FixPromptBuilder::new(
            "CVE-2021-44228".to_string(),
            "2.14.1".to_string(),
            "2.21.1".to_string(),
            "Maven".to_string(),
        );

        let request = builder.build();
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(request.messages[1].content.contains("CVE-2021-44228"));
    }

    #[test]
    fn test_fix_prompt_with_breaking_changes() {
        let builder = FixPromptBuilder::new(
            "CVE-2024-1234".to_string(),
            "5.0.0".to_string(),
            "6.0.0".to_string(),
            "Gradle".to_string(),
        )
        .with_breaking_changes(vec![
            "API change: method renamed".to_string(),
            "Config format changed".to_string(),
        ]);

        let request = builder.build();
        assert!(request.messages[1]
            .content
            .contains("Known breaking changes"));
        assert!(request.messages[1].content.contains("API change"));
    }

    #[test]
    fn test_policy_query_builder() {
        let builder = PolicyQueryBuilder::new("What severity threshold should I use?".to_string())
            .with_project_type("Spring Boot".to_string())
            .with_compliance(vec!["PCI-DSS".to_string(), "SOC 2".to_string()]);

        let request = builder.build();
        assert_eq!(request.messages.len(), 2);
        assert!(request.messages[1].content.contains("Spring Boot"));
        assert!(request.messages[1].content.contains("PCI-DSS"));
    }

    #[test]
    fn test_privacy_safe_checks() {
        // Local Ollama is privacy-safe
        let config = LlmConfig {
            provider: LlmProvider::Ollama {
                base_url: "http://localhost:11434".to_string(),
                model: "llama2".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);
        assert!(client.is_privacy_safe());
        assert!(!client.is_external());
        assert!(client.privacy_level().contains("LOCAL"));

        // Mock is privacy-safe
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        assert!(client.is_privacy_safe());
        assert!(!client.is_external());

        // OpenAI is external
        let config = LlmConfig {
            provider: LlmProvider::OpenAI {
                api_key: "test".to_string(),
                model: "gpt-4".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);
        assert!(!client.is_privacy_safe());
        assert!(client.is_external());
        assert!(client.privacy_level().contains("EXTERNAL"));
        assert!(client.privacy_level().contains("OpenAI"));

        // Anthropic is external
        let config = LlmConfig {
            provider: LlmProvider::Anthropic {
                api_key: "test".to_string(),
                model: "claude-3-sonnet-20240229".to_string(),
            },
            ..Default::default()
        };
        let client = LlmClient::new(config);
        assert!(!client.is_privacy_safe());
        assert!(client.is_external());
        assert!(client.privacy_level().contains("EXTERNAL"));
        assert!(client.privacy_level().contains("Anthropic"));
    }

    #[test]
    fn test_default_is_privacy_safe() {
        // Default config should be privacy-safe (never external)
        let config = LlmConfig::default();
        let client = LlmClient::new(config);
        assert!(client.is_privacy_safe());
        assert!(!client.is_external());
    }

    #[test]
    fn test_token_cost_estimation() {
        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
            estimated_cost_usd: 0.0,
        };

        let gpt4_cost = usage.estimate_gpt4_cost();
        assert!((gpt4_cost - 0.06).abs() < 0.001); // $0.03 + $0.03

        let gpt35_cost = usage.estimate_gpt35_cost();
        assert!((gpt35_cost - 0.0025).abs() < 0.0001); // $0.0015 + $0.001

        // Test Claude pricing (per million tokens)
        let usage_large = TokenUsage {
            prompt_tokens: 100_000,
            completion_tokens: 50_000,
            total_tokens: 150_000,
            estimated_cost_usd: 0.0,
        };

        let opus_cost = usage_large.estimate_claude_opus_cost();
        assert!((opus_cost - 5.25).abs() < 0.001); // $1.5 + $3.75

        let sonnet_cost = usage_large.estimate_claude_sonnet_cost();
        assert!((sonnet_cost - 1.05).abs() < 0.001); // $0.3 + $0.75

        let haiku_cost = usage_large.estimate_claude_haiku_cost();
        assert!((haiku_cost - 0.0875).abs() < 0.001); // $0.025 + $0.0625
    }
}
