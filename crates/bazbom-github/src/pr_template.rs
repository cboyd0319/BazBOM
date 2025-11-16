use crate::error::Result;
use crate::models::BazBomPrMetadata;
use std::collections::HashMap;

/// PR template engine for generating comprehensive PR descriptions
pub struct PrTemplateEngine {
    /// Template content
    template: String,
}

impl PrTemplateEngine {
    /// Create a new PR template engine with default template
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a PR template engine with custom template
    pub fn with_template(template: String) -> Self {
        Self { template }
    }

    /// Render a PR description with all intelligence modules
    pub fn render(&self, metadata: &BazBomPrMetadata) -> Result<String> {
        let variables = self.build_variables(metadata);
        let mut result = self.template.clone();

        // Replace all {variable} placeholders with values from the map
        for (key, value) in &variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Build variable map from metadata
    fn build_variables(&self, metadata: &BazBomPrMetadata) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // Basic vulnerability info
        vars.insert("cve_id".to_string(), metadata.cve_id.clone());
        vars.insert("package".to_string(), metadata.package.clone());
        vars.insert("current_version".to_string(), metadata.current_version.clone());
        vars.insert("fix_version".to_string(), metadata.fix_version.clone());
        vars.insert("severity".to_string(), metadata.severity.clone());

        // Risk scoring
        vars.insert("ml_risk_score".to_string(), metadata.ml_risk_score.to_string());

        // Reachability
        let reachability_status = if metadata.reachable {
            "⚠️ REACHABLE"
        } else {
            "✅ UNREACHABLE"
        };
        vars.insert("reachability_status".to_string(), reachability_status.to_string());

        // Auto-merge
        let auto_merge_eligible = if metadata.auto_merge_eligible {
            "✅ Yes"
        } else {
            "No"
        };
        vars.insert("auto_merge_eligible".to_string(), auto_merge_eligible.to_string());

        let auto_merge_status = if metadata.auto_merge_eligible {
            "✅ ENABLED - Will merge after tests pass and approval"
        } else {
            "❌ DISABLED - Requires manual review"
        };
        vars.insert("auto_merge_status".to_string(), auto_merge_status.to_string());

        // Risk badge
        let risk_badge = match metadata.severity.as_str() {
            "CRITICAL" => "🔴",
            "HIGH" => "🟠",
            "MEDIUM" => "🟡",
            "LOW" => "🟢",
            _ => "⚪",
        };
        vars.insert("risk_badge".to_string(), risk_badge.to_string());

        // Confidence badge and level
        let confidence_score = if metadata.auto_merge_eligible { 95 } else { 75 };
        let confidence_level = if confidence_score >= 90 {
            "HIGH"
        } else if confidence_score >= 70 {
            "MEDIUM"
        } else {
            "LOW"
        };
        let confidence_badge = if confidence_score >= 90 { "✅" } else { "⚠️" };

        vars.insert("confidence_score".to_string(), confidence_score.to_string());
        vars.insert("confidence_level".to_string(), confidence_level.to_string());
        vars.insert("confidence_badge".to_string(), confidence_badge.to_string());

        // Why fix this (severity-based)
        let why_fix = match metadata.severity.as_str() {
            "CRITICAL" => "🚨 **Hackers are using this right now!** This vulnerability is being actively exploited in the wild. Patching immediately prevents attackers from exploiting your system.",
            "HIGH" => "⚠️ This vulnerability poses a significant security risk and should be patched as soon as possible.",
            "MEDIUM" => "This vulnerability should be addressed in your next security update cycle.",
            "LOW" => "This vulnerability has minimal impact but should be included in routine maintenance.",
            _ => "This vulnerability should be reviewed and addressed according to your security policy.",
        };
        vars.insert("why_fix".to_string(), why_fix.to_string());

        // Jira link (if available)
        if let Some(jira_ticket) = &metadata.jira_ticket {
            vars.insert("jira_link".to_string(), format!(" | [Jira: {}]({})", jira_ticket, jira_ticket));
            vars.insert("jira_ticket_link".to_string(), format!("- **Jira Ticket:** [{}]({})", jira_ticket, jira_ticket));
        } else {
            vars.insert("jira_link".to_string(), String::new());
            vars.insert("jira_ticket_link".to_string(), String::new());
        }

        // BazBOM scan link (if available)
        if let Some(scan_url) = &metadata.bazbom_scan_url {
            vars.insert("bazbom_scan_link".to_string(), format!("- **BazBOM Scan:** [View Details]({})", scan_url));
        } else {
            vars.insert("bazbom_scan_link".to_string(), String::new());
        }

        vars
    }
}

impl Default for PrTemplateEngine {
    fn default() -> Self {
        Self {
            template: include_str!("../templates/pr_template.md").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = PrTemplateEngine::new();
        assert!(!engine.template.is_empty());
    }

    #[test]
    fn test_custom_template_engine() {
        let custom = "Custom template: {cve_id}";
        let engine = PrTemplateEngine::with_template(custom.to_string());
        assert_eq!(engine.template, custom);
    }

    #[test]
    fn test_render_basic() {
        let template = "CVE: {cve_id}, Package: {package}, Version: {current_version} -> {fix_version}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "log4j-core".to_string(),
            current_version: "2.17.0".to_string(),
            fix_version: "2.20.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 92,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata).unwrap();
        assert_eq!(result, "CVE: CVE-2024-1234, Package: log4j-core, Version: 2.17.0 -> 2.20.0");
    }

    #[test]
    fn test_render_with_severity() {
        let template = "Severity: {severity}, Risk: {risk_badge}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 95,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata).unwrap();
        assert_eq!(result, "Severity: CRITICAL, Risk: 🔴");
    }

    #[test]
    fn test_render_with_reachability() {
        let template = "Reachable: {reachability_status}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        // Test reachable
        let metadata_reachable = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "HIGH".to_string(),
            ml_risk_score: 80,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata_reachable).unwrap();
        assert_eq!(result, "Reachable: ⚠️ REACHABLE");

        // Test unreachable
        let metadata_unreachable = BazBomPrMetadata {
            reachable: false,
            ..metadata_reachable
        };

        let result = engine.render(&metadata_unreachable).unwrap();
        assert_eq!(result, "Reachable: ✅ UNREACHABLE");
    }

    #[test]
    fn test_render_with_auto_merge() {
        let template = "Auto-merge: {auto_merge_status}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        // Test auto-merge eligible
        let metadata_eligible = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "MEDIUM".to_string(),
            ml_risk_score: 50,
            reachable: false,
            auto_merge_eligible: true,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata_eligible).unwrap();
        assert!(result.contains("ENABLED"));

        // Test auto-merge not eligible
        let metadata_not_eligible = BazBomPrMetadata {
            auto_merge_eligible: false,
            ..metadata_eligible
        };

        let result = engine.render(&metadata_not_eligible).unwrap();
        assert!(result.contains("DISABLED"));
    }

    #[test]
    fn test_render_with_jira_ticket() {
        let template = "Jira: {jira_link}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "HIGH".to_string(),
            ml_risk_score: 75,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: Some("SEC-567".to_string()),
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata).unwrap();
        assert!(result.contains("SEC-567"));
    }

    #[test]
    fn test_render_with_scan_url() {
        let template = "Scan: {bazbom_scan_link}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "HIGH".to_string(),
            ml_risk_score: 75,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: Some("https://bazbom.example.com/scan/123".to_string()),
        };

        let result = engine.render(&metadata).unwrap();
        assert!(result.contains("bazbom.example.com"));
    }

    #[test]
    fn test_render_with_ml_risk_score() {
        let template = "ML Risk: {ml_risk_score}/100";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test-pkg".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 92,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        let result = engine.render(&metadata).unwrap();
        assert_eq!(result, "ML Risk: 92/100");
    }

    #[test]
    fn test_render_complete_pr_with_default_template() {
        let engine = PrTemplateEngine::new();

        let metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "log4j-core".to_string(),
            current_version: "2.17.0".to_string(),
            fix_version: "2.20.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 92,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: Some("SEC-567".to_string()),
            bazbom_scan_url: Some("https://bazbom.example.com/scan/abc123".to_string()),
        };

        let result = engine.render(&metadata).unwrap();

        // Verify key elements are present
        assert!(result.contains("CVE-2024-1234"));
        assert!(result.contains("log4j-core"));
        assert!(result.contains("2.17.0"));
        assert!(result.contains("2.20.0"));
        assert!(result.contains("CRITICAL"));
        assert!(result.contains("92/100"));
        assert!(result.contains("REACHABLE"));
        assert!(result.contains("SEC-567"));
        assert!(result.contains("bazbom.example.com"));
    }

    #[test]
    fn test_severity_risk_badges() {
        let template = "{risk_badge}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let base_metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 95,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        // Test CRITICAL
        assert_eq!(engine.render(&base_metadata).unwrap(), "🔴");

        // Test HIGH
        let high_metadata = BazBomPrMetadata { severity: "HIGH".to_string(), ..base_metadata.clone() };
        assert_eq!(engine.render(&high_metadata).unwrap(), "🟠");

        // Test MEDIUM
        let medium_metadata = BazBomPrMetadata { severity: "MEDIUM".to_string(), ..base_metadata.clone() };
        assert_eq!(engine.render(&medium_metadata).unwrap(), "🟡");

        // Test LOW
        let low_metadata = BazBomPrMetadata { severity: "LOW".to_string(), ..base_metadata };
        assert_eq!(engine.render(&low_metadata).unwrap(), "🟢");
    }

    #[test]
    fn test_why_fix_messages() {
        let template = "{why_fix}";
        let engine = PrTemplateEngine::with_template(template.to_string());

        let base_metadata = BazBomPrMetadata {
            cve_id: "CVE-2024-1234".to_string(),
            package: "test".to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: "CRITICAL".to_string(),
            ml_risk_score: 95,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        };

        // Test CRITICAL message
        let result = engine.render(&base_metadata).unwrap();
        assert!(result.contains("Hackers are using this right now"));

        // Test HIGH message
        let high_metadata = BazBomPrMetadata { severity: "HIGH".to_string(), ..base_metadata };
        let result = engine.render(&high_metadata).unwrap();
        assert!(result.contains("significant security risk"));
    }
}
