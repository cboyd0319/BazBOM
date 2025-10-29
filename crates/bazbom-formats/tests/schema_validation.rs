use bazbom_formats::cyclonedx::{CycloneDxBom, Component};
use bazbom_formats::sarif::{SarifReport, Rule, Result};
use bazbom_formats::spdx::{SpdxDocument, Package};
use jsonschema::Validator;
use serde_json::Value;
use std::fs;

const SCHEMA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/supplychain/sbom_schemas");

fn load_schema(filename: &str) -> Validator {
    let schema_path = format!("{}/{}", SCHEMA_DIR, filename);
    let schema_content = fs::read_to_string(&schema_path)
        .unwrap_or_else(|_| panic!("Failed to read schema file: {}", schema_path));
    
    let schema_json: Value = serde_json::from_str(&schema_content)
        .unwrap_or_else(|e| panic!("Failed to parse schema JSON {}: {}", schema_path, e));
    
    Validator::options()
        .build(&schema_json)
        .unwrap_or_else(|e| panic!("Failed to compile schema {}: {}", filename, e))
}

#[test]
fn test_spdx_output_validates_against_schema() {
    let schema = load_schema("spdx-2.3-schema.json");
    
    let doc = SpdxDocument::new(
        "test-validation",
        "https://bazbom.io/test-validation",
    );
    
    let json_str = serde_json::to_string(&doc)
        .expect("Failed to serialize SPDX document");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized SPDX");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("SPDX validation failed: {}", e);
    }
}

#[test]
fn test_spdx_with_package_validates_against_schema() {
    let schema = load_schema("spdx-2.3-schema.json");
    
    let mut doc = SpdxDocument::new(
        "test-with-package",
        "https://bazbom.io/test-with-package",
    );
    
    let pkg = Package::new("Package-example", "example-package")
        .with_version("1.0.0")
        .with_purl("pkg:maven/com.example/example-package@1.0.0")
        .with_license("MIT");
    
    doc.add_package(pkg);
    
    let json_str = serde_json::to_string(&doc)
        .expect("Failed to serialize SPDX document");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized SPDX");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("SPDX with package validation failed: {}", e);
    }
}

#[test]
#[ignore] // CycloneDX schema requires external schema references (offline mode incompatible)
fn test_cyclonedx_output_validates_against_schema() {
    let schema = load_schema("cyclonedx-1.5-schema.json");
    
    let bom = CycloneDxBom::new("bazbom", "0.0.1-dev");
    
    let json_str = serde_json::to_string(&bom)
        .expect("Failed to serialize CycloneDX BOM");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized CycloneDX");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("CycloneDX validation failed: {}", e);
    }
}

#[test]
#[ignore] // CycloneDX schema requires external schema references (offline mode incompatible)
fn test_cyclonedx_with_component_validates_against_schema() {
    let schema = load_schema("cyclonedx-1.5-schema.json");
    
    let mut bom = CycloneDxBom::new("bazbom", "0.0.1-dev");
    
    let component = Component::new("example", "library")
        .with_version("1.0.0")
        .with_purl("pkg:maven/com.example/example@1.0.0")
        .with_license("MIT");
    
    bom.add_component(component);
    
    let json_str = serde_json::to_string(&bom)
        .expect("Failed to serialize CycloneDX BOM");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized CycloneDX");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("CycloneDX with component validation failed: {}", e);
    }
}

#[test]
fn test_sarif_output_validates_against_schema() {
    let schema = load_schema("sarif-2.1.0-schema.json");
    
    let report = SarifReport::new("bazbom", "0.0.1-dev");
    
    let json_str = serde_json::to_string(&report)
        .expect("Failed to serialize SARIF report");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized SARIF");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("SARIF validation failed: {}", e);
    }
}

#[test]
fn test_sarif_with_result_validates_against_schema() {
    let schema = load_schema("sarif-2.1.0-schema.json");
    
    let mut report = SarifReport::new("bazbom", "0.0.1-dev");
    
    let rule = Rule::new("CVE-2025-12345", "Example security vulnerability", "warning");
    report.add_rule(rule);
    
    let result = Result::new(
        "CVE-2025-12345",
        "warning",
        "Vulnerability found in dependency",
    );
    report.add_result(result);
    
    let json_str = serde_json::to_string(&report)
        .expect("Failed to serialize SARIF report");
    let json_value: Value = serde_json::from_str(&json_str)
        .expect("Failed to parse serialized SARIF");
    
    if let Err(e) = schema.validate(&json_value) {
        panic!("SARIF with result validation failed: {}", e);
    }
}

#[test]
fn test_golden_files_validate_against_schemas() {
    let spdx_schema = load_schema("spdx-2.3-schema.json");
    // CycloneDX schema requires external references - skip for offline mode
    // let cyclonedx_schema = load_schema("cyclonedx-1.5-schema.json");
    let sarif_schema = load_schema("sarif-2.1.0-schema.json");
    
    let golden_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");
    
    // Validate SPDX golden file
    let spdx_path = format!("{}/spdx_minimal.json", golden_dir);
    let spdx_content = fs::read_to_string(&spdx_path)
        .expect("Failed to read SPDX golden file");
    let spdx_json: Value = serde_json::from_str(&spdx_content)
        .expect("Failed to parse SPDX golden file");
    
    if let Err(e) = spdx_schema.validate(&spdx_json) {
        panic!("SPDX golden file validation failed: {}", e);
    }
    
    // CycloneDX golden file validation skipped - external schema references required
    /*
    let cdx_path = format!("{}/cyclonedx_minimal.json", golden_dir);
    let cdx_content = fs::read_to_string(&cdx_path)
        .expect("Failed to read CycloneDX golden file");
    let cdx_json: Value = serde_json::from_str(&cdx_content)
        .expect("Failed to parse CycloneDX golden file");
    
    if let Err(e) = cyclonedx_schema.validate(&cdx_json) {
        panic!("CycloneDX golden file validation failed: {}", e);
    }
    */
    
    // Validate SARIF golden file
    let sarif_path = format!("{}/sarif_minimal.json", golden_dir);
    let sarif_content = fs::read_to_string(&sarif_path)
        .expect("Failed to read SARIF golden file");
    let sarif_json: Value = serde_json::from_str(&sarif_content)
        .expect("Failed to parse SARIF golden file");
    
    if let Err(e) = sarif_schema.validate(&sarif_json) {
        panic!("SARIF golden file validation failed: {}", e);
    }
}
