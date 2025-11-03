#[cfg(feature = "rego")]
use regorus::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegoEvaluationResult {
    pub deny: Vec<String>,
    pub warn: Vec<String>,
    pub allow: Vec<String>,
}

#[cfg(feature = "rego")]
pub struct RegoPolicy {
    engine: Engine,
}

#[cfg(feature = "rego")]
impl RegoPolicy {
    pub fn from_file(path: &Path) -> Result<Self, String> {
        let mut engine = Engine::new();
        let policy_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read Rego policy file: {}", e))?;

        engine
            .add_policy(path.to_string_lossy().to_string(), policy_content)
            .map_err(|e| format!("Failed to parse Rego policy: {}", e))?;

        Ok(Self { engine })
    }

    pub fn from_string(policy_name: &str, policy_content: &str) -> Result<Self, String> {
        let mut engine = Engine::new();
        engine
            .add_policy(policy_name.to_string(), policy_content.to_string())
            .map_err(|e| format!("Failed to parse Rego policy: {}", e))?;

        Ok(Self { engine })
    }

    pub fn evaluate(&mut self, input: &Value) -> Result<RegoEvaluationResult, String> {
        let rego_input: regorus::Value = serde_json::from_value(input.clone())
            .map_err(|e| format!("Failed to convert input to Rego value: {}", e))?;

        self.engine.set_input(rego_input);

        let deny_result = self
            .engine
            .eval_query("data.bazbom.deny".to_string(), false)
            .map_err(|e| format!("Failed to evaluate deny rule: {}", e))?;

        let warn_result = self
            .engine
            .eval_query("data.bazbom.warn".to_string(), false)
            .map_err(|e| format!("Failed to evaluate warn rule: {}", e))?;

        let allow_result = self
            .engine
            .eval_query("data.bazbom.allow".to_string(), false)
            .map_err(|e| format!("Failed to evaluate allow rule: {}", e))?;

        Ok(RegoEvaluationResult {
            deny: extract_messages_from_results(&deny_result),
            warn: extract_messages_from_results(&warn_result),
            allow: extract_messages_from_results(&allow_result),
        })
    }
}

#[cfg(feature = "rego")]
fn extract_messages_from_results(results: &regorus::QueryResults) -> Vec<String> {
    let mut messages = Vec::new();

    for result in results.result.iter() {
        for expr in result.expressions.iter() {
            extract_strings_from_value(&expr.value, &mut messages);
        }
    }

    messages
}

#[cfg(feature = "rego")]
fn extract_strings_from_value(value: &regorus::Value, messages: &mut Vec<String>) {
    match value {
        regorus::Value::String(s) => {
            messages.push(s.to_string());
        }
        regorus::Value::Set(items) => {
            for item in items.iter() {
                extract_strings_from_value(item, messages);
            }
        }
        regorus::Value::Array(items) => {
            for item in items.iter() {
                extract_strings_from_value(item, messages);
            }
        }
        _ => {}
    }
}

#[cfg(not(feature = "rego"))]
pub struct RegoPolicy;

#[cfg(not(feature = "rego"))]
impl RegoPolicy {
    pub fn from_file(_path: &Path) -> Result<Self, String> {
        Err("Rego support is not enabled. Rebuild with --features rego".to_string())
    }

    pub fn from_string(_policy_name: &str, _policy_content: &str) -> Result<Self, String> {
        Err("Rego support is not enabled. Rebuild with --features rego".to_string())
    }

    pub fn evaluate(&mut self, _input: &Value) -> Result<RegoEvaluationResult, String> {
        Err("Rego support is not enabled. Rebuild with --features rego".to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "rego")]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rego_policy_from_string() {
        let policy = r#"
            package bazbom
            
            deny[msg] {
                input.severity == "CRITICAL"
                msg := "Critical vulnerability detected"
            }
        "#;

        let result = RegoPolicy::from_string("test_policy", policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rego_policy_evaluation() {
        let policy = r#"
            package bazbom
            
            deny[msg] {
                input.severity == "CRITICAL"
                msg := "Critical vulnerability detected"
            }
            
            warn[msg] {
                input.severity == "HIGH"
                msg := "High severity vulnerability"
            }
        "#;

        let mut rego = RegoPolicy::from_string("test_policy", policy).unwrap();

        let input = json!({
            "severity": "CRITICAL"
        });

        let result = rego.evaluate(&input).unwrap();
        assert_eq!(result.deny.len(), 1);
        assert!(result.deny[0].contains("Critical vulnerability"));
        assert_eq!(result.warn.len(), 0);
    }

    #[test]
    fn test_rego_policy_warn_rule() {
        let policy = r#"
            package bazbom
            
            warn[msg] {
                input.epss > 0.5
                msg := sprintf("High EPSS score: %v", [input.epss])
            }
        "#;

        let mut rego = RegoPolicy::from_string("test_policy", policy).unwrap();

        let input = json!({
            "epss": 0.75
        });

        let result = rego.evaluate(&input).unwrap();
        assert_eq!(result.deny.len(), 0);
        assert_eq!(result.warn.len(), 1);
        assert!(result.warn[0].contains("High EPSS score"));
    }

    #[test]
    fn test_rego_policy_allow_rule() {
        let policy = r#"
            package bazbom
            
            deny[msg] {
                input.cve_id
                msg := sprintf("CVE detected: %s", [input.cve_id])
            }
            
            allow[msg] {
                input.cve_id == "CVE-2023-12345"
                input.exception == true
                msg := "Exception approved for CVE-2023-12345"
            }
        "#;

        let mut rego = RegoPolicy::from_string("test_policy", policy).unwrap();

        let input = json!({
            "cve_id": "CVE-2023-12345",
            "exception": true
        });

        let result = rego.evaluate(&input).unwrap();
        assert_eq!(result.deny.len(), 1);
        assert_eq!(result.allow.len(), 1);
        assert!(result.allow[0].contains("Exception approved"));
    }

    #[test]
    fn test_rego_policy_complex_conditions() {
        let policy = r#"
            package bazbom
            
            deny[msg] {
                vuln := input.vulnerabilities[_]
                vuln.severity == "CRITICAL"
                vuln.reachable == true
                msg := sprintf("CRITICAL vulnerability %s is reachable", [vuln.id])
            }
        "#;

        let mut rego = RegoPolicy::from_string("test_policy", policy).unwrap();

        let input = json!({
            "vulnerabilities": [
                {
                    "id": "CVE-2024-1234",
                    "severity": "CRITICAL",
                    "reachable": true
                },
                {
                    "id": "CVE-2024-5678",
                    "severity": "CRITICAL",
                    "reachable": false
                }
            ]
        });

        let result = rego.evaluate(&input).unwrap();
        assert_eq!(result.deny.len(), 1);
        assert!(result.deny[0].contains("CVE-2024-1234"));
        assert!(!result.deny[0].contains("CVE-2024-5678"));
    }

    #[test]
    fn test_rego_policy_kev_check() {
        let policy = r#"
            package bazbom
            
            deny[msg] {
                vuln := input.vulnerabilities[_]
                vuln.cisa_kev == true
                msg := sprintf("CISA KEV vulnerability %s must be fixed", [vuln.id])
            }
        "#;

        let mut rego = RegoPolicy::from_string("test_policy", policy).unwrap();

        let input = json!({
            "vulnerabilities": [
                {
                    "id": "CVE-2024-9999",
                    "severity": "MEDIUM",
                    "cisa_kev": true
                }
            ]
        });

        let result = rego.evaluate(&input).unwrap();
        assert_eq!(result.deny.len(), 1);
        assert!(result.deny[0].contains("CISA KEV"));
        assert!(result.deny[0].contains("CVE-2024-9999"));
    }

    #[test]
    fn test_rego_invalid_policy() {
        let invalid_policy = r#"
            package bazbom
            
            deny[msg] {
                this is not valid rego
            }
        "#;

        let result = RegoPolicy::from_string("invalid_policy", invalid_policy);
        assert!(result.is_err());
    }
}
