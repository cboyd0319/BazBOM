// LLM-powered fix generation for vulnerability remediation
//
// Generates migration guides and code change suggestions using LLMs

use crate::llm::{FixPromptBuilder, LlmClient};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Vulnerability fix context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixContext {
    pub cve: String,
    pub package: String,
    pub current_version: String,
    pub fixed_version: String,
    pub build_system: String,
    pub severity: String,
    pub cvss_score: Option<f64>,
    pub breaking_changes: Vec<String>,
}

/// Generated fix guide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixGuide {
    pub cve: String,
    pub package: String,
    pub upgrade_steps: Vec<String>,
    pub code_changes: Vec<CodeChange>,
    pub configuration_changes: Vec<ConfigChange>,
    pub testing_recommendations: Vec<String>,
    pub estimated_effort_hours: Option<f32>,
    pub breaking_change_severity: BreakingSeverity,
}

/// Code change recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub description: String,
    pub file_pattern: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub reason: String,
}

/// Configuration change recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub file: String,
    pub description: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Breaking change severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingSeverity {
    None,
    Minor,
    Moderate,
    Major,
}

/// LLM-powered fix generator
pub struct FixGenerator {
    llm_client: LlmClient,
}

impl FixGenerator {
    /// Create new fix generator with LLM client
    pub fn new(llm_client: LlmClient) -> Self {
        Self { llm_client }
    }

    /// Generate fix guide for vulnerability
    pub fn generate_fix_guide(&mut self, context: FixContext) -> Result<FixGuide> {
        // Build LLM prompt
        let prompt_builder = FixPromptBuilder::new(
            context.cve.clone(),
            context.current_version.clone(),
            context.fixed_version.clone(),
            context.build_system.clone(),
        );

        let prompt_builder = if !context.breaking_changes.is_empty() {
            prompt_builder.with_breaking_changes(context.breaking_changes.clone())
        } else {
            prompt_builder
        };

        let request = prompt_builder.build();

        // Get LLM response
        let response = self
            .llm_client
            .chat_completion(request)
            .context("Failed to get LLM response for fix guide")?;

        // Parse response into structured guide
        let guide = self.parse_fix_guide_response(&context, &response.content)?;

        Ok(guide)
    }

    /// Parse LLM response into structured fix guide
    fn parse_fix_guide_response(&self, context: &FixContext, content: &str) -> Result<FixGuide> {
        // NOTE: This is a simplified parser
        // Real implementation would use more sophisticated NLP or structured output

        let lines: Vec<&str> = content.lines().collect();
        let mut upgrade_steps = Vec::new();
        let mut code_changes = Vec::new();
        let config_changes = Vec::new();
        let mut testing_recommendations = Vec::new();

        // Simple heuristic parsing (would be improved with structured LLM output)
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Look for upgrade steps (numbered lists)
            if line.starts_with(|c: char| c.is_numeric()) && line.contains('.') {
                upgrade_steps.push(line.to_string());
            }

            // Look for code change keywords
            if line.contains("code change") || line.contains("update") || line.contains("replace") {
                code_changes.push(CodeChange {
                    description: line.to_string(),
                    file_pattern: "**/*.java".to_string(), // Default pattern
                    before: None,
                    after: None,
                    reason: "LLM recommendation".to_string(),
                });
            }

            // Look for testing keywords
            if line.contains("test") && (line.contains("should") || line.contains("verify")) {
                testing_recommendations.push(line.to_string());
            }
        }

        // Determine breaking severity
        let breaking_severity = if context.breaking_changes.is_empty() {
            BreakingSeverity::None
        } else if context.breaking_changes.len() > 5 {
            BreakingSeverity::Major
        } else if context.breaking_changes.len() > 2 {
            BreakingSeverity::Moderate
        } else {
            BreakingSeverity::Minor
        };

        // Estimate effort based on breaking changes and response complexity
        let estimated_effort_hours = if context.breaking_changes.is_empty() {
            Some(0.5) // Simple version bump
        } else {
            Some(2.0 + (context.breaking_changes.len() as f32 * 0.5))
        };

        Ok(FixGuide {
            cve: context.cve.clone(),
            package: context.package.clone(),
            upgrade_steps,
            code_changes,
            configuration_changes: config_changes,
            testing_recommendations,
            estimated_effort_hours,
            breaking_change_severity: breaking_severity,
        })
    }

    /// Generate batch fix recommendations
    pub fn generate_batch_fix_plan(&mut self, contexts: Vec<FixContext>) -> Result<BatchFixPlan> {
        let mut guides = Vec::new();
        let mut total_effort_hours = 0.0;

        for context in contexts {
            let guide = self.generate_fix_guide(context)?;
            if let Some(effort) = guide.estimated_effort_hours {
                total_effort_hours += effort;
            }
            guides.push(guide);
        }

        // Group by breaking severity
        let no_breaking: Vec<_> = guides
            .iter()
            .filter(|g| g.breaking_change_severity == BreakingSeverity::None)
            .cloned()
            .collect();

        let minor_breaking: Vec<_> = guides
            .iter()
            .filter(|g| g.breaking_change_severity == BreakingSeverity::Minor)
            .cloned()
            .collect();

        let moderate_breaking: Vec<_> = guides
            .iter()
            .filter(|g| g.breaking_change_severity == BreakingSeverity::Moderate)
            .cloned()
            .collect();

        let major_breaking: Vec<_> = guides
            .iter()
            .filter(|g| g.breaking_change_severity == BreakingSeverity::Major)
            .cloned()
            .collect();

        Ok(BatchFixPlan {
            total_vulnerabilities: guides.len(),
            total_estimated_hours: total_effort_hours,
            no_breaking_changes: no_breaking,
            minor_breaking_changes: minor_breaking,
            moderate_breaking_changes: moderate_breaking,
            major_breaking_changes: major_breaking,
        })
    }

    /// Get token usage statistics
    pub fn token_usage(&self) -> &crate::llm::TokenUsage {
        self.llm_client.token_usage()
    }
}

/// Batch fix plan with grouped recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFixPlan {
    pub total_vulnerabilities: usize,
    pub total_estimated_hours: f32,
    pub no_breaking_changes: Vec<FixGuide>,
    pub minor_breaking_changes: Vec<FixGuide>,
    pub moderate_breaking_changes: Vec<FixGuide>,
    pub major_breaking_changes: Vec<FixGuide>,
}

impl BatchFixPlan {
    /// Get recommended fix order (easiest first)
    pub fn recommended_order(&self) -> Vec<&FixGuide> {
        let mut all_guides = Vec::new();
        all_guides.extend(self.no_breaking_changes.iter());
        all_guides.extend(self.minor_breaking_changes.iter());
        all_guides.extend(self.moderate_breaking_changes.iter());
        all_guides.extend(self.major_breaking_changes.iter());
        all_guides
    }

    /// Get high-priority fixes (critical severity)
    pub fn high_priority_fixes(&self) -> Vec<&FixGuide> {
        let all_guides = self.recommended_order();
        // In real implementation, would filter by CVSS score or severity
        // For now, return first 5 as high priority
        all_guides.into_iter().take(5).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{LlmConfig, LlmProvider};

    fn create_test_context() -> FixContext {
        FixContext {
            cve: "CVE-2021-44228".to_string(),
            package: "log4j-core".to_string(),
            current_version: "2.14.1".to_string(),
            fixed_version: "2.21.1".to_string(),
            build_system: "Maven".to_string(),
            severity: "CRITICAL".to_string(),
            cvss_score: Some(10.0),
            breaking_changes: vec![],
        }
    }

    #[test]
    fn test_fix_generator_creation() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let generator = FixGenerator::new(client);
        assert_eq!(generator.token_usage().total_tokens, 0);
    }

    #[test]
    fn test_generate_simple_fix_guide() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let mut generator = FixGenerator::new(client);

        let context = create_test_context();
        let guide = generator.generate_fix_guide(context).unwrap();

        assert_eq!(guide.cve, "CVE-2021-44228");
        assert_eq!(guide.package, "log4j-core");
        assert_eq!(guide.breaking_change_severity, BreakingSeverity::None);
        assert!(guide.estimated_effort_hours.is_some());
    }

    #[test]
    fn test_generate_fix_with_breaking_changes() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let mut generator = FixGenerator::new(client);

        let mut context = create_test_context();
        context.breaking_changes = vec![
            "API method renamed".to_string(),
            "Config format changed".to_string(),
            "Dependency removed".to_string(),
        ];

        let guide = generator.generate_fix_guide(context).unwrap();

        assert_eq!(guide.breaking_change_severity, BreakingSeverity::Moderate);
        assert!(guide.estimated_effort_hours.unwrap() > 2.0);
    }

    #[test]
    fn test_breaking_severity_classification() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let mut generator = FixGenerator::new(client);

        // No breaking changes
        let context = create_test_context();
        let guide = generator.generate_fix_guide(context).unwrap();
        assert_eq!(guide.breaking_change_severity, BreakingSeverity::None);

        // Minor breaking changes (1-2)
        let mut context = create_test_context();
        context.breaking_changes = vec!["Minor change".to_string()];
        let guide = generator.generate_fix_guide(context).unwrap();
        assert_eq!(guide.breaking_change_severity, BreakingSeverity::Minor);

        // Moderate breaking changes (3-5)
        let mut context = create_test_context();
        context.breaking_changes = vec![
            "Change 1".to_string(),
            "Change 2".to_string(),
            "Change 3".to_string(),
        ];
        let guide = generator.generate_fix_guide(context).unwrap();
        assert_eq!(guide.breaking_change_severity, BreakingSeverity::Moderate);

        // Major breaking changes (6+)
        let mut context = create_test_context();
        context.breaking_changes = vec![
            "Change 1".to_string(),
            "Change 2".to_string(),
            "Change 3".to_string(),
            "Change 4".to_string(),
            "Change 5".to_string(),
            "Change 6".to_string(),
        ];
        let guide = generator.generate_fix_guide(context).unwrap();
        assert_eq!(guide.breaking_change_severity, BreakingSeverity::Major);
    }

    #[test]
    fn test_batch_fix_plan() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let mut generator = FixGenerator::new(client);

        let contexts = vec![
            create_test_context(),
            {
                let mut ctx = create_test_context();
                ctx.cve = "CVE-2024-1234".to_string();
                ctx.breaking_changes = vec!["Minor change".to_string()];
                ctx
            },
            {
                let mut ctx = create_test_context();
                ctx.cve = "CVE-2024-5678".to_string();
                ctx.breaking_changes = vec![
                    "Change 1".to_string(),
                    "Change 2".to_string(),
                    "Change 3".to_string(),
                ];
                ctx
            },
        ];

        let plan = generator.generate_batch_fix_plan(contexts).unwrap();

        assert_eq!(plan.total_vulnerabilities, 3);
        assert!(plan.total_estimated_hours > 0.0);
        assert_eq!(plan.no_breaking_changes.len(), 1);
        assert_eq!(plan.minor_breaking_changes.len(), 1);
        assert_eq!(plan.moderate_breaking_changes.len(), 1);
        assert_eq!(plan.major_breaking_changes.len(), 0);
    }

    #[test]
    fn test_recommended_fix_order() {
        let config = LlmConfig {
            provider: LlmProvider::Mock,
            ..Default::default()
        };
        let client = LlmClient::new(config);
        let mut generator = FixGenerator::new(client);

        let contexts = vec![
            {
                let mut ctx = create_test_context();
                ctx.cve = "CVE-MAJOR".to_string();
                ctx.breaking_changes = vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                    "5".to_string(),
                    "6".to_string(),
                ];
                ctx
            },
            create_test_context(), // No breaking
            {
                let mut ctx = create_test_context();
                ctx.cve = "CVE-MODERATE".to_string();
                ctx.breaking_changes = vec!["1".to_string(), "2".to_string(), "3".to_string()];
                ctx
            },
        ];

        let plan = generator.generate_batch_fix_plan(contexts).unwrap();
        let ordered = plan.recommended_order();

        // Should be ordered: none, minor, moderate, major
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].cve, "CVE-2021-44228"); // No breaking
        assert_eq!(ordered[1].cve, "CVE-MODERATE"); // Moderate
        assert_eq!(ordered[2].cve, "CVE-MAJOR"); // Major
    }
}
