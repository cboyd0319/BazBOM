//! Ecosystem-specific scanners for package managers and build systems.
//!
//! Each ecosystem module contains the parsing and scanning logic for a specific
//! package manager or build system.

pub mod npm;
pub mod python;
pub mod go;
pub mod rust;
pub mod ruby;
pub mod php;
pub mod maven;
pub mod gradle;
pub mod bazel;

// Re-exports will be added as Scanner trait is implemented
