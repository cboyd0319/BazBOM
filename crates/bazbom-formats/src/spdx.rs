use serde::{Deserialize, Serialize};

pub const SPDX_VERSION: &str = "SPDX-2.3";
pub const DATA_LICENSE: &str = "CC0-1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpdxDocument {
    pub spdx_version: String,
    pub data_license: String,
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    pub document_namespace: String,
    pub creation_info: CreationInfo,
    pub packages: Vec<Package>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationInfo {
    pub created: String,
    pub creators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    #[serde(rename = "SPDXID")]
    pub spdxid: String,
    pub name: String,
    pub version_info: Option<String>,
    pub download_location: String,
    pub files_analyzed: bool,
    pub license_concluded: Option<String>,
    pub license_declared: Option<String>,
    pub external_refs: Option<Vec<ExternalRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalRef {
    pub reference_category: String,
    pub reference_type: String,
    pub reference_locator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    #[serde(rename = "spdxElementId")]
    pub spdx_element_id: String,
    pub relationship_type: String,
    pub related_spdx_element: String,
}

impl SpdxDocument {
    pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            spdx_version: SPDX_VERSION.to_string(),
            data_license: DATA_LICENSE.to_string(),
            spdxid: "SPDXRef-DOCUMENT".to_string(),
            name: name.into(),
            document_namespace: namespace.into(),
            creation_info: CreationInfo {
                created: now,
                creators: vec!["Tool: bazbom".to_string()],
            },
            packages: Vec::new(),
            relationships: Vec::new(),
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.push(package);
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }
}

impl Package {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            spdxid: format!("SPDXRef-{}", id.into()),
            name: name.into(),
            version_info: None,
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: None,
            license_declared: None,
            external_refs: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version_info = Some(version.into());
        self
    }

    pub fn with_purl(mut self, purl: impl Into<String>) -> Self {
        let ext_ref = ExternalRef {
            reference_category: "PACKAGE-MANAGER".to_string(),
            reference_type: "purl".to_string(),
            reference_locator: purl.into(),
        };
        self.external_refs = Some(vec![ext_ref]);
        self
    }

    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        let lic = license.into();
        self.license_concluded = Some(lic.clone());
        self.license_declared = Some(lic);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spdx_document() {
        let doc = SpdxDocument::new("test-sbom", "https://example.com/sbom/test");
        assert_eq!(doc.spdx_version, SPDX_VERSION);
        assert_eq!(doc.data_license, DATA_LICENSE);
        assert_eq!(doc.name, "test-sbom");
    }

    #[test]
    fn test_add_package() {
        let mut doc = SpdxDocument::new("test", "https://example.com/test");
        let pkg = Package::new("pkg1", "test-package")
            .with_version("1.0.0")
            .with_purl("pkg:maven/com.example/test@1.0.0")
            .with_license("MIT");
        doc.add_package(pkg);
        assert_eq!(doc.packages.len(), 1);
        assert_eq!(doc.packages[0].name, "test-package");
    }

    #[test]
    fn test_serialize_to_json() {
        let doc = SpdxDocument::new("test", "https://example.com/test");
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("SPDX-2.3"));
        assert!(json.contains("CC0-1.0"));
    }
}
