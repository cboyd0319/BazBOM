//! Unified reachability analysis for all supported languages
//!
//! This crate provides call graph analysis and reachability detection
//! for multiple programming languages and build systems.

pub mod bazel;
pub mod go;
pub mod java;
pub mod js;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;

// Re-export commonly used types
pub use java::JavaReachabilityAnalyzer;
pub use js::JsReachabilityAnalyzer;
pub use python::PythonReachabilityAnalyzer;
