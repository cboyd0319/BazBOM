use bazbom_formats::cyclonedx::CycloneDxBom;
use bazbom_formats::sarif::SarifReport;
use bazbom_formats::spdx::SpdxDocument;
use std::fs;

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

#[test]
fn test_spdx_minimal_golden() {
    let golden_path = format!("{}/spdx_minimal.json", GOLDEN_DIR);
    let golden_content = fs::read_to_string(&golden_path).expect("Failed to read golden file");

    // Parse golden file
    let golden: SpdxDocument =
        serde_json::from_str(&golden_content).expect("Failed to parse golden SPDX document");

    // Verify structure
    assert_eq!(golden.spdx_version, "SPDX-2.3");
    assert_eq!(golden.data_license, "CC0-1.0");
    assert_eq!(golden.name, "test-document");
    assert!(golden.packages.is_empty());

    // Verify it can be serialized back
    let serialized =
        serde_json::to_string_pretty(&golden).expect("Failed to serialize SPDX document");

    // Parse again to ensure round-trip consistency
    let _roundtrip: SpdxDocument =
        serde_json::from_str(&serialized).expect("Failed to parse round-trip SPDX");
}

#[test]
fn test_cyclonedx_minimal_golden() {
    let golden_path = format!("{}/cyclonedx_minimal.json", GOLDEN_DIR);
    let golden_content = fs::read_to_string(&golden_path).expect("Failed to read golden file");

    // Parse golden file
    let golden: CycloneDxBom =
        serde_json::from_str(&golden_content).expect("Failed to parse golden CycloneDX BOM");

    // Verify structure
    assert_eq!(golden.bom_format, "CycloneDX");
    assert_eq!(golden.spec_version, "1.5");
    assert_eq!(golden.version, 1);
    assert!(golden.components.is_empty());

    // Verify it can be serialized back
    let serialized =
        serde_json::to_string_pretty(&golden).expect("Failed to serialize CycloneDX BOM");

    // Parse again to ensure round-trip consistency
    let _roundtrip: CycloneDxBom =
        serde_json::from_str(&serialized).expect("Failed to parse round-trip CycloneDX");
}

#[test]
fn test_sarif_minimal_golden() {
    let golden_path = format!("{}/sarif_minimal.json", GOLDEN_DIR);
    let golden_content = fs::read_to_string(&golden_path).expect("Failed to read golden file");

    // Parse golden file
    let golden: SarifReport =
        serde_json::from_str(&golden_content).expect("Failed to parse golden SARIF report");

    // Verify structure
    assert_eq!(golden.version, "2.1.0");
    assert_eq!(
        golden.schema,
        "https://json.schemastore.org/sarif-2.1.0.json"
    );
    assert_eq!(golden.runs.len(), 1);
    assert_eq!(golden.runs[0].tool.driver.name, "bazbom");
    assert!(golden.runs[0].results.is_empty());

    // Verify it can be serialized back
    let serialized =
        serde_json::to_string_pretty(&golden).expect("Failed to serialize SARIF report");

    // Parse again to ensure round-trip consistency
    let _roundtrip: SarifReport =
        serde_json::from_str(&serialized).expect("Failed to parse round-trip SARIF");
}

#[test]
fn test_spdx_schema_compliance() {
    // Create a document programmatically
    let doc = SpdxDocument::new("test-doc", "https://bazbom.io/test");

    // Serialize
    let json = serde_json::to_string_pretty(&doc).expect("Failed to serialize");

    // Verify required fields are present
    assert!(json.contains("\"spdxVersion\""));
    assert!(json.contains("\"dataLicense\""));
    assert!(json.contains("\"SPDXID\""));
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"documentNamespace\""));
    assert!(json.contains("\"creationInfo\""));

    // Parse back to ensure valid
    let _parsed: SpdxDocument =
        serde_json::from_str(&json).expect("Failed to parse generated SPDX");
}

#[test]
fn test_cyclonedx_schema_compliance() {
    // Create a BOM programmatically
    let bom = CycloneDxBom::new("bazbom", "0.0.1-dev");

    // Serialize
    let json = serde_json::to_string_pretty(&bom).expect("Failed to serialize");

    // Verify required fields are present
    assert!(json.contains("\"bomFormat\""));
    assert!(json.contains("\"specVersion\""));
    assert!(json.contains("\"version\""));
    assert!(json.contains("\"metadata\""));

    // Parse back to ensure valid
    let _parsed: CycloneDxBom =
        serde_json::from_str(&json).expect("Failed to parse generated CycloneDX");
}

#[test]
fn test_sarif_schema_compliance() {
    // Create a report programmatically
    let report = SarifReport::new("bazbom", "0.0.1-dev");

    // Serialize
    let json = serde_json::to_string_pretty(&report).expect("Failed to serialize");

    // Verify required fields are present
    assert!(json.contains("\"version\""));
    assert!(json.contains("\"$schema\""));
    assert!(json.contains("\"runs\""));
    assert!(json.contains("\"tool\""));
    assert!(json.contains("\"driver\""));

    // Parse back to ensure valid
    let _parsed: SarifReport =
        serde_json::from_str(&json).expect("Failed to parse generated SARIF");
}

#[test]
fn test_golden_files_are_minimal_valid_schemas() {
    // This test ensures our golden files are actually minimal valid schemas
    // that can be parsed and used as reference implementations

    // SPDX
    let spdx_path = format!("{}/spdx_minimal.json", GOLDEN_DIR);
    let spdx_content = fs::read_to_string(&spdx_path).unwrap();
    let _spdx: SpdxDocument = serde_json::from_str(&spdx_content).unwrap();

    // CycloneDX
    let cdx_path = format!("{}/cyclonedx_minimal.json", GOLDEN_DIR);
    let cdx_content = fs::read_to_string(&cdx_path).unwrap();
    let _cdx: CycloneDxBom = serde_json::from_str(&cdx_content).unwrap();

    // SARIF
    let sarif_path = format!("{}/sarif_minimal.json", GOLDEN_DIR);
    let sarif_content = fs::read_to_string(&sarif_path).unwrap();
    let _sarif: SarifReport = serde_json::from_str(&sarif_content).unwrap();
}
