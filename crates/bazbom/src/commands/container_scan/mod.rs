//! Container Security Scanning Command
//!
//! Complete container security analysis with:
//! - Multi-tool parallel scanning (Trivy, Grype, Syft, Dockle, Dive, TruffleHog)
//! - SBOM generation
//! - Vulnerability scanning with KEV/EPSS enrichment
//! - Layer attribution (which layer introduced each vulnerability)
//! - Priority scoring (P0-P4)
//! - Reachability analysis
//! - Beautiful UX with progress tracking

// Sub-modules
mod dependency_graph;
mod display;
mod enrichment;
mod types;

// Re-exports
pub(crate) use display::{
    apply_filter, create_github_issues, display_baseline_comparison, display_image_comparison,
    display_results, load_baseline, save_baseline,
};
pub(crate) use enrichment::{
    analyze_upgrade_impact, enrich_vulnerabilities, enrich_vulnerabilities_with_os,
    format_difficulty_label,
};
pub(crate) use types::{
    detect_ecosystem, ActionItem, ComplianceResults, ComplianceStatus, DockerLayerMetadata,
    PackageEcosystem, ProvenanceStatus, QuickWin, ReachabilitySummary, SignatureStatus,
};
pub use types::{
    ContainerScanOptions, ContainerScanResults, LayerInfo, UpgradeRecommendation, VulnerabilityInfo,
};

// Import the implementation from handler.rs
mod handler;
pub use handler::handle_container_scan;
