//! Performance monitoring and metrics
//!
//! Provides utilities for tracking scan performance, memory usage,
//! and generating performance reports for optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total scan duration
    pub total_duration: Duration,
    /// Time spent on SBOM generation
    pub sbom_generation_duration: Option<Duration>,
    /// Time spent on vulnerability scanning
    pub vulnerability_scan_duration: Option<Duration>,
    /// Time spent on reachability analysis
    pub reachability_duration: Option<Duration>,
    /// Time spent on threat detection
    pub threat_detection_duration: Option<Duration>,
    /// Number of dependencies scanned
    pub dependencies_count: usize,
    /// Number of vulnerabilities found
    pub vulnerabilities_count: usize,
    /// Whether cache was used
    pub cache_hit: bool,
    /// Build system type
    pub build_system: String,
    /// Project size indicators
    pub project_metrics: ProjectMetrics,
}

/// Project size and complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    /// Number of source files
    pub source_files: usize,
    /// Number of modules/subprojects
    pub modules: usize,
    /// Total lines of code (estimated)
    pub lines_of_code: Option<usize>,
}

impl Default for ProjectMetrics {
    fn default() -> Self {
        Self {
            source_files: 0,
            modules: 1,
            lines_of_code: None,
        }
    }
}

/// Performance monitor for tracking scan phases
pub struct PerformanceMonitor {
    start_time: Instant,
    phase_times: HashMap<String, Duration>,
    current_phase: Option<(String, Instant)>,
    dependencies_count: usize,
    vulnerabilities_count: usize,
    cache_hit: bool,
    build_system: String,
    project_metrics: ProjectMetrics,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(build_system: String) -> Self {
        Self {
            start_time: Instant::now(),
            phase_times: HashMap::new(),
            current_phase: None,
            dependencies_count: 0,
            vulnerabilities_count: 0,
            cache_hit: false,
            build_system,
            project_metrics: ProjectMetrics::default(),
        }
    }

    /// Start tracking a phase
    pub fn start_phase(&mut self, phase: &str) {
        if let Some((prev_phase, prev_start)) = self.current_phase.take() {
            // End previous phase
            let duration = prev_start.elapsed();
            self.phase_times.insert(prev_phase, duration);
        }

        self.current_phase = Some((phase.to_string(), Instant::now()));
    }

    /// End the current phase
    pub fn end_phase(&mut self) {
        if let Some((phase, start)) = self.current_phase.take() {
            let duration = start.elapsed();
            self.phase_times.insert(phase, duration);
        }
    }

    /// Record dependencies count
    pub fn set_dependencies_count(&mut self, count: usize) {
        self.dependencies_count = count;
    }

    /// Record vulnerabilities count
    pub fn set_vulnerabilities_count(&mut self, count: usize) {
        self.vulnerabilities_count = count;
    }

    /// Mark as cache hit
    pub fn mark_cache_hit(&mut self) {
        self.cache_hit = true;
    }

    /// Set project metrics
    pub fn set_project_metrics(&mut self, metrics: ProjectMetrics) {
        self.project_metrics = metrics;
    }

    /// Get total elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Generate final metrics report
    pub fn finalize(mut self) -> PerformanceMetrics {
        // End any ongoing phase
        self.end_phase();

        PerformanceMetrics {
            total_duration: self.start_time.elapsed(),
            sbom_generation_duration: self.phase_times.get("sbom_generation").copied(),
            vulnerability_scan_duration: self.phase_times.get("vulnerability_scan").copied(),
            reachability_duration: self.phase_times.get("reachability_analysis").copied(),
            threat_detection_duration: self.phase_times.get("threat_detection").copied(),
            dependencies_count: self.dependencies_count,
            vulnerabilities_count: self.vulnerabilities_count,
            cache_hit: self.cache_hit,
            build_system: self.build_system,
            project_metrics: self.project_metrics,
        }
    }

    /// Get performance summary as human-readable string
    pub fn summary(&self) -> String {
        let total = self.start_time.elapsed();
        let mut lines = vec![
            format!("Performance Summary"),
            format!("=================="),
            format!("Total time: {:.2}s", total.as_secs_f64()),
            format!("Build system: {}", self.build_system),
            format!("Dependencies: {}", self.dependencies_count),
            format!("Vulnerabilities: {}", self.vulnerabilities_count),
            format!("Cache hit: {}", self.cache_hit),
        ];

        if !self.phase_times.is_empty() {
            lines.push(String::new());
            lines.push("Phase breakdown:".to_string());
            let mut phases: Vec<_> = self.phase_times.iter().collect();
            phases.sort_by_key(|(_, duration)| std::cmp::Reverse(*duration));

            for (phase, duration) in phases {
                let pct = (duration.as_secs_f64() / total.as_secs_f64()) * 100.0;
                lines.push(format!(
                    "  {}: {:.2}s ({:.1}%)",
                    phase,
                    duration.as_secs_f64(),
                    pct
                ));
            }
        }

        lines.join("\n")
    }
}

/// Calculate estimated time savings from incremental analysis
pub fn estimate_time_savings(
    affected_targets: usize,
    total_targets: usize,
    avg_time_per_target: Duration,
) -> Duration {
    let skipped_targets = total_targets.saturating_sub(affected_targets);
    avg_time_per_target * skipped_targets as u32
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs < 1.0 {
        format!("{:.0}ms", duration.as_millis())
    } else if total_secs < 60.0 {
        format!("{:.2}s", total_secs)
    } else {
        let mins = (total_secs / 60.0).floor();
        let secs = total_secs % 60.0;
        format!("{:.0}m {:.0}s", mins, secs)
    }
}

/// Performance comparison between two scans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceComparison {
    /// Baseline metrics
    pub baseline: PerformanceMetrics,
    /// Current metrics
    pub current: PerformanceMetrics,
    /// Percentage improvement (positive = faster)
    pub improvement_pct: f64,
    /// Absolute time saved
    pub time_saved: Duration,
}

impl PerformanceComparison {
    /// Compare two performance metrics
    pub fn compare(baseline: PerformanceMetrics, current: PerformanceMetrics) -> Self {
        let baseline_secs = baseline.total_duration.as_secs_f64();
        let current_secs = current.total_duration.as_secs_f64();

        let improvement_pct = if baseline_secs > 0.0 {
            ((baseline_secs - current_secs) / baseline_secs) * 100.0
        } else {
            0.0
        };

        let time_saved = if baseline_secs > current_secs {
            Duration::from_secs_f64(baseline_secs - current_secs)
        } else {
            Duration::from_secs(0)
        };

        Self {
            baseline,
            current,
            improvement_pct,
            time_saved,
        }
    }

    /// Generate comparison report
    pub fn report(&self) -> String {
        let mut lines = vec![
            format!("Performance Comparison"),
            format!("====================="),
            format!(""),
            format!(
                "Baseline: {}",
                format_duration(self.baseline.total_duration)
            ),
            format!("Current:  {}", format_duration(self.current.total_duration)),
            format!(""),
        ];

        if self.improvement_pct > 0.0 {
            lines.push(format!(
                "[+] {} faster ({:.1}% improvement)",
                format_duration(self.time_saved),
                self.improvement_pct
            ));
        } else if self.improvement_pct < 0.0 {
            lines.push(format!(
                "[!] {} slower ({:.1}% regression)",
                format_duration(Duration::from_secs_f64(-self.time_saved.as_secs_f64())),
                -self.improvement_pct
            ));
        } else {
            lines.push("No change in performance".to_string());
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_basic() {
        let mut monitor = PerformanceMonitor::new("maven".to_string());
        monitor.start_phase("sbom_generation");
        std::thread::sleep(Duration::from_millis(10));
        monitor.end_phase();

        let metrics = monitor.finalize();
        assert_eq!(metrics.build_system, "maven");
        assert!(metrics.sbom_generation_duration.is_some());
        assert!(metrics.total_duration.as_millis() >= 10);
    }

    #[test]
    fn test_performance_monitor_multiple_phases() {
        let mut monitor = PerformanceMonitor::new("gradle".to_string());

        monitor.start_phase("sbom_generation");
        std::thread::sleep(Duration::from_millis(5));
        monitor.start_phase("vulnerability_scan");
        std::thread::sleep(Duration::from_millis(5));
        monitor.end_phase();

        let metrics = monitor.finalize();
        assert!(metrics.sbom_generation_duration.is_some());
        assert!(metrics.vulnerability_scan_duration.is_some());
    }

    #[test]
    fn test_performance_monitor_cache_hit() {
        let mut monitor = PerformanceMonitor::new("bazel".to_string());
        monitor.mark_cache_hit();
        monitor.set_dependencies_count(100);
        monitor.set_vulnerabilities_count(5);

        let metrics = monitor.finalize();
        assert!(metrics.cache_hit);
        assert_eq!(metrics.dependencies_count, 100);
        assert_eq!(metrics.vulnerabilities_count, 5);
    }

    #[test]
    fn test_estimate_time_savings() {
        let avg_time = Duration::from_secs(1);
        let savings = estimate_time_savings(10, 100, avg_time);
        assert_eq!(savings.as_secs(), 90);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs_f64(5.5)), "5.50s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
    }

    #[test]
    fn test_performance_comparison_improvement() {
        let baseline = PerformanceMetrics {
            total_duration: Duration::from_secs(100),
            sbom_generation_duration: None,
            vulnerability_scan_duration: None,
            reachability_duration: None,
            threat_detection_duration: None,
            dependencies_count: 1000,
            vulnerabilities_count: 10,
            cache_hit: false,
            build_system: "maven".to_string(),
            project_metrics: ProjectMetrics::default(),
        };

        let current = PerformanceMetrics {
            total_duration: Duration::from_secs(50),
            ..baseline.clone()
        };

        let comparison = PerformanceComparison::compare(baseline, current);
        assert_eq!(comparison.improvement_pct, 50.0);
        assert_eq!(comparison.time_saved.as_secs(), 50);
    }

    #[test]
    fn test_performance_comparison_regression() {
        let baseline = PerformanceMetrics {
            total_duration: Duration::from_secs(50),
            sbom_generation_duration: None,
            vulnerability_scan_duration: None,
            reachability_duration: None,
            threat_detection_duration: None,
            dependencies_count: 1000,
            vulnerabilities_count: 10,
            cache_hit: false,
            build_system: "maven".to_string(),
            project_metrics: ProjectMetrics::default(),
        };

        let current = PerformanceMetrics {
            total_duration: Duration::from_secs(75),
            ..baseline.clone()
        };

        let comparison = PerformanceComparison::compare(baseline, current);
        assert!(comparison.improvement_pct < 0.0);
    }

    #[test]
    fn test_project_metrics_default() {
        let metrics = ProjectMetrics::default();
        assert_eq!(metrics.source_files, 0);
        assert_eq!(metrics.modules, 1);
        assert!(metrics.lines_of_code.is_none());
    }

    #[test]
    fn test_performance_monitor_summary() {
        let mut monitor = PerformanceMonitor::new("bazel".to_string());
        monitor.set_dependencies_count(50);
        monitor.set_vulnerabilities_count(3);

        let summary = monitor.summary();
        assert!(summary.contains("Performance Summary"));
        assert!(summary.contains("bazel"));
        assert!(summary.contains("Dependencies: 50"));
        assert!(summary.contains("Vulnerabilities: 3"));
    }
}
