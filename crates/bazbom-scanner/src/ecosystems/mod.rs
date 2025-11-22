//! Ecosystem-specific scanners for package managers and build systems.
//!
//! Each ecosystem module contains the parsing and scanning logic for a specific
//! package manager or build system.

pub mod bazel;
pub mod go;
pub mod gradle;
pub mod maven;
pub mod npm;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;

// Re-exports will be added as Scanner trait is implemented
