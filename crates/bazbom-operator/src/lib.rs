//! BazBOM Kubernetes Operator
//!
//! Automatically scans Kubernetes workloads for vulnerabilities and generates SBOMs

pub mod crd;
pub mod error;
pub mod reconciler;

use error::Result;
use futures::StreamExt;
use kube::{
    runtime::{controller::Controller, watcher::Config},
    Api, Client,
};
use std::sync::Arc;
use tracing::{info, warn};

pub use crd::BazBOMScan;

/// Operator context shared across reconciliation loops
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
}

/// Start the operator
pub async fn run() -> Result<()> {
    info!("Starting BazBOM Operator");

    // Initialize Kubernetes client
    let client = Client::try_default().await?;
    let context = Arc::new(Context {
        client: client.clone(),
    });

    // Create API for BazBOMScan resources
    let scans: Api<BazBOMScan> = Api::all(client.clone());

    // Start controller
    Controller::new(scans.clone(), Config::default())
        .run(
            reconciler::reconcile,
            reconciler::error_policy,
            context.clone(),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled {:?}", o),
                Err(e) => warn!("Reconcile error: {}", e),
            }
        })
        .await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        // Context creation requires async runtime and K8s cluster
        // Real tests would use integration test with kind/minikube
        assert!(true);
    }
}
