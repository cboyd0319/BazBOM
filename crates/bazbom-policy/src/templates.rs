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
            // Regulatory Compliance
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
                id: "gdpr".to_string(),
                name: "GDPR Data Protection".to_string(),
                description: "EU General Data Protection Regulation compliance".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/gdpr.yml".to_string(),
            },
            PolicyTemplate {
                id: "iso27001".to_string(),
                name: "ISO 27001".to_string(),
                description: "Information Security Management System standard".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/iso27001.yml".to_string(),
            },
            PolicyTemplate {
                id: "nist-csf".to_string(),
                name: "NIST Cybersecurity Framework".to_string(),
                description: "NIST CSF for risk-based security management".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/nist-csf.yml".to_string(),
            },
            PolicyTemplate {
                id: "cis-benchmarks".to_string(),
                name: "CIS Benchmarks".to_string(),
                description: "Center for Internet Security consensus-based security guidelines".to_string(),
                category: "Regulatory".to_string(),
                path: "examples/policies/cis-benchmarks.yml".to_string(),
            },
            // Industry-Specific
            PolicyTemplate {
                id: "financial-services".to_string(),
                name: "Financial Services".to_string(),
                description: "Stringent security for banking and fintech applications".to_string(),
                category: "Industry".to_string(),
                path: "examples/policies/financial-services.yml".to_string(),
            },
            PolicyTemplate {
                id: "healthcare-provider".to_string(),
                name: "Healthcare Provider".to_string(),
                description: "Comprehensive security for healthcare organizations".to_string(),
                category: "Industry".to_string(),
                path: "examples/policies/healthcare-provider.yml".to_string(),
            },
            PolicyTemplate {
                id: "government".to_string(),
                name: "Government/Defense".to_string(),
                description: "Security policy for government and defense applications".to_string(),
                category: "Industry".to_string(),
                path: "examples/policies/government.yml".to_string(),
            },
            PolicyTemplate {
                id: "saas-cloud".to_string(),
                name: "SaaS/Cloud Provider".to_string(),
                description: "Multi-tenant cloud security policy".to_string(),
                category: "Industry".to_string(),
                path: "examples/policies/saas-cloud.yml".to_string(),
            },
            PolicyTemplate {
                id: "ecommerce".to_string(),
                name: "E-commerce/Retail".to_string(),
                description: "Security policy for online retail and payment systems".to_string(),
                category: "Industry".to_string(),
                path: "examples/policies/ecommerce.yml".to_string(),
            },
            // Framework-Specific
            PolicyTemplate {
                id: "spring-boot".to_string(),
                name: "Spring Boot Microservices".to_string(),
                description: "Optimized for Spring Boot microservice architectures".to_string(),
                category: "Framework".to_string(),
                path: "examples/policies/spring-boot.yml".to_string(),
            },
            PolicyTemplate {
                id: "android".to_string(),
                name: "Android Applications".to_string(),
                description: "Mobile security policy for Android app development".to_string(),
                category: "Framework".to_string(),
                path: "examples/policies/android.yml".to_string(),
            },
            PolicyTemplate {
                id: "microservices".to_string(),
                name: "Microservices Architecture".to_string(),
                description: "Cloud-native security policy for distributed systems".to_string(),
                category: "Framework".to_string(),
                path: "examples/policies/microservices.yml".to_string(),
            },
            PolicyTemplate {
                id: "kubernetes".to_string(),
                name: "Kubernetes Deployments".to_string(),
                description: "Security policy for Kubernetes workloads".to_string(),
                category: "Framework".to_string(),
                path: "examples/policies/kubernetes.yml".to_string(),
            },
            // Development Stages
            PolicyTemplate {
                id: "corporate-permissive".to_string(),
                name: "Development (Permissive)".to_string(),
                description: "Permissive policy for development and testing environments".to_string(),
                category: "Stage".to_string(),
                path: "examples/policies/corporate-permissive.yml".to_string(),
            },
            PolicyTemplate {
                id: "staging".to_string(),
                name: "Staging (Moderate)".to_string(),
                description: "Moderate policy for pre-production testing".to_string(),
                category: "Stage".to_string(),
                path: "examples/policies/staging.yml".to_string(),
            },
            PolicyTemplate {
                id: "production".to_string(),
                name: "Production (Strict)".to_string(),
                description: "Strict policy with zero tolerance for known vulnerabilities".to_string(),
                category: "Stage".to_string(),
                path: "examples/policies/production.yml".to_string(),
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
            // Regulatory
            "pci-dss" => Some(include_str!("../../../examples/policies/pci-dss.yml").to_string()),
            "hipaa" => Some(include_str!("../../../examples/policies/hipaa.yml").to_string()),
            "fedramp-moderate" => {
                Some(include_str!("../../../examples/policies/fedramp-moderate.yml").to_string())
            }
            "soc2" => Some(include_str!("../../../examples/policies/soc2.yml").to_string()),
            "gdpr" => Some(include_str!("../../../examples/policies/gdpr.yml").to_string()),
            "iso27001" => Some(include_str!("../../../examples/policies/iso27001.yml").to_string()),
            "nist-csf" => Some(include_str!("../../../examples/policies/nist-csf.yml").to_string()),
            "cis-benchmarks" => Some(include_str!("../../../examples/policies/cis-benchmarks.yml").to_string()),
            // Industry
            "financial-services" => Some(include_str!("../../../examples/policies/financial-services.yml").to_string()),
            "healthcare-provider" => Some(include_str!("../../../examples/policies/healthcare-provider.yml").to_string()),
            "government" => Some(include_str!("../../../examples/policies/government.yml").to_string()),
            "saas-cloud" => Some(include_str!("../../../examples/policies/saas-cloud.yml").to_string()),
            "ecommerce" => Some(include_str!("../../../examples/policies/ecommerce.yml").to_string()),
            // Framework
            "spring-boot" => Some(include_str!("../../../examples/policies/spring-boot.yml").to_string()),
            "android" => Some(include_str!("../../../examples/policies/android.yml").to_string()),
            "microservices" => Some(include_str!("../../../examples/policies/microservices.yml").to_string()),
            "kubernetes" => Some(include_str!("../../../examples/policies/kubernetes.yml").to_string()),
            // Stage
            "corporate-permissive" => Some(
                include_str!("../../../examples/policies/corporate-permissive.yml").to_string(),
            ),
            "staging" => Some(include_str!("../../../examples/policies/staging.yml").to_string()),
            "production" => Some(include_str!("../../../examples/policies/production.yml").to_string()),
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
        assert_eq!(templates.len(), 20, "Should have 20 policy templates");

        let template_ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
        // Regulatory
        assert!(template_ids.contains(&"pci-dss"));
        assert!(template_ids.contains(&"hipaa"));
        assert!(template_ids.contains(&"fedramp-moderate"));
        assert!(template_ids.contains(&"soc2"));
        assert!(template_ids.contains(&"gdpr"));
        assert!(template_ids.contains(&"iso27001"));
        assert!(template_ids.contains(&"nist-csf"));
        assert!(template_ids.contains(&"cis-benchmarks"));
        // Industry
        assert!(template_ids.contains(&"financial-services"));
        assert!(template_ids.contains(&"healthcare-provider"));
        assert!(template_ids.contains(&"government"));
        assert!(template_ids.contains(&"saas-cloud"));
        assert!(template_ids.contains(&"ecommerce"));
        // Framework
        assert!(template_ids.contains(&"spring-boot"));
        assert!(template_ids.contains(&"android"));
        assert!(template_ids.contains(&"microservices"));
        assert!(template_ids.contains(&"kubernetes"));
        // Stage
        assert!(template_ids.contains(&"corporate-permissive"));
        assert!(template_ids.contains(&"staging"));
        assert!(template_ids.contains(&"production"));
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
        assert_eq!(regulatory.len(), 8, "Should have 8 regulatory templates");

        let industry: Vec<_> = templates
            .iter()
            .filter(|t| t.category == "Industry")
            .collect();
        assert_eq!(industry.len(), 5, "Should have 5 industry templates");

        let framework: Vec<_> = templates
            .iter()
            .filter(|t| t.category == "Framework")
            .collect();
        assert_eq!(framework.len(), 4, "Should have 4 framework templates");

        let stage: Vec<_> = templates
            .iter()
            .filter(|t| t.category == "Stage")
            .collect();
        assert_eq!(stage.len(), 3, "Should have 3 stage templates");
    }

    #[test]
    fn test_embedded_templates_exist() {
        let template_ids = vec![
            // Regulatory
            "pci-dss",
            "hipaa",
            "fedramp-moderate",
            "soc2",
            "gdpr",
            "iso27001",
            "nist-csf",
            "cis-benchmarks",
            // Industry
            "financial-services",
            "healthcare-provider",
            "government",
            "saas-cloud",
            "ecommerce",
            // Framework
            "spring-boot",
            "android",
            "microservices",
            "kubernetes",
            // Stage
            "corporate-permissive",
            "staging",
            "production",
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
