use crate::config::RoutingRule;
use crate::error::{JiraError, Result};
use regex::Regex;

/// Routing engine for team and component assignment
pub struct RoutingEngine {
    /// Routing rules
    rules: Vec<CompiledRoutingRule>,
}

struct CompiledRoutingRule {
    pattern: Regex,
    project: Option<String>,
    component: Option<String>,
    assignee: Option<String>,
    labels: Vec<String>,
    priority: Option<String>,
}

impl RoutingEngine {
    /// Create a new routing engine with the given rules
    pub fn new(rules: Vec<RoutingRule>) -> Result<Self> {
        let compiled_rules = rules
            .into_iter()
            .map(|rule| {
                let pattern = Regex::new(&rule.pattern)
                    .map_err(|e| JiraError::Routing(format!("Invalid regex pattern: {}", e)))?;

                Ok(CompiledRoutingRule {
                    pattern,
                    project: rule.project,
                    component: rule.component,
                    assignee: rule.assignee,
                    labels: rule.labels,
                    priority: rule.priority,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            rules: compiled_rules,
        })
    }

    /// Find matching routing rule for a package
    pub fn find_match(&self, package: &str) -> Option<RoutingMatch> {
        for rule in &self.rules {
            if rule.pattern.is_match(package) {
                return Some(RoutingMatch {
                    project: rule.project.clone(),
                    component: rule.component.clone(),
                    assignee: rule.assignee.clone(),
                    labels: rule.labels.clone(),
                    priority: rule.priority.clone(),
                });
            }
        }

        None
    }
}

/// Routing match result
#[derive(Debug, Clone)]
pub struct RoutingMatch {
    pub project: Option<String>,
    pub component: Option<String>,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub priority: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RoutingRule;

    #[test]
    fn test_routing_engine() {
        let rules = vec![RoutingRule {
            pattern: "^org\\.springframework\\..*".to_string(),
            project: Some("SEC".to_string()),
            component: Some("Backend".to_string()),
            assignee: Some("backend-team".to_string()),
            labels: vec!["spring".to_string()],
            priority: Some("High".to_string()),
        }];

        let engine = RoutingEngine::new(rules).unwrap();
        let result = engine.find_match("org.springframework.boot");

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.component, Some("Backend".to_string()));
    }
}
