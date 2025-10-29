use bazbom_formats::sarif::{Result as SarifResult, SarifReport};

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
