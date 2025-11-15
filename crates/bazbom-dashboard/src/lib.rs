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
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

pub mod export;
mod models;
mod routes;

pub use export::{export_to_html, Vulnerability};
pub use models::*;

/// Dashboard application state
#[derive(Clone)]
pub struct AppState {
    /// Path to BazBOM cache directory
    pub cache_dir: PathBuf,
    /// Path to project root
    pub project_root: PathBuf,
    /// Optional bearer token for API authentication
    pub auth_token: Option<String>,
}

/// Dashboard configuration
pub struct DashboardConfig {
    pub port: u16,
    pub cache_dir: PathBuf,
    pub project_root: PathBuf,
    /// Optional bearer token for API authentication
    /// If None, authentication is disabled (localhost-only recommended)
    pub auth_token: Option<String>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            cache_dir: PathBuf::from(".bazbom/cache"),
            project_root: PathBuf::from("."),
            auth_token: std::env::var("BAZBOM_DASHBOARD_TOKEN").ok(),
        }
    }
}

/// Authentication middleware
/// Validates Bearer token if configured in AppState
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no auth token is configured, allow the request (localhost-only mode)
    let Some(ref expected_token) = state.auth_token else {
        return Ok(next.run(req).await);
    };

    // Check for Authorization header
    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];
            if token == expected_token {
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Start the dashboard server
pub async fn start_dashboard(config: DashboardConfig) -> Result<()> {
    let state = Arc::new(AppState {
        cache_dir: config.cache_dir,
        project_root: config.project_root,
        auth_token: config.auth_token.clone(),
    });

    // Configure CORS - restrict to localhost only for security
    let cors = CorsLayer::new()
        .allow_origin(
            format!("http://127.0.0.1:{}", config.port)
                .parse::<HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Security warning if no auth token is set
    if config.auth_token.is_none() {
        println!("[!] WARNING: Dashboard running WITHOUT authentication");
        println!("[!] Ensure the dashboard is NOT exposed to untrusted networks");
        println!("[!] Set BAZBOM_DASHBOARD_TOKEN environment variable to enable authentication");
        println!();
    }

    // Build the application router with security layers
    let app = Router::new()
        // API routes (protected by auth middleware)
        .route("/api/dashboard/summary", get(routes::get_dashboard_summary))
        .route("/api/dependencies/graph", get(routes::get_dependency_graph))
        .route("/api/vulnerabilities", get(routes::get_vulnerabilities))
        .route("/api/sbom", get(routes::get_sbom))
        .route("/api/team/dashboard", get(routes::get_team_dashboard))
        // Health check (unprotected)
        .route("/health", get(health_check))
        // Static files (future: React frontend)
        .nest_service("/", ServeDir::new("static"))
        // State and layers
        .with_state(state.clone())
        // Security middleware
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        // Security headers
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(cors);

    // Start server
    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    println!("[*] BazBOM Dashboard running at http://{}", addr);
    println!("[*] Security Score: Loading...");
    println!("[!] Vulnerabilities: Analyzing...");
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

        assert!(
            static_dir.join("index.html").exists(),
            "index.html should exist in static directory"
        );
        assert!(
            static_dir.join("css/dashboard.css").exists(),
            "dashboard.css should exist in static/css directory"
        );
        assert!(
            static_dir.join("js/dashboard.js").exists(),
            "dashboard.js should exist in static/js directory"
        );
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
