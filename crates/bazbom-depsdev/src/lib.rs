//! deps.dev API client for BazBOM
//!
//! This crate provides a Rust client for the deps.dev API (https://deps.dev)
//! which offers comprehensive package metadata, dependency graphs, and
//! vulnerability information across multiple ecosystems.

pub mod client;
pub mod models;
pub mod error;

pub use client::DepsDevClient;
pub use models::*;
pub use error::{DepsDevError, Result};
