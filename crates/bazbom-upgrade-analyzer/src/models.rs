use serde::{Deserialize, Serialize};
use std::fmt;

/// Overall risk level for an upgrade
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            RiskLevel::Low => "✅",
            RiskLevel::Medium => "⚠️",
            RiskLevel::High => "🚨",
            RiskLevel::Critical => "💥",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.emoji(), self.label())
    }
}

/// A single breaking change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub description: String,
    pub version: String,
    pub auto_fixable: bool,
    pub affected_apis: Vec<String>,
    pub migration_hint: Option<String>,
}

/// Why a dependency needs to be upgraded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpgradeReason {
    VersionAlignment { required_by: String },
    NewDependency,
    IncompatibleVersion { conflict: String },
    SecurityFix { cve: String },
    Removed,
}

impl fmt::Display for UpgradeReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpgradeReason::VersionAlignment { required_by } => {
                write!(f, "Version alignment (required by {})", required_by)
            }
            UpgradeReason::NewDependency => write!(f, "New dependency added"),
            UpgradeReason::IncompatibleVersion { conflict } => {
                write!(f, "Version conflict: {}", conflict)
            }
            UpgradeReason::SecurityFix { cve } => write!(f, "Security fix for {}", cve),
            UpgradeReason::Removed => write!(f, "Dependency removed"),
        }
    }
}

/// A required dependency upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredUpgrade {
    pub package: String,
    pub from_version: String,
    pub to_version: String,
    pub reason: UpgradeReason,
    pub breaking_changes: Vec<BreakingChange>,
    pub risk_level: RiskLevel,
    pub optional: bool,
}

/// Complete analysis of an upgrade including transitive changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeAnalysis {
    /// The package being upgraded
    pub target_package: String,
    pub from_version: String,
    pub to_version: String,

    /// Breaking changes directly in the target package
    pub direct_breaking_changes: Vec<BreakingChange>,

    /// All dependencies that must also be upgraded
    pub required_upgrades: Vec<RequiredUpgrade>,

    /// Overall risk considering ALL changes (direct + transitive)
    pub overall_risk: RiskLevel,

    /// Estimated effort in hours
    pub estimated_effort_hours: f32,

    /// GitHub repository URL (if found)
    pub github_repo: Option<String>,

    /// Migration guide URL (if found)
    pub migration_guide_url: Option<String>,

    /// Additional compatibility notes
    pub compatibility_notes: Vec<String>,

    /// Success rate from community data (if available)
    pub success_rate: Option<f32>,
}

impl UpgradeAnalysis {
    /// Total number of breaking changes (direct + transitive)
    pub fn total_breaking_changes(&self) -> usize {
        self.direct_breaking_changes.len()
            + self
                .required_upgrades
                .iter()
                .map(|u| u.breaking_changes.len())
                .sum::<usize>()
    }

    /// Total packages affected (including the target)
    pub fn total_packages_affected(&self) -> usize {
        1 + self.required_upgrades.len()
    }

    /// Get all breaking changes across all packages
    pub fn all_breaking_changes(&self) -> Vec<(&str, &BreakingChange)> {
        let mut changes = vec![];

        // Direct changes
        for change in &self.direct_breaking_changes {
            changes.push((self.target_package.as_str(), change));
        }

        // Transitive changes
        for upgrade in &self.required_upgrades {
            for change in &upgrade.breaking_changes {
                changes.push((upgrade.package.as_str(), change));
            }
        }

        changes
    }

    /// Is this a safe upgrade with no breaking changes?
    pub fn is_safe(&self) -> bool {
        self.total_breaking_changes() == 0 && self.overall_risk <= RiskLevel::Low
    }
}

/// Compatibility notes about an upgrade
#[derive(Debug, Clone)]
pub struct CompatibilityNote {
    pub level: NoteLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum NoteLevel {
    Info,
    Warning,
    Error,
}
