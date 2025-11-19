//! Go reachability analyzer

use crate::error::{GoReachabilityError, Result};
use crate::models::ReachabilityReport;
use std::path::Path;
use std::process::Command;
use tracing::info;

pub struct GoReachabilityAnalyzer;

impl GoReachabilityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a Go project for reachability
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        info!("Starting Go reachability analysis");

        // Find the go-analyzer tool
        let analyzer_path = self.find_go_analyzer()?;

        // Run the Go analyzer tool
        info!("Running go-analyzer at {:?}", analyzer_path);
        let output = Command::new(&analyzer_path)
            .arg(project_root.to_str().unwrap())
            .output()
            .map_err(|e| GoReachabilityError::AnalyzerFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GoReachabilityError::AnalyzerFailed(format!(
                "Go analyzer failed: {}",
                stderr
            )));
        }

        // Parse JSON output
        let report_json = String::from_utf8_lossy(&output.stdout);
        let mut report: ReachabilityReport = serde_json::from_str(&report_json)?;

        // Convert Vec to HashSet for reachable/unreachable functions
        report.reachable_functions = report
            .reachable_functions
            .iter()
            .cloned()
            .collect();
        
        report.unreachable_functions = report
            .unreachable_functions
            .iter()
            .cloned()
            .collect();

        info!(
            "Go analysis complete: {}/{} functions reachable",
            report.reachable_functions.len(),
            report.all_functions.len()
        );

        Ok(report)
    }

    /// Find the go-analyzer binary
    fn find_go_analyzer(&self) -> Result<std::path::PathBuf> {
        // Try multiple locations
        let candidates = vec![
            // In tools directory (development)
            std::path::PathBuf::from("tools/go-analyzer/go-analyzer"),
            // In project root tools directory
            std::path::PathBuf::from("../tools/go-analyzer/go-analyzer"),
            std::path::PathBuf::from("../../tools/go-analyzer/go-analyzer"),
            // In PATH
            std::path::PathBuf::from("go-analyzer"),
        ];

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try to build it if main.go exists
        let main_go = std::path::PathBuf::from("tools/go-analyzer/main.go");
        if main_go.exists() {
            info!("Building go-analyzer from source");
            let build_output = Command::new("go")
                .args(&["build", "-o", "tools/go-analyzer/go-analyzer"])
                .current_dir("tools/go-analyzer")
                .output();

            if let Ok(output) = build_output {
                if output.status.success() {
                    return Ok(std::path::PathBuf::from("tools/go-analyzer/go-analyzer"));
                }
            }
        }

        Err(GoReachabilityError::AnalyzerNotFound(
            "go-analyzer binary not found. Run 'cd tools/go-analyzer && go build' to build it."
                .to_string(),
        ))
    }
}
