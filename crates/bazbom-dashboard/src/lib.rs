//! BazBOM Web Dashboard
//!
//! Self-hosted web dashboard for interactive SBOM and vulnerability visualization.
//! 
//! Features:
//! - Security score dashboard
//! - Interactive dependency graph (D3.js)
//! - Vulnerability timeline
//! - SBOM explorer
//! - Executive reports (PDF)

use anyhow::Result;
use axum::{
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

mod models;
mod routes;

pub use models::*;

/// Dashboard application state
#[derive(Clone)]
pub struct AppState {
    /// Path to BazBOM cache directory
    pub cache_dir: PathBuf,
    /// Path to project root
    pub project_root: PathBuf,
}

/// Dashboard configuration
pub struct DashboardConfig {
    pub port: u16,
    pub cache_dir: PathBuf,
    pub project_root: PathBuf,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            cache_dir: PathBuf::from(".bazbom/cache"),
            project_root: PathBuf::from("."),
        }
    }
}

/// Start the dashboard server
pub async fn start_dashboard(config: DashboardConfig) -> Result<()> {
    let state = Arc::new(AppState {
        cache_dir: config.cache_dir,
        project_root: config.project_root,
    });

    // Build the application router
    let app = Router::new()
        // API routes
        .route("/api/dashboard/summary", get(routes::get_dashboard_summary))
        .route("/api/dependencies/graph", get(routes::get_dependency_graph))
        .route("/api/vulnerabilities", get(routes::get_vulnerabilities))
        .route("/api/sbom", get(routes::get_sbom))
        // Health check
        .route("/health", get(health_check))
        // Static files (future: React frontend)
        .nest_service("/", ServeDir::new("static"))
        // State and layers
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 BazBOM Dashboard running at http://{}", addr);
    println!("📊 Security Score: Loading...");
    println!("⚠️  Vulnerabilities: Analyzing...");
    println!();
    println!("Press Ctrl+C to stop");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DashboardConfig::default();
        assert_eq!(config.port, 3000);
        assert_eq!(config.cache_dir, PathBuf::from(".bazbom/cache"));
    }

    #[test]
    fn test_static_files_exist() {
        // Verify that the static dashboard files exist
        let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
        
        assert!(static_dir.join("index.html").exists(), 
            "index.html should exist in static directory");
        assert!(static_dir.join("css/dashboard.css").exists(), 
            "dashboard.css should exist in static/css directory");
        assert!(static_dir.join("js/dashboard.js").exists(), 
            "dashboard.js should exist in static/js directory");
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState {
            cache_dir: PathBuf::from(".bazbom/cache"),
            project_root: PathBuf::from("."),
        };
        
        assert_eq!(state.cache_dir, PathBuf::from(".bazbom/cache"));
        assert_eq!(state.project_root, PathBuf::from("."));
    }
}
