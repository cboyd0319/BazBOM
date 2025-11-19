//! Unified reachability analysis for all supported languages
//!
//! This crate provides call graph analysis and reachability detection
//! for multiple programming languages and build systems.

pub mod js;
pub mod python;
pub mod java;
pub mod go;
pub mod rust;
pub mod ruby;
pub mod php;
pub mod bazel;

// Re-export commonly used types
pub use js::JsReachabilityAnalyzer;
pub use python::PythonReachabilityAnalyzer;
pub use java::JavaReachabilityAnalyzer;
