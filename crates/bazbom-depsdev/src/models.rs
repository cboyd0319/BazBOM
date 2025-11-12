use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Package system identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum System {
    Maven,
    Npm,
    Cargo,
    PyPI,
    Go,
    NuGet,
    #[serde(rename = "RUBYGEMS")]
    RubyGems,
}

impl System {
    pub fn as_str(&self) -> &str {
        match self {
            System::Maven => "MAVEN",
            System::Npm => "NPM",
            System::Cargo => "CARGO",
            System::PyPI => "PYPI",
            System::Go => "GO",
            System::NuGet => "NUGET",
            System::RubyGems => "RUBYGEMS",
        }
    }
}

/// Version key uniquely identifies a package version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionKey {
    pub system: System,
    pub name: String,
    pub version: String,
}

impl VersionKey {
    pub fn to_purl(&self) -> String {
        let system_lower = match self.system {
            System::Maven => "maven",
            System::Npm => "npm",
            System::Cargo => "cargo",
            System::PyPI => "pypi",
            System::Go => "golang",
            System::NuGet => "nuget",
            System::RubyGems => "gem",
        };

        // Maven uses namespace/name format
        if self.system == System::Maven {
            let parts: Vec<&str> = self.name.split(':').collect();
            if parts.len() == 2 {
                return format!("pkg:{}/{}/{}@{}", system_lower, parts[0], parts[1], self.version);
            }
        }

        format!("pkg:{}/{}@{}", system_lower, self.name, self.version)
    }
}

/// Links associated with a package version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    /// Where the package is hosted (e.g., Maven Central URL)
    #[serde(default)]
    pub origins: Vec<String>,

    /// Homepage URL
    pub homepage: Option<String>,

    /// Source repository URL (often GitHub)
    pub repository: Option<String>,
}

/// Security advisory affecting a package version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    /// OSV identifier (e.g., "GHSA-xxxx-yyyy-zzzz")
    pub key: String,

    /// URL to advisory details
    pub url: String,

    /// Advisory title
    pub title: String,

    /// CVE aliases
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Complete version information from deps.dev
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub version_key: VersionKey,

    /// When this version was published
    #[serde(with = "chrono::serde::ts_seconds")]
    pub published_at: DateTime<Utc>,

    /// Is this the default/latest version?
    pub is_default: bool,

    /// SPDX license expressions
    #[serde(default)]
    pub licenses: Vec<String>,

    /// Security advisories
    #[serde(default)]
    pub advisories: Vec<Advisory>,

    /// External links
    pub links: Links,
}

/// Dependency edge type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Relation {
    /// Direct dependency
    Direct,
    /// Indirect (transitive) dependency
    Indirect,
    /// Self reference
    #[serde(rename = "SELF")]
    SelfRelation,
}

/// A node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyNode {
    pub version_key: VersionKey,

    /// How this dependency is related to the root
    pub relation: Relation,

    /// Errors encountered while resolving this dependency
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Resolved dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All nodes in the dependency graph
    pub nodes: Vec<DependencyNode>,

    /// Edges between nodes (simplified - real API has more detail)
    #[serde(default)]
    pub edges: Vec<DependencyEdge>,
}

/// Edge between two dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEdge {
    pub from_node: usize,     // Index into nodes array
    pub to_node: usize,       // Index into nodes array
    pub requirement: String,  // Version requirement
}

impl DependencyGraph {
    /// Get all direct dependencies
    pub fn direct_dependencies(&self) -> Vec<&DependencyNode> {
        self.nodes
            .iter()
            .filter(|n| n.relation == Relation::Direct)
            .collect()
    }

    /// Find a node by package name
    pub fn find_node(&self, package_name: &str) -> Option<&DependencyNode> {
        self.nodes
            .iter()
            .find(|n| n.version_key.name == package_name)
    }
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub package_key: PackageKey,

    /// All available versions
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageKey {
    pub system: System,
    pub name: String,
}
