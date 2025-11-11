//! Beautiful UX for container image scanning
//!
//! Provides rich, visual feedback during container scans with layer-by-layer progress
//! and beautiful summaries.

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Container scan progress tracker
pub struct ContainerScanProgress {
    image_name: String,
    total_layers: usize,
    current_layer: usize,
    spinner: ProgressBar,
}

impl ContainerScanProgress {
    /// Create a new container scan progress tracker
    pub fn new(image_name: &str, total_layers: usize) -> Self {
        // Print header
        println!();
        println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".bright_blue().bold());
        println!("{} {} {}",
            "║".bright_blue().bold(),
            format!("🐳 CONTAINER SCAN: {}", image_name).bright_cyan().bold(),
            " ║".bright_blue().bold()
        );
        println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".bright_blue().bold());
        println!();

        let spinner = ProgressBar::new(total_layers as u64);
        spinner.set_style(
            ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/blue}] {pos}/{len} layers | {msg}")
                .unwrap()
                .progress_chars("█▓▒░")
        );

        Self {
            image_name: image_name.to_string(),
            total_layers,
            current_layer: 0,
            spinner,
        }
    }

    /// Start scanning a layer
    pub fn start_layer(&mut self, layer_id: &str, size_mb: f64) {
        self.current_layer += 1;
        self.spinner.set_position(self.current_layer as u64);
        self.spinner.set_message(format!(
            "Scanning layer {} ({:.1} MB)...",
            &layer_id[..12.min(layer_id.len())],
            size_mb
        ));
    }

    /// Complete a layer with findings count
    pub fn complete_layer(&mut self, artifacts_found: usize, vulnerabilities: usize) {
        let status = if vulnerabilities > 0 {
            format!("⚠️  {} vulns", vulnerabilities).yellow()
        } else if artifacts_found > 0 {
            format!("✓ {} artifacts", artifacts_found).green()
        } else {
            "✓ clean".dimmed()
        };

        self.spinner.set_message(format!("Layer {}/{}: {}",
            self.current_layer,
            self.total_layers,
            status
        ));
    }

    /// Finish the scan
    pub fn finish(&self) {
        self.spinner.finish_with_message("Scan complete!".green().to_string());
        println!();
    }
}

/// Container scan summary
#[derive(Debug, Default)]
pub struct ContainerSummary {
    pub image_name: String,
    pub image_digest: String,
    pub total_layers: usize,
    pub total_size_mb: f64,
    pub base_image: Option<String>,
    pub java_artifacts: usize,
    pub vulnerabilities: usize,
    pub critical_vulns: usize,
    pub high_vulns: usize,
    pub medium_vulns: usize,
    pub low_vulns: usize,
    pub scan_duration: Duration,
}

impl ContainerSummary {
    /// Print beautiful container summary
    pub fn print(&self) {
        println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{} {:^67} {}",
            "║".bright_cyan().bold(),
            "🐳 CONTAINER SCAN SUMMARY",
            "║".bright_cyan().bold()
        );
        println!("{}", "╠═══════════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Image info
        self.print_row("Image:", &self.image_name.bright_white().bold().to_string());
        let digest_display = if self.image_digest.len() > 16 {
            format!("{}...", &self.image_digest[..16])
        } else {
            self.image_digest.clone()
        };
        self.print_row("Digest:", &digest_display.dimmed().to_string());

        if let Some(ref base) = self.base_image {
            self.print_row("Base Image:", &base.bright_white().to_string());
        }

        println!("{}", "╠═══════════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Layer info
        self.print_row("Total Layers:", &self.total_layers.to_string().bright_white().bold().to_string());
        self.print_row("Total Size:", &format!("{:.1} MB", self.total_size_mb).bright_white().bold().to_string());

        println!("{}", "╠═══════════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Findings
        self.print_row("Java Artifacts:", &self.java_artifacts.to_string().bright_white().bold().to_string());

        let vuln_display = if self.vulnerabilities == 0 {
            format!("{} {}", "✅", "0".green().bold())
        } else if self.critical_vulns > 0 {
            format!("{} {}", "🔴", self.vulnerabilities.to_string().red().bold())
        } else if self.high_vulns > 0 {
            format!("{} {}", "🟠", self.vulnerabilities.to_string().yellow().bold())
        } else {
            format!("{} {}", "🟡", self.vulnerabilities.to_string().yellow())
        };

        self.print_row("Vulnerabilities:", &vuln_display);

        if self.vulnerabilities > 0 {
            if self.critical_vulns > 0 {
                self.print_row("  ├─ Critical:", &format!("{:>3}  {}", self.critical_vulns, "🔴"));
            }
            if self.high_vulns > 0 {
                self.print_row("  ├─ High:", &format!("{:>3}  {}", self.high_vulns, "🟠"));
            }
            if self.medium_vulns > 0 {
                self.print_row("  ├─ Medium:", &format!("{:>3}  {}", self.medium_vulns, "🟡"));
            }
            if self.low_vulns > 0 {
                self.print_row("  └─ Low:", &format!("{:>3}  {}", self.low_vulns, "🟢"));
            }
        }

        println!("{}", "╠═══════════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Performance
        self.print_row("Scan Duration:", &crate::summary::format_duration(self.scan_duration).bright_white().bold().to_string());

        println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".bright_cyan().bold());
        println!();

        // Next steps
        self.print_next_steps();
    }

    fn print_row(&self, label: &str, value: &str) {
        let value_display_len = console::strip_ansi_codes(value).len();
        let padding = 45usize.saturating_sub(value_display_len);

        println!("{} {:<23} {:>width$} {}",
            "║".bright_cyan().bold(),
            label,
            value,
            "║".bright_cyan().bold(),
            width = padding
        );
    }

    fn print_next_steps(&self) {
        println!("{}", "Next steps:".bold().bright_white());

        if self.vulnerabilities > 0 {
            println!("  {} Vulnerabilities found in container image",
                "⚠️ ".yellow()
            );
            println!("    Run {} to analyze Java dependencies",
                "'bazbom scan <extracted-layers>'".bright_white().bold()
            );
        }

        if self.java_artifacts > 0 {
            println!("  {} {} Java artifacts detected",
                "☕".to_string(),
                self.java_artifacts
            );
            println!("    Consider scanning with {} for full dependency analysis",
                "'--with-semgrep'".bright_white().bold()
            );
        }

        if self.vulnerabilities == 0 && self.java_artifacts == 0 {
            println!("  {} {} No Java artifacts or vulnerabilities found",
                "✨".green(),
                "".to_string()
            );
        }

        println!();
    }
}

/// Print visual layer breakdown
pub fn print_layer_breakdown(layers: &[(String, f64, usize, usize)]) {
    println!("{}", "Layer Breakdown:".bold().bright_white());
    println!();

    let max_size = layers.iter().map(|(_, size, _, _)| *size).fold(0.0f64, f64::max);

    for (i, (layer_id, size_mb, artifacts, vulns)) in layers.iter().enumerate() {
        let is_last = i == layers.len() - 1;
        let tree_char = if is_last { "└─" } else { "├─" };

        // Create size bar (proportional to layer size)
        let bar_width = ((size_mb / max_size) * 30.0) as usize;
        let bar = "█".repeat(bar_width);

        let status = if *vulns > 0 {
            format!("⚠️  {} vulns", vulns).red()
        } else if *artifacts > 0 {
            format!("✓ {} artifacts", artifacts).green()
        } else {
            "clean".dimmed()
        };

        println!("  {} {} {:<30} {:.1} MB | {}",
            tree_char.cyan(),
            &layer_id[..12.min(layer_id.len())].dimmed(),
            bar.cyan(),
            size_mb,
            status
        );
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_summary() {
        let summary = ContainerSummary {
            image_name: "myapp:latest".to_string(),
            image_digest: "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            total_layers: 8,
            total_size_mb: 245.6,
            base_image: Some("eclipse-temurin:17-jre".to_string()),
            java_artifacts: 42,
            vulnerabilities: 5,
            critical_vulns: 0,
            high_vulns: 2,
            medium_vulns: 2,
            low_vulns: 1,
            scan_duration: Duration::from_secs(45),
        };

        summary.print();
    }

    #[test]
    fn test_layer_breakdown() {
        let layers = vec![
            ("sha256:abc123".to_string(), 50.2, 0, 0),
            ("sha256:def456".to_string(), 100.5, 15, 2),
            ("sha256:ghi789".to_string(), 25.1, 5, 0),
            ("sha256:jkl012".to_string(), 70.8, 22, 3),
        ];

        print_layer_breakdown(&layers);
    }
}
