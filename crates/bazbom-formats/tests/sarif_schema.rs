use bazbom_formats::sarif::{Result as SarifResult, Rule, SarifReport};

#[test]
fn test_sarif_report_creation() {
    let report = SarifReport::new("bazbom", "0.0.1");
    
    assert_eq!(report.version, "2.1.0");
    assert_eq!(report.schema, "https://json.schemastore.org/sarif-2.1.0.json");
    assert_eq!(report.runs.len(), 1);
    assert_eq!(report.runs[0].tool.driver.name, "bazbom");
}

#[test]
fn test_sarif_add_result() {
    let mut report = SarifReport::new("bazbom", "0.0.1");
    
    let result = SarifResult::new("CVE-2023-1234", "warning", "Vulnerability found");
    
    report.add_result(result);
    
    assert_eq!(report.runs[0].results.len(), 1);
    assert_eq!(report.runs[0].results[0].rule_id, "CVE-2023-1234");
    assert_eq!(report.runs[0].results[0].message.text, "Vulnerability found");
}

#[test]
fn test_sarif_result_with_location() {
    let result = SarifResult::new("CVE-2023-5678", "error", "Outdated dependency")
        .with_location("pom.xml");
    
    assert_eq!(result.locations.as_ref().unwrap().len(), 1);
    assert_eq!(result.locations.as_ref().unwrap()[0].physical_location.artifact_location.uri, "pom.xml");
}

#[test]
fn test_sarif_serialization() {
    let report = SarifReport::new("bazbom", "0.0.1");
    
    let json = serde_json::to_string(&report).expect("Failed to serialize SARIF report");
    
    assert!(json.contains("\"version\":\"2.1.0\""));
    assert!(json.contains("\"$schema\""));
}

#[test]
fn test_sarif_multiple_results() {
    let mut report = SarifReport::new("bazbom", "0.0.1");
    
    let result1 = SarifResult::new("CVE-2023-0001", "error", "Critical vulnerability");
    let result2 = SarifResult::new("CVE-2023-0002", "warning", "Medium vulnerability");
    
    report.add_result(result1);
    report.add_result(result2);
    
    assert_eq!(report.runs[0].results.len(), 2);
    assert_eq!(report.runs[0].results[0].level, "error");
    assert_eq!(report.runs[0].results[1].level, "warning");
}

#[test]
fn test_sarif_level_types() {
    let error_result = SarifResult::new("TEST-001", "error", "Error level");
    let warning_result = SarifResult::new("TEST-002", "warning", "Warning level");
    let note_result = SarifResult::new("TEST-003", "note", "Note level");
    
    assert_eq!(error_result.level, "error");
    assert_eq!(warning_result.level, "warning");
    assert_eq!(note_result.level, "note");
}

#[test]
fn test_sarif_result_without_location() {
    let result = SarifResult::new("CVE-2023-9999", "warning", "Test vulnerability");
    
    assert!(result.locations.is_none());
}

#[test]
fn test_sarif_with_multiple_locations() {
    let result1 = SarifResult::new("CVE-2023-0001", "error", "Vuln in POM")
        .with_location("pom.xml");
    let result2 = SarifResult::new("CVE-2023-0002", "error", "Vuln in Gradle")
        .with_location("build.gradle");
    
    assert_eq!(result1.locations.as_ref().unwrap()[0].physical_location.artifact_location.uri, "pom.xml");
    assert_eq!(result2.locations.as_ref().unwrap()[0].physical_location.artifact_location.uri, "build.gradle");
}

#[test]
fn test_sarif_add_rule() {
    let mut report = SarifReport::new("bazbom", "0.0.1");
    
    let rule = Rule::new("CVE-2023-1234", "Critical security vulnerability", "error");
    
    report.add_rule(rule);
    
    assert!(report.runs[0].tool.driver.rules.is_some());
    let rules = report.runs[0].tool.driver.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, "CVE-2023-1234");
    assert_eq!(rules[0].short_description.text, "Critical security vulnerability");
}

#[test]
fn test_sarif_add_multiple_rules() {
    let mut report = SarifReport::new("bazbom", "0.0.1");
    
    let rule1 = Rule::new("CVE-2023-0001", "First vulnerability", "error");
    let rule2 = Rule::new("CVE-2023-0002", "Second vulnerability", "warning");
    
    report.add_rule(rule1);
    report.add_rule(rule2);
    
    let rules = report.runs[0].tool.driver.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].id, "CVE-2023-0001");
    assert_eq!(rules[1].id, "CVE-2023-0002");
}

#[test]
fn test_sarif_rule_creation() {
    let rule = Rule::new("TEST-001", "Test rule description", "warning");
    
    assert_eq!(rule.id, "TEST-001");
    assert_eq!(rule.short_description.text, "Test rule description");
    assert!(rule.default_configuration.is_some());
    assert_eq!(rule.default_configuration.unwrap().level, "warning");
    assert!(rule.full_description.is_none());
    assert!(rule.help.is_none());
}

#[test]
fn test_sarif_result_without_location_locations_none() {
    let result = SarifResult::new("CVE-2024-0001", "error", "Test message");
    
    assert!(result.locations.is_none());
    assert_eq!(result.rule_id, "CVE-2024-0001");
    assert_eq!(result.level, "error");
}

#[test]
fn test_sarif_deserialization() {
    let json = r#"{
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "bazbom",
                    "version": "0.0.1",
                    "informationUri": null,
                    "rules": null
                }
            },
            "results": []
        }]
    }"#;
    
    let report: SarifReport = serde_json::from_str(json).expect("Failed to deserialize");
    
    assert_eq!(report.version, "2.1.0");
    assert_eq!(report.runs[0].tool.driver.name, "bazbom");
    assert_eq!(report.runs[0].results.len(), 0);
}
