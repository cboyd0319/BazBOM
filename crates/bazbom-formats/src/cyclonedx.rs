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
