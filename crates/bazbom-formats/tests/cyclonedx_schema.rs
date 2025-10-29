use bazbom_formats::cyclonedx::{Component, CycloneDxBom};

#[test]
fn test_cyclonedx_document_creation() {
    let bom = CycloneDxBom::new("bazbom", "0.0.1");
    
    assert_eq!(bom.bom_format, "CycloneDX");
    assert_eq!(bom.spec_version, "1.5");
    assert_eq!(bom.version, 1);
    assert_eq!(bom.metadata.tools[0].name, "bazbom");
    assert!(bom.components.is_empty());
}

#[test]
fn test_cyclonedx_add_component() {
    let mut bom = CycloneDxBom::new("bazbom", "0.0.1");
    
    let component = Component::new("example-lib", "library")
        .with_version("1.0.0");
    bom.add_component(component);
    
    assert_eq!(bom.components.len(), 1);
    assert_eq!(bom.components[0].name, "example-lib");
    assert_eq!(bom.components[0].version, Some("1.0.0".to_string()));
}

#[test]
fn test_cyclonedx_component_with_purl() {
    let component = Component::new("test-lib", "library")
        .with_version("2.0.0")
        .with_purl("pkg:maven/com.example/test-lib@2.0.0");
    
    assert_eq!(component.name, "test-lib");
    assert_eq!(component.version, Some("2.0.0".to_string()));
    assert_eq!(component.purl, Some("pkg:maven/com.example/test-lib@2.0.0".to_string()));
}

#[test]
fn test_cyclonedx_serialization() {
    let bom = CycloneDxBom::new("bazbom", "0.0.1");
    
    let json = serde_json::to_string(&bom).expect("Failed to serialize CycloneDX BOM");
    
    assert!(json.contains("\"bomFormat\":\"CycloneDX\""));
    assert!(json.contains("\"specVersion\":\"1.5\""));
}

#[test]
fn test_cyclonedx_with_multiple_components() {
    let mut bom = CycloneDxBom::new("bazbom", "0.0.1");
    
    let comp1 = Component::new("lib-a", "library").with_version("1.0.0");
    let comp2 = Component::new("lib-b", "library").with_version("2.0.0");
    let comp3 = Component::new("lib-c", "library").with_version("3.0.0");
    
    bom.add_component(comp1);
    bom.add_component(comp2);
    bom.add_component(comp3);
    
    assert_eq!(bom.components.len(), 3);
}

#[test]
fn test_cyclonedx_deserialization() {
    let json = r#"{
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "metadata": {
            "timestamp": "2025-10-29T00:00:00Z",
            "tools": [{"name": "bazbom", "version": "0.0.1"}]
        },
        "components": []
    }"#;
    
    let bom: CycloneDxBom = serde_json::from_str(json).expect("Failed to deserialize CycloneDX BOM");
    
    assert_eq!(bom.bom_format, "CycloneDX");
    assert_eq!(bom.spec_version, "1.5");
    assert!(bom.components.is_empty());
}

#[test]
fn test_cyclonedx_component_with_license() {
    let component = Component::new("licensed-lib", "library")
        .with_version("1.0.0")
        .with_license("Apache-2.0");
    
    assert!(component.licenses.is_some());
    assert_eq!(component.licenses.as_ref().unwrap().len(), 1);
}

#[test]
fn test_cyclonedx_component_types() {
    let library = Component::new("my-lib", "library");
    let application = Component::new("my-app", "application");
    let framework = Component::new("my-framework", "framework");
    
    assert_eq!(library.component_type, "library");
    assert_eq!(application.component_type, "application");
    assert_eq!(framework.component_type, "framework");
}
