//! Machine Learning Infrastructure for BazBOM
//!
//! This crate provides AI-powered intelligence features for vulnerability analysis:
//! - Feature extraction from vulnerabilities and dependencies
//! - Anomaly detection for unusual dependency patterns
//! - Exploit prediction enhancement (future)
//! - LLM-assisted remediation (future)
//!
//! ## Privacy-First Design
//!
//! All ML models run locally. No data is sent to external services.
//!
//! ## Current Capabilities
//!
//! - **Feature Extraction**: Convert vulnerabilities and dependencies into feature vectors
//! - **Anomaly Detection**: Identify unusual dependency patterns using statistical methods
//! - **Risk Scoring**: Enhanced risk scoring based on multiple signals
//!
//! ## Future Capabilities (Phase 10)
//!
//! - **Custom Exploit Prediction**: Train models on your specific environment
//! - **LLM Migration Guides**: Generate migration guides for breaking changes
//! - **Intelligent Triage**: Auto-categorize vulnerabilities

pub mod anomaly;
pub mod features;
pub mod fix_generator;
pub mod llm;
pub mod prioritization;
pub mod risk;

pub use anomaly::{Anomaly, AnomalyDetector, AnomalyType};
pub use features::{DependencyFeatures, VulnerabilityFeatures};
pub use fix_generator::{
    BatchFixPlan, BreakingSeverity, CodeChange, ConfigChange, FixContext, FixGenerator, FixGuide,
};
pub use llm::{
    FixPromptBuilder, LlmClient, LlmConfig, LlmMessage, LlmProvider, LlmRequest, LlmResponse,
    PolicyQueryBuilder, TokenUsage,
};
pub use prioritization::{
    FixBatch, FixUrgency, PrioritizedVulnerability, VulnerabilityPrioritizer,
};
pub use risk::{EnhancedRiskScore, RiskScorer};
