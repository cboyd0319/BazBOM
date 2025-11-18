use serde::{Deserialize, Serialize};

pub const SPEC_VERSION: &str = "1.5";
pub const BOM_FORMAT: &str = "CycloneDX";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycloneDxBom {
    pub bom_format: String,
    pub spec_version: String,
    pub version: u32,
    pub metadata: Metadata,
    pub components: Vec<Component>,
    pub dependencies: Option<Vec<Dependency>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub timestamp: String,
    pub tools: Vec<Tool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: Option<String>,
    pub purl: Option<String>,
    pub licenses: Option<Vec<License>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_references: Option<Vec<ExternalReference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<Vec<Hash>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalReference {
    #[serde(rename = "type")]
    pub reference_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    pub alg: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub license: LicenseChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LicenseChoice {
    Id { id: String },
    Name { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
}

impl CycloneDxBom {
    pub fn new(tool_name: impl Into<String>, tool_version: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            bom_format: BOM_FORMAT.to_string(),
            spec_version: SPEC_VERSION.to_string(),
            version: 1,
            metadata: Metadata {
                timestamp: now,
                tools: vec![Tool {
                    name: tool_name.into(),
                    version: tool_version.into(),
                }],
            },
            components: Vec::new(),
            dependencies: None,
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }
}

impl Component {
    pub fn new(name: impl Into<String>, component_type: impl Into<String>) -> Self {
        Self {
            component_type: component_type.into(),
            name: name.into(),
            version: None,
            purl: None,
            licenses: None,
            external_references: None,
            hashes: None,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_purl(mut self, purl: impl Into<String>) -> Self {
        self.purl = Some(purl.into());
        self
    }

    pub fn with_license(mut self, license_id: impl Into<String>) -> Self {
        let license = License {
            license: LicenseChoice::Id {
                id: license_id.into(),
            },
        };
        self.licenses = Some(vec![license]);
        self
    }

    pub fn with_download_url(mut self, url: impl Into<String>) -> Self {
        let reference = ExternalReference {
            reference_type: "distribution".to_string(),
            url: url.into(),
        };
        match &mut self.external_references {
            Some(refs) => refs.push(reference),
            None => self.external_references = Some(vec![reference]),
        }
        self
    }

    pub fn with_hash(mut self, alg: impl Into<String>, content: impl Into<String>) -> Self {
        let hash = Hash {
            alg: alg.into(),
            content: content.into(),
        };
        match &mut self.hashes {
            Some(hashes) => hashes.push(hash),
            None => self.hashes = Some(vec![hash]),
        }
        self
    }
}

impl CycloneDxBom {
    /// Convert to XML format
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<bom xmlns=\"http://cyclonedx.org/schema/bom/1.5\" version=\"");
        xml.push_str(&self.version.to_string());
        xml.push_str("\" serialNumber=\"urn:uuid:");
        xml.push_str(&uuid::Uuid::new_v4().to_string());
        xml.push_str("\">\n");

        // Metadata
        xml.push_str("  <metadata>\n");
        xml.push_str("    <timestamp>");
        xml.push_str(&self.metadata.timestamp);
        xml.push_str("</timestamp>\n");

        if !self.metadata.tools.is_empty() {
            xml.push_str("    <tools>\n");
            for tool in &self.metadata.tools {
                xml.push_str("      <tool>\n");
                xml.push_str("        <name>");
                xml.push_str(&xmlescape(&tool.name));
                xml.push_str("</name>\n");
                xml.push_str("        <version>");
                xml.push_str(&xmlescape(&tool.version));
                xml.push_str("</version>\n");
                xml.push_str("      </tool>\n");
            }
            xml.push_str("    </tools>\n");
        }
        xml.push_str("  </metadata>\n");

        // Components
        if !self.components.is_empty() {
            xml.push_str("  <components>\n");
            for component in &self.components {
                xml.push_str("    <component type=\"");
                xml.push_str(&component.component_type);
                xml.push_str("\">\n");

                xml.push_str("      <name>");
                xml.push_str(&xmlescape(&component.name));
                xml.push_str("</name>\n");

                if let Some(version) = &component.version {
                    xml.push_str("      <version>");
                    xml.push_str(&xmlescape(version));
                    xml.push_str("</version>\n");
                }

                if let Some(purl) = &component.purl {
                    xml.push_str("      <purl>");
                    xml.push_str(&xmlescape(purl));
                    xml.push_str("</purl>\n");
                }

                // Hashes
                if let Some(hashes) = &component.hashes {
                    if !hashes.is_empty() {
                        xml.push_str("      <hashes>\n");
                        for hash in hashes {
                            xml.push_str("        <hash alg=\"");
                            xml.push_str(&hash.alg);
                            xml.push_str("\">");
                            xml.push_str(&hash.content);
                            xml.push_str("</hash>\n");
                        }
                        xml.push_str("      </hashes>\n");
                    }
                }

                // Licenses
                if let Some(licenses) = &component.licenses {
                    if !licenses.is_empty() {
                        xml.push_str("      <licenses>\n");
                        for license in licenses {
                            match &license.license {
                                LicenseChoice::Id { id } => {
                                    xml.push_str("        <license><id>");
                                    xml.push_str(&xmlescape(id));
                                    xml.push_str("</id></license>\n");
                                }
                                LicenseChoice::Name { name } => {
                                    xml.push_str("        <license><name>");
                                    xml.push_str(&xmlescape(name));
                                    xml.push_str("</name></license>\n");
                                }
                            }
                        }
                        xml.push_str("      </licenses>\n");
                    }
                }

                // External references
                if let Some(external_refs) = &component.external_references {
                    if !external_refs.is_empty() {
                        xml.push_str("      <externalReferences>\n");
                        for ext_ref in external_refs {
                            xml.push_str("        <reference type=\"");
                            xml.push_str(&ext_ref.reference_type);
                            xml.push_str("\">\n");
                            xml.push_str("          <url>");
                            xml.push_str(&xmlescape(&ext_ref.url));
                            xml.push_str("</url>\n");
                            xml.push_str("        </reference>\n");
                        }
                        xml.push_str("      </externalReferences>\n");
                    }
                }

                xml.push_str("    </component>\n");
            }
            xml.push_str("  </components>\n");
        }

        xml.push_str("</bom>\n");
        xml
    }
}

/// Escape XML special characters
fn xmlescape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bom() {
        let bom = CycloneDxBom::new("bazbom", "0.0.1");
        assert_eq!(bom.bom_format, BOM_FORMAT);
        assert_eq!(bom.spec_version, SPEC_VERSION);
        assert_eq!(bom.version, 1);
    }

    #[test]
    fn test_add_component() {
        let mut bom = CycloneDxBom::new("bazbom", "0.0.1");
        let comp = Component::new("test-lib", "library")
            .with_version("1.0.0")
            .with_purl("pkg:maven/com.example/test-lib@1.0.0")
            .with_license("MIT");
        bom.add_component(comp);
        assert_eq!(bom.components.len(), 1);
        assert_eq!(bom.components[0].name, "test-lib");
    }

    #[test]
    fn test_serialize_to_json() {
        let bom = CycloneDxBom::new("bazbom", "0.0.1");
        let json = serde_json::to_string(&bom).unwrap();
        assert!(json.contains("CycloneDX"));
        assert!(json.contains("1.5"));
    }
}
