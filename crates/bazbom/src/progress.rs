//! Beautiful progress indicators for BazBOM operations
//!
//! This module provides reusable progress bars, spinners, and multi-progress
//! displays for long-running operations like scans, API calls, and analysis.

use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

/// Progress indicator for orchestrated scans with multiple phases
pub struct ScanProgress {
    #[allow(dead_code)]
    multi: Arc<MultiProgress>,
    phases: Vec<ProgressBar>,
}

impl ScanProgress {
    /// Create a new scan progress tracker with named phases
    pub fn new(phase_names: &[&str]) -> Self {
        let multi = Arc::new(MultiProgress::new());

        // Print header
        println!();
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────────────┐".bright_blue()
        );
        println!(
            "{} {} {}",
            "│".bright_blue(),
            "SCAN Running Security Scan".bold().bright_cyan(),
            "                                  │".bright_blue()
        );
        println!(
            "{}",
            "├─────────────────────────────────────────────────────────────────┤".bright_blue()
        );

        let mut phases = Vec::new();

        for name in phase_names {
            let pb = multi.add(ProgressBar::new(100));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(&format!(
                        "│ {{spinner:.cyan}} {{prefix:18}} {{bar:30.cyan/blue}} {{pos:>3}}% {{msg}}{}",
                        " │".bright_blue()
                    ))
                    .unwrap()
                    .progress_chars("█▓▒░")
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            pb.set_prefix(name.to_string());
            pb.set_message("PAUSED  Queued".dimmed().to_string());
            pb.enable_steady_tick(Duration::from_millis(80));
            phases.push(pb);
        }

        Self { multi, phases }
    }

    /// Mark a phase as started
    pub fn start_phase(&self, index: usize, message: &str) {
        if let Some(pb) = self.phases.get(index) {
            pb.set_position(0);
            pb.set_message(format!("{} {}", "⏳".yellow(), message));
        }
    }

    /// Update progress for a phase (0-100)
    pub fn update_phase(&self, index: usize, progress: u64, message: &str) {
        if let Some(pb) = self.phases.get(index) {
            pb.set_position(progress);
            if progress < 100 {
                pb.set_message(format!("{} {}", "⏳".yellow(), message));
            }
        }
    }

    /// Mark a phase as completed
    pub fn complete_phase(&self, index: usize, message: &str) {
        if let Some(pb) = self.phases.get(index) {
            pb.set_position(100);
            pb.set_message(format!("{} {}", "OK".green(), message));
            pb.finish();
        }
    }

    /// Mark a phase as failed
    pub fn fail_phase(&self, index: usize, error: &str) {
        if let Some(pb) = self.phases.get(index) {
            pb.set_message(format!("{} {}", "FAIL".red(), error));
            pb.finish();
        }
    }

    /// Mark a phase as skipped
    pub fn skip_phase(&self, index: usize, reason: &str) {
        if let Some(pb) = self.phases.get(index) {
            pb.set_message(format!("{} {}", "⊘".dimmed(), reason.dimmed()));
            pb.finish();
        }
    }

    /// Finish all phases and print footer
    pub fn finish(&self, summary: &str) {
        for pb in &self.phases {
            pb.finish();
        }

        println!(
            "{}",
            "├─────────────────────────────────────────────────────────────────┤".bright_blue()
        );
        println!(
            "{} {} {}",
            "│".bright_blue(),
            summary.bright_white().bold(),
            " │".bright_blue()
        );
        println!(
            "{}",
            "└─────────────────────────────────────────────────────────────────┘".bright_blue()
        );
        println!();
    }
}

/// Spinner for API calls and network operations
pub struct ApiSpinner {
    spinner: ProgressBar,
}

impl ApiSpinner {
    /// Create a new API spinner with a message
    pub fn new(message: &str) -> Self {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("   {spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(80));

        Self { spinner }
    }

    /// Update the spinner message
    pub fn set_message(&self, message: &str) {
        self.spinner.set_message(message.to_string());
    }

    /// Finish with success message
    pub fn finish_success(&self, message: &str) {
        self.spinner
            .finish_with_message(format!("{} {}", "OK".green(), message));
    }

    /// Finish with error message
    pub fn finish_error(&self, message: &str) {
        self.spinner
            .finish_with_message(format!("{} {}", "FAIL".red(), message));
    }

    /// Finish and clear
    pub fn finish(&self) {
        self.spinner.finish_and_clear();
    }
}

/// Progress bar for counting operations (like scanning targets)
pub struct CountingProgress {
    bar: ProgressBar,
}

impl CountingProgress {
    /// Create a new counting progress bar
    pub fn new(total: u64, operation: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "PKG {}\n{{bar:50.cyan/blue}} {{pos:>7}}/{{len:7}} | {{elapsed_precise}} | ETA: {{eta}}",
                    operation.bold()
                ))
                .unwrap()
                .progress_chars("█▓▒░"),
        );

        Self { bar }
    }

    /// Increment progress by 1
    pub fn inc(&self) {
        self.bar.inc(1);
    }

    /// Set progress to a specific value
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Set a custom message
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        self.bar
            .finish_with_message("Complete!".green().to_string());
    }

    /// Finish and clear
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

/// Simple spinner for quick operations
pub fn simple_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template(&format!("   {} {{spinner:.green}} {{msg}}", "⚡".yellow()))
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// Multi-step progress for sequential operations
pub struct MultiStepProgress {
    steps: Vec<String>,
    current: usize,
    spinner: ProgressBar,
}

impl MultiStepProgress {
    /// Create a new multi-step progress tracker
    pub fn new(steps: Vec<String>) -> Self {
        let spinner = ProgressBar::new(steps.len() as u64);
        spinner.set_style(
            ProgressStyle::default_bar()
                .template("   [{bar:25.cyan/blue}] {pos}/{len} {spinner:.cyan} {msg}")
                .unwrap()
                .progress_chars("█▓░")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));

        Self {
            steps,
            current: 0,
            spinner,
        }
    }

    /// Start the next step
    pub fn next_step(&mut self) -> Option<&str> {
        if self.current < self.steps.len() {
            let step = &self.steps[self.current];
            self.spinner.set_position(self.current as u64);
            self.spinner.set_message(step.clone());
            self.current += 1;
            Some(step)
        } else {
            None
        }
    }

    /// Finish all steps
    pub fn finish(&self) {
        self.spinner
            .finish_with_message("All steps complete!".green().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_scan_progress() {
        let progress = ScanProgress::new(&["SCA Analysis", "Semgrep", "CodeQL"]);

        progress.start_phase(0, "Starting...");
        thread::sleep(Duration::from_millis(100));
        progress.update_phase(0, 50, "Analyzing...");
        thread::sleep(Duration::from_millis(100));
        progress.complete_phase(0, "Done");

        progress.skip_phase(1, "Not configured");
        progress.complete_phase(2, "Complete");

        progress.finish("✨ Scan complete!");
    }

    #[test]
    fn test_api_spinner() {
        let spinner = ApiSpinner::new("Fetching data...");
        thread::sleep(Duration::from_millis(100));
        spinner.set_message("Processing...");
        thread::sleep(Duration::from_millis(100));
        spinner.finish_success("Data fetched!");
    }

    #[test]
    fn test_counting_progress() {
        let progress = CountingProgress::new(100, "Scanning targets");
        for i in 0..100 {
            progress.set_position(i);
            if i % 10 == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        progress.finish();
    }
}
