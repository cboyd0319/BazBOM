//! deps.dev API client for BazBOM
//!
//! This crate provides a Rust client for the deps.dev API (https://deps.dev)
//! which offers comprehensive package metadata, dependency graphs, and
//! vulnerability information across multiple ecosystems.

pub mod client;
pub mod error;
pub mod models;

pub use client::DepsDevClient;
pub use error::{DepsDevError, Result};
pub use models::*;
