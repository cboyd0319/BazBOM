//! Reconciliation logic for BazBOMScan resources

use crate::{crd::*, error::{OperatorError, Result}, Context};
use k8s_openapi::api::{
    batch::v1::{Job, JobSpec},
    core::v1::{Container, PodSpec, PodTemplateSpec},
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, Patch, PatchParams, PostParams},
    runtime::controller::Action,
    ResourceExt,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Reconcile a BazBOMScan resource
pub async fn reconcile(scan: Arc<BazBOMScan>, ctx: Arc<Context>) -> Result<Action> {
    let namespace = scan
        .namespace()
        .ok_or_else(|| OperatorError::InvalidConfig("BazBOMScan must have a namespace".to_string()))?;
    let name = scan.name_any();

    info!("Reconciling BazBOMScan {}/{}", namespace, name);

    // Create scan job
    if let Err(e) = create_scan_job(&scan, &namespace, &ctx.client).await {
        error!("Failed to create scan job for {}/{}: {}", namespace, name, e);
        update_scan_status(&scan, &namespace, &ctx.client, "Failed", Some(&e.to_string())).await?;
        return Ok(Action::requeue(Duration::from_secs(300)));
    }

    // Update status to Running
    update_scan_status(&scan, &namespace, &ctx.client, "Running", None).await?;

    // Requeue after 5 minutes to check status
    Ok(Action::requeue(Duration::from_secs(300)))
}

/// Error policy for reconciliation failures
pub fn error_policy(_scan: Arc<BazBOMScan>, _error: &OperatorError, _ctx: Arc<Context>) -> Action {
    // Retry after 1 minute on error
    Action::requeue(Duration::from_secs(60))
}

/// Create a Kubernetes Job to run the scan
async fn create_scan_job(
    scan: &BazBOMScan,
    namespace: &str,
    client: &kube::Client,
) -> Result<()> {
    let name = scan.name_any();
    let job_name = format!("bazbom-scan-{}", name);

    info!("Creating scan job: {}", job_name);

    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);

    // Check if job already exists
    if let Ok(_existing) = jobs.get(&job_name).await {
        info!("Job {} already exists, skipping creation", job_name);
        return Ok(());
    }

    // Build command arguments
    let mut args = vec![
        "bazbom".to_string(),
        "scan".to_string(),
        ".".to_string(),
        "--format".to_string(),
        scan.spec.output_format.clone(),
    ];

    // Add build system if specified
    if let Some(ref build_system) = scan.spec.build_system {
        args.push("--build-system".to_string());
        args.push(build_system.clone());
    }

    // Add scan options
    if scan.spec.scan_options.scan_containers {
        args.push("--scan-containers".to_string());
    }
    if scan.spec.scan_options.reachability_analysis {
        args.push("--reachability".to_string());
    }
    if scan.spec.scan_options.ml_prioritize {
        args.push("--ml-prioritize".to_string());
    }
    if scan.spec.scan_options.llm_fixes {
        args.push("--llm".to_string());
    }

    // Create job spec
    let job = Job {
        metadata: ObjectMeta {
            name: Some(job_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some({
                let mut labels = BTreeMap::new();
                labels.insert("app".to_string(), "bazbom-scan".to_string());
                labels.insert("scan-name".to_string(), name.clone());
                labels
            }),
            ..Default::default()
        },
        spec: Some(JobSpec {
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some({
                        let mut labels = BTreeMap::new();
                        labels.insert("app".to_string(), "bazbom-scan".to_string());
                        labels.insert("scan-name".to_string(), name.clone());
                        labels
                    }),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    containers: vec![Container {
                        name: "bazbom-scanner".to_string(),
                        image: Some("bazbom/bazbom:latest".to_string()),
                        command: Some(vec!["sh".to_string(), "-c".to_string()]),
                        args: Some(vec![args.join(" ")]),
                        ..Default::default()
                    }],
                    restart_policy: Some("Never".to_string()),
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the job
    jobs.create(&PostParams::default(), &job)
        .await
        .map_err(|e| OperatorError::JobCreationFailed(e.to_string()))?;

    info!("Successfully created scan job: {}", job_name);
    Ok(())
}

/// Update the status of a BazBOMScan
async fn update_scan_status(
    scan: &BazBOMScan,
    namespace: &str,
    client: &kube::Client,
    phase: &str,
    error_message: Option<&str>,
) -> Result<()> {
    let name = scan.name_any();
    let scans: Api<BazBOMScan> = Api::namespaced(client.clone(), namespace);

    // Build status update
    let mut status = scan.status.clone().unwrap_or_default();
    status.phase = phase.to_string();
    status.last_scan_time = Some(chrono::Utc::now().to_rfc3339());
    if let Some(msg) = error_message {
        status.error_message = Some(msg.to_string());
    }

    // Create patch
    let patch = serde_json::json!({
        "status": status
    });

    scans
        .patch_status(
            &name,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
        .map_err(|e| OperatorError::StatusUpdateFailed(e.to_string()))?;

    info!("Updated status for {}/{} to {}", namespace, name, phase);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_options_default() {
        let opts = ScanOptions::default();
        assert!(!opts.scan_containers);
        assert!(!opts.reachability_analysis);
        assert!(!opts.ml_prioritize);
        assert!(!opts.llm_fixes);
    }

    #[test]
    fn test_vulnerability_counts_default() {
        let counts = VulnerabilityCounts::default();
        assert_eq!(counts.critical, 0);
        assert_eq!(counts.high, 0);
        assert_eq!(counts.medium, 0);
        assert_eq!(counts.low, 0);
    }
}
