use serde::{Deserialize, Serialize};

pub const SARIF_VERSION: &str = "2.1.0";
pub const SCHEMA_URI: &str = "https://json.schemastore.org/sarif-2.1.0.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReport {
    pub version: String,
    #[serde(rename = "$schema")]
    pub schema: String,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub tool: Tool,
    pub results: Vec<Result>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub driver: Driver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Driver {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub short_description: MessageString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<MessageString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<MessageString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_configuration: Option<Configuration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageString {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    pub rule_id: String,
    pub level: String,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<Location>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub physical_location: PhysicalLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalLocation {
    pub artifact_location: ArtifactLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLocation {
    pub uri: String,
}

impl SarifReport {
    pub fn new(tool_name: impl Into<String>, tool_version: impl Into<String>) -> Self {
        Self {
            version: SARIF_VERSION.to_string(),
            schema: SCHEMA_URI.to_string(),
            runs: vec![Run {
                tool: Tool {
                    driver: Driver {
                        name: tool_name.into(),
                        version: tool_version.into(),
                        information_uri: None,
                        rules: None,
                    },
                },
                results: Vec::new(),
            }],
        }
    }

    pub fn add_result(&mut self, result: Result) {
        if let Some(run) = self.runs.first_mut() {
            run.results.push(result);
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        if let Some(run) = self.runs.first_mut() {
            if run.tool.driver.rules.is_none() {
                run.tool.driver.rules = Some(Vec::new());
            }
            if let Some(rules) = &mut run.tool.driver.rules {
                rules.push(rule);
            }
        }
    }
}

impl Result {
    pub fn new(
        rule_id: impl Into<String>,
        level: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            level: level.into(),
            message: Message {
                text: message.into(),
            },
            locations: None,
        }
    }

    pub fn with_location(mut self, uri: impl Into<String>) -> Self {
        let location = Location {
            physical_location: PhysicalLocation {
                artifact_location: ArtifactLocation { uri: uri.into() },
            },
        };
        self.locations = Some(vec![location]);
        self
    }
}

impl Rule {
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        level: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            short_description: MessageString {
                text: description.into(),
            },
            full_description: None,
            help: None,
            default_configuration: Some(Configuration {
                level: level.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sarif_report() {
        let report = SarifReport::new("bazbom", "0.0.1");
        assert_eq!(report.version, SARIF_VERSION);
        assert_eq!(report.runs.len(), 1);
    }

    #[test]
    fn test_add_result() {
        let mut report = SarifReport::new("bazbom", "0.0.1");
        let result = Result::new("CVE-2024-1234", "error", "Critical vulnerability found")
            .with_location("pom.xml");
        report.add_result(result);
        assert_eq!(report.runs[0].results.len(), 1);
    }

    #[test]
    fn test_serialize_to_json() {
        let report = SarifReport::new("bazbom", "0.0.1");
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("2.1.0"));
        assert!(json.contains("bazbom"));
    }
}
