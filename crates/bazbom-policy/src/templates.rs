use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub path: String,
}

pub struct PolicyTemplateLibrary;

impl PolicyTemplateLibrary {
    pub fn list_templates() -> Vec<PolicyTemplate> {
        vec![
            PolicyTemplate {
                id: "pci-dss".to_string(),
                name: "PCI-DSS v4.0 Compliance".to_string(),
                description: "Payment Card Industry Data Security Standard for payment processing systems".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/pci-dss.yml".to_string(),
            },
            PolicyTemplate {
                id: "hipaa".to_string(),
                name: "HIPAA Security Rule".to_string(),
                description: "Health Insurance Portability and Accountability Act for healthcare applications".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/hipaa.yml".to_string(),
            },
            PolicyTemplate {
                id: "fedramp-moderate".to_string(),
                name: "FedRAMP Moderate".to_string(),
                description: "Federal Risk and Authorization Management Program for government cloud services".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/fedramp-moderate.yml".to_string(),
            },
            PolicyTemplate {
                id: "soc2".to_string(),
                name: "SOC 2 Type II".to_string(),
                description: "Service Organization Control 2 for B2B SaaS applications".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/soc2.yml".to_string(),
            },
            PolicyTemplate {
                id: "corporate-permissive".to_string(),
                name: "Corporate Standard (Development)".to_string(),
                description: "Permissive policy for development and testing environments".to_string(),
                category: "Development".to_string(),
                path: "examples/policies/corporate-permissive.yml".to_string(),
            },
        ]
    }

    pub fn get_template(template_id: &str) -> Option<PolicyTemplate> {
        Self::list_templates()
            .into_iter()
            .find(|t| t.id == template_id)
    }

    pub fn initialize_template(template_id: &str, project_path: &Path) -> Result<String, String> {
        let template = Self::get_template(template_id)
            .ok_or_else(|| format!("Template not found: {}", template_id))?;

        let source_path = Path::new(&template.path);

        let template_content = if source_path.exists() {
            fs::read_to_string(source_path)
                .map_err(|e| format!("Failed to read template file: {}", e))?
        } else {
            Self::get_embedded_template(template_id)
                .ok_or_else(|| format!("Template content not found for: {}", template_id))?
        };

        let dest = project_path.join("bazbom.yml");

        if dest.exists() {
            return Err(format!(
                "File already exists: {}. Remove it first or use a different location.",
                dest.display()
            ));
        }

        fs::write(&dest, template_content)
            .map_err(|e| format!("Failed to write template file: {}", e))?;

        Ok(format!(
            "✅ Initialized policy template: {} at {}",
            template.name,
            dest.display()
        ))
    }

    fn get_embedded_template(template_id: &str) -> Option<String> {
        match template_id {
            "pci-dss" => Some(include_str!("../../../examples/policies/pci-dss.yml").to_string()),
            "hipaa" => Some(include_str!("../../../examples/policies/hipaa.yml").to_string()),
            "fedramp-moderate" => {
                Some(include_str!("../../../examples/policies/fedramp-moderate.yml").to_string())
            }
            "soc2" => Some(include_str!("../../../examples/policies/soc2.yml").to_string()),
            "corporate-permissive" => Some(
                include_str!("../../../examples/policies/corporate-permissive.yml").to_string(),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let templates = PolicyTemplateLibrary::list_templates();
        assert_eq!(templates.len(), 5);

        let template_ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        assert!(template_ids.contains(&"pci-dss"));
        assert!(template_ids.contains(&"hipaa"));
        assert!(template_ids.contains(&"fedramp-moderate"));
        assert!(template_ids.contains(&"soc2"));
        assert!(template_ids.contains(&"corporate-permissive"));
    }

    #[test]
    fn test_get_template() {
        let template = PolicyTemplateLibrary::get_template("pci-dss");
        assert!(template.is_some());

        let template = template.unwrap();
        assert_eq!(template.id, "pci-dss");
        assert_eq!(template.name, "PCI-DSS v4.0 Compliance");
        assert_eq!(template.category, "Regulatory");
    }

    #[test]
    fn test_get_template_not_found() {
        let template = PolicyTemplateLibrary::get_template("nonexistent");
        assert!(template.is_none());
    }

    #[test]
    fn test_template_categories() {
        let templates = PolicyTemplateLibrary::list_templates();

        let regulatory: Vec<_> = templates
            .iter()
            .filter(|t| t.category == "Regulatory")
            .collect();
        assert_eq!(regulatory.len(), 4);

        let development: Vec<_> = templates
            .iter()
            .filter(|t| t.category == "Development")
            .collect();
        assert_eq!(development.len(), 1);
    }

    #[test]
    fn test_embedded_templates_exist() {
        let template_ids = vec![
            "pci-dss",
            "hipaa",
            "fedramp-moderate",
            "soc2",
            "corporate-permissive",
        ];

        for id in template_ids {
            let content = PolicyTemplateLibrary::get_embedded_template(id);
            assert!(
                content.is_some(),
                "Embedded template should exist for: {}",
                id
            );
            assert!(
                !content.unwrap().is_empty(),
                "Template content should not be empty for: {}",
                id
            );
        }
    }

    #[test]
    fn test_template_serialization() {
        let template = PolicyTemplate {
            id: "test".to_string(),
            name: "Test Template".to_string(),
            description: "Test description".to_string(),
            category: "Test".to_string(),
            path: "test/path.yml".to_string(),
        };

        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Test Template"));
    }
}
