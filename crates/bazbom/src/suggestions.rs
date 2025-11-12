//! Smart suggestions after scan completion
//!
//! Analyzes scan results and provides contextual recommendations

use crate::output;
use std::path::Path;
use std::time::Duration;

/// Scan metadata for generating suggestions
pub struct ScanMetadata {
    pub duration: Duration,
    pub total_vulns: usize,
    pub critical_vulns: usize,
    pub high_vulns: usize,
    pub reachable_vulns: usize,
    pub reachability_enabled: bool,
    pub baseline_exists: bool,
    pub is_ci: bool,
    pub json_output: bool,
}

impl ScanMetadata {
    /// Generate contextual suggestions based on scan results
    pub fn generate_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Performance suggestions
        if self.duration.as_secs() > 30 && !self.is_ci {
            suggestions.push(format!(
                "This scan took {:.0}s - try 'bazbom check' for < 10s scans during development",
                self.duration.as_secs()
            ));
        }

        if self.reachability_enabled && self.duration.as_secs() > 60 {
            suggestions.push(
                "Reachability analysis is slow - use 'bazbom scan --fast' for quick checks"
                    .to_string(),
            );
        }

        // Reachability suggestions
        if !self.reachability_enabled && self.total_vulns > 10 {
            let reduction_estimate = ((self.total_vulns as f64) * 0.75).round() as usize;
            suggestions.push(format!(
                "Enable reachability analysis to reduce noise - estimated ~{} fewer alerts: 'bazbom scan --reachability'",
                reduction_estimate
            ));
        }

        // CI/CD suggestions
        if !self.is_ci && self.total_vulns > 0 {
            suggestions.push(
                "Add BazBOM to your CI pipeline: 'bazbom install ci-github' or 'bazbom ci'"
                    .to_string(),
            );
        }

        // Baseline/diff suggestions
        if !self.baseline_exists && self.total_vulns > 0 {
            suggestions.push(
                "Save this scan as a baseline to track changes over time: 'cp sca_findings.sarif baseline.json'".to_string(),
            );
        }

        if self.baseline_exists && !self.json_output {
            suggestions.push(
                "Use diff mode to see what changed since last scan: 'bazbom scan --diff --baseline baseline.json'"
                    .to_string(),
            );
        }

        // Severity-based suggestions
        if self.critical_vulns > 0 {
            suggestions.push(format!(
                "Fix {} critical vulnerabilities immediately - they pose the highest risk",
                self.critical_vulns
            ));
        }

        if self.reachable_vulns > 0 {
            suggestions.push(format!(
                "Prioritize {} reachable vulnerabilities - they're actively exploitable in your code",
                self.reachable_vulns
            ));
        }

        // Fix command suggestions
        if self.total_vulns > 0 && self.total_vulns <= 5 {
            suggestions.push(
                "Small number of vulnerabilities - try auto-fix: 'bazbom fix --suggest'".to_string(),
            );
        }

        // Success suggestions
        if self.total_vulns == 0 {
            suggestions.push(
                "No vulnerabilities found! Run periodic scans to stay secure: 'bazbom watch'"
                    .to_string(),
            );
            suggestions.push(
                "Share your security posture: 'bazbom report generate --format pdf'".to_string(),
            );
        }

        // Profile suggestions
        if !Path::new("bazbom.toml").exists() {
            suggestions.push(
                "Create bazbom.toml for custom scan profiles: 'bazbom init'".to_string(),
            );
        }

        suggestions
    }

    /// Print suggestions after scan
    pub fn print_suggestions(&self) {
        let suggestions = self.generate_suggestions();
        if !suggestions.is_empty() {
            output::print_suggestions(suggestions);
        }
    }
}

/// Quick suggestion generator for common scenarios
pub struct QuickSuggestions;

impl QuickSuggestions {
    /// Suggest next steps after finding vulnerabilities
    pub fn after_vulnerabilities_found(count: usize, reachable: usize) -> Vec<String> {
        let mut suggestions = vec![
            format!("Found {} vulnerabilities ({} reachable)", count, reachable),
            String::new(),
            "📋 Next steps:".to_string(),
        ];

        suggestions.push("  1. Prioritize reachable vulnerabilities: 'bazbom explore'".to_string());
        suggestions.push("  2. Get detailed explanations: 'bazbom explain CVE-2024-XXXX'".to_string());
        suggestions.push("  3. Apply automated fixes: 'bazbom fix --suggest'".to_string());
        suggestions.push("  4. Generate report: 'bazbom report generate'".to_string());

        suggestions
    }

    /// Suggest CI integration
    pub fn ci_integration() -> Vec<String> {
        vec![
            "💡 Add BazBOM to your CI pipeline:".to_string(),
            String::new(),
            "GitHub Actions:".to_string(),
            "  bazbom install ci-github".to_string(),
            String::new(),
            "GitLab CI:".to_string(),
            "  bazbom install ci-gitlab".to_string(),
            String::new(),
            "Manual integration:".to_string(),
            "  bazbom ci --format sarif -o ./artifacts".to_string(),
        ]
    }

    /// Suggest performance improvements
    pub fn performance_improvements(scan_time_secs: u64) -> Vec<String> {
        if scan_time_secs < 15 {
            return vec![];
        }

        let mut suggestions = vec![
            format!("⚡ Scan took {}s - here's how to speed it up:", scan_time_secs),
            String::new(),
        ];

        if scan_time_secs > 60 {
            suggestions.push("  • Use fast mode: 'bazbom scan --fast'".to_string());
            suggestions.push("  • Enable incremental scans: 'bazbom pr'".to_string());
        }

        if scan_time_secs > 30 {
            suggestions.push("  • Use quick command: 'bazbom check'".to_string());
            suggestions.push("  • Target specific module: 'bazbom scan --target core'".to_string());
        }

        suggestions
    }

    /// Suggest reachability analysis
    pub fn reachability_benefits(total_vulns: usize) -> Vec<String> {
        if total_vulns < 5 {
            return vec![];
        }

        let estimated_reduction = (total_vulns as f64 * 0.75).round() as usize;

        vec![
            format!("🎯 Enable reachability to reduce alerts by ~{}:", estimated_reduction),
            String::new(),
            "  bazbom scan --reachability".to_string(),
            String::new(),
            "Why it helps:".to_string(),
            "  • Only shows vulnerabilities in code you actually use".to_string(),
            "  • Reduces false positives by 70-90%".to_string(),
            "  • Helps prioritize fixes".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_scan_no_suggestions() {
        let metadata = ScanMetadata {
            duration: Duration::from_secs(5),
            total_vulns: 0,
            critical_vulns: 0,
            high_vulns: 0,
            reachable_vulns: 0,
            reachability_enabled: false,
            baseline_exists: false,
            is_ci: false,
            json_output: false,
        };

        let suggestions = metadata.generate_suggestions();
        assert!(!suggestions.is_empty()); // Should suggest creating baseline, etc.
    }

    #[test]
    fn test_slow_scan_suggestions() {
        let metadata = ScanMetadata {
            duration: Duration::from_secs(45),
            total_vulns: 10,
            critical_vulns: 2,
            high_vulns: 5,
            reachable_vulns: 3,
            reachability_enabled: false,
            baseline_exists: false,
            is_ci: false,
            json_output: false,
        };

        let suggestions = metadata.generate_suggestions();
        assert!(suggestions.iter().any(|s| s.contains("took 45s")));
        assert!(suggestions.iter().any(|s| s.contains("reachability")));
    }

    #[test]
    fn test_quick_suggestions() {
        let suggestions = QuickSuggestions::after_vulnerabilities_found(15, 5);
        assert!(!suggestions.is_empty());

        let perf_suggestions = QuickSuggestions::performance_improvements(65);
        assert!(!perf_suggestions.is_empty());

        let reachability_suggestions = QuickSuggestions::reachability_benefits(20);
        assert!(!reachability_suggestions.is_empty());
    }
}
