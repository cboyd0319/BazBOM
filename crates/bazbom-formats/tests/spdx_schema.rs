use bazbom_formats::spdx::{Package, Relationship, SpdxDocument};

#[test]
fn test_spdx_document_creation() {
    let doc = SpdxDocument::new("test-sbom", "https://example.com/sbom/test");

    assert_eq!(doc.spdx_version, "SPDX-2.3");
    assert_eq!(doc.data_license, "CC0-1.0");
    assert_eq!(doc.spdxid, "SPDXRef-DOCUMENT");
    assert_eq!(doc.name, "test-sbom");
    assert_eq!(doc.document_namespace, "https://example.com/sbom/test");
    assert_eq!(doc.creation_info.creators, vec!["Tool: bazbom"]);
    assert!(doc.packages.is_empty());
    assert!(doc.relationships.is_empty());
}

#[test]
fn test_spdx_package_creation() {
    let pkg = Package::new("pkg1", "example-package")
        .with_version("1.0.0")
        .with_purl("pkg:maven/com.example/example-package@1.0.0");

    assert_eq!(pkg.spdxid, "SPDXRef-pkg1");
    assert_eq!(pkg.name, "example-package");
    assert_eq!(pkg.version_info, Some("1.0.0".to_string()));
    assert_eq!(pkg.download_location, "NOASSERTION");
    assert!(!pkg.files_analyzed);

    let external_refs = pkg.external_refs.unwrap();
    assert_eq!(external_refs.len(), 1);
    assert_eq!(external_refs[0].reference_type, "purl");
}

#[test]
fn test_spdx_document_with_packages() {
    let mut doc = SpdxDocument::new("test-sbom", "https://example.com/sbom/test");

    let pkg1 = Package::new("pkg1", "package-one").with_version("1.0.0");
    let pkg2 = Package::new("pkg2", "package-two").with_version("2.0.0");

    doc.add_package(pkg1);
    doc.add_package(pkg2);

    assert_eq!(doc.packages.len(), 2);
    assert_eq!(doc.packages[0].name, "package-one");
    assert_eq!(doc.packages[1].name, "package-two");
}

#[test]
fn test_spdx_document_with_relationships() {
    let mut doc = SpdxDocument::new("test-sbom", "https://example.com/sbom/test");

    let rel = Relationship {
        spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
        relationship_type: "DESCRIBES".to_string(),
        related_spdx_element: "SPDXRef-pkg1".to_string(),
    };

    doc.add_relationship(rel);

    assert_eq!(doc.relationships.len(), 1);
    assert_eq!(doc.relationships[0].relationship_type, "DESCRIBES");
}

#[test]
fn test_spdx_serialization() {
    let doc = SpdxDocument::new("test-sbom", "https://example.com/sbom/test");

    // Should be serializable to JSON
    let json = serde_json::to_string(&doc).expect("Failed to serialize SPDX document");

    // Should contain required fields
    assert!(json.contains("\"spdxVersion\":\"SPDX-2.3\""));
    assert!(json.contains("\"dataLicense\":\"CC0-1.0\""));
    assert!(json.contains("\"SPDXID\":\"SPDXRef-DOCUMENT\""));
}

#[test]
fn test_spdx_deserialization() {
    let json = r#"{
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "test-sbom",
        "documentNamespace": "https://example.com/sbom/test",
        "creationInfo": {
            "created": "2025-10-29T00:00:00Z",
            "creators": ["Tool: bazbom"]
        },
        "packages": [],
        "relationships": []
    }"#;

    let doc: SpdxDocument =
        serde_json::from_str(json).expect("Failed to deserialize SPDX document");

    assert_eq!(doc.spdx_version, "SPDX-2.3");
    assert_eq!(doc.name, "test-sbom");
    assert!(doc.packages.is_empty());
}

#[test]
fn test_package_with_license() {
    let pkg = Package::new("pkg1", "licensed-package")
        .with_version("1.0.0")
        .with_license("MIT");

    assert_eq!(pkg.license_concluded, Some("MIT".to_string()));
    assert_eq!(pkg.license_declared, Some("MIT".to_string()));
}

#[test]
fn test_package_without_version() {
    let pkg = Package::new("pkg1", "unversioned-package");

    assert_eq!(pkg.spdxid, "SPDXRef-pkg1");
    assert_eq!(pkg.name, "unversioned-package");
    assert_eq!(pkg.version_info, None);
    assert_eq!(pkg.download_location, "NOASSERTION");
}
