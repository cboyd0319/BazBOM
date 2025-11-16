//! BazBOM Core Library
//!
//! This is the main library crate that provides internal modules for the BazBOM CLI.
//! It contains analyzers, scanners, and utilities for JVM ecosystem SBOM generation.
//!
//! This crate is primarily for internal use and testing. Public API users should
//! use the individual specialized crates like `bazbom-core`, `bazbom-formats`, etc.

pub mod advisory;
pub mod analyzers;
pub mod android;
pub mod ant;
pub mod backup;
pub mod batch_fixer;
pub mod bazel;
pub mod buildr;
pub mod cli;
pub mod cli_validation;
pub mod clojure;
pub mod command_timeout;
pub mod config;
pub mod container_ux;
pub mod context;
pub mod enrich;
pub mod explore;
pub mod fixes;
pub mod groovy;
pub mod hooks;
pub mod incremental;
pub mod init;
pub mod interactive_fix;
pub mod kotlin_multiplatform;
pub mod notifications;
pub mod parallel;
pub mod performance;
pub mod pipeline;
pub mod policy_integration;
pub mod progress;
pub mod publish;
pub mod reachability;
pub mod reachability_cache;
pub mod remediation;
pub mod sbt;
pub mod scan;
pub mod scan_cache;
pub mod scan_orchestrator;
pub mod security;
pub mod shading;
pub mod summary;
pub mod team;
pub mod test_runner;
pub mod toolchain;
