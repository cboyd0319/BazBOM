use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Handle the `bazbom watch` command
pub fn handle_watch(path: String, interval: u64, critical_only: bool) -> Result<()> {
    println!();
    println!(
        "{}",
        "🔍 Starting continuous security monitoring..."
            .bold()
            .green()
    );
    println!();
    println!("  {} {}", "Watching:".dimmed(), path.bold());
    println!("  {} {} seconds", "Interval:".dimmed(), interval);
    if critical_only {
        println!("  {} Critical vulnerabilities only", "Filter:".dimmed());
    }
    println!();
    println!("{}", "Press Ctrl+C to stop".dimmed());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();

    let project_path = PathBuf::from(&path);
    let mut file_hashes = HashMap::new();

    // Initial scan
    println!(
        "[{}] {} Running initial scan...",
        chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
        "▶".green()
    );
    run_scan(&project_path, critical_only)?;

    // Track dependency files
    let watch_files = find_dependency_files(&project_path)?;
    for file in &watch_files {
        if let Ok(hash) = compute_file_hash(file) {
            file_hashes.insert(file.clone(), hash);
        }
    }

    println!();
    println!(
        "[{}] {} Monitoring {} files for changes...",
        chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
        "👀".cyan(),
        watch_files.len()
    );

    // Watch loop
    loop {
        thread::sleep(Duration::from_secs(interval));

        // Check for changes
        let mut changed_files = Vec::new();
        for file in &watch_files {
            if let Ok(current_hash) = compute_file_hash(file) {
                if let Some(old_hash) = file_hashes.get(file) {
                    if old_hash != &current_hash {
                        changed_files.push(file.clone());
                        file_hashes.insert(file.clone(), current_hash);
                    }
                } else {
                    file_hashes.insert(file.clone(), current_hash);
                    changed_files.push(file.clone());
                }
            }
        }

        if !changed_files.is_empty() {
            println!();
            println!(
                "[{}] {} Detected changes in:",
                chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
                "⚡".yellow()
            );
            for file in &changed_files {
                println!("  {} {}", "•".dimmed(), file.display().to_string().cyan());
            }
            println!();
            println!(
                "[{}] {} Re-scanning...",
                chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
                "🔄".blue()
            );
            run_scan(&project_path, critical_only)?;
        }
    }
}

/// Find dependency files to watch
fn find_dependency_files(project_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Cargo
    let cargo_toml = project_path.join("Cargo.toml");
    if cargo_toml.exists() {
        files.push(cargo_toml);
    }
    let cargo_lock = project_path.join("Cargo.lock");
    if cargo_lock.exists() {
        files.push(cargo_lock);
    }

    // Maven
    let pom = project_path.join("pom.xml");
    if pom.exists() {
        files.push(pom);
    }

    // Gradle
    let gradle_build = project_path.join("build.gradle");
    if gradle_build.exists() {
        files.push(gradle_build);
    }
    let gradle_kts = project_path.join("build.gradle.kts");
    if gradle_kts.exists() {
        files.push(gradle_kts);
    }

    // Node.js
    let package_json = project_path.join("package.json");
    if package_json.exists() {
        files.push(package_json);
    }
    let package_lock = project_path.join("package-lock.json");
    if package_lock.exists() {
        files.push(package_lock);
    }

    // Python
    let requirements = project_path.join("requirements.txt");
    if requirements.exists() {
        files.push(requirements);
    }
    let pipfile = project_path.join("Pipfile");
    if pipfile.exists() {
        files.push(pipfile);
    }

    // Go
    let go_mod = project_path.join("go.mod");
    if go_mod.exists() {
        files.push(go_mod);
    }

    if files.is_empty() {
        anyhow::bail!("No dependency files found to watch");
    }

    Ok(files)
}

/// Compute hash of file for change detection
fn compute_file_hash(path: &Path) -> Result<u64> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let size = metadata.len();

    let mut hasher = DefaultHasher::new();
    modified.hash(&mut hasher);
    size.hash(&mut hasher);

    Ok(hasher.finish())
}

/// Run a quick scan
fn run_scan(project_path: &Path, critical_only: bool) -> Result<()> {
    // Run fast scan
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            "check",
            project_path.to_str().unwrap_or("."),
        ])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!(
                "[{}] {} Scan complete - no critical issues",
                chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
                "✅".green()
            );
        }
        Ok(_) => {
            println!(
                "[{}] {} Vulnerabilities detected!",
                chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
                "⚠️".yellow()
            );

            // Show quick summary
            if let Ok(findings) = fs::read_to_string("./sca_findings.sarif") {
                if let Ok(sarif) = serde_json::from_str::<serde_json::Value>(&findings) {
                    show_quick_summary(&sarif, critical_only);
                }
            }
        }
        Err(e) => {
            println!(
                "[{}] {} Scan failed: {}",
                chrono::Local::now().format("%H:%M:%S").to_string().dimmed(),
                "❌".red(),
                e
            );
        }
    }

    Ok(())
}

/// Show quick vulnerability summary
fn show_quick_summary(sarif: &serde_json::Value, critical_only: bool) {
    let mut critical_count = 0;
    let mut high_count = 0;

    if let Some(runs) = sarif.get("runs").and_then(|r| r.as_array()) {
        for run in runs {
            if let Some(results) = run.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    if let Some(level) = result.get("level").and_then(|l| l.as_str()) {
                        match level {
                            "error" => critical_count += 1,
                            "warning" => high_count += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    if critical_count > 0 {
        println!(
            "  {} {} critical {}",
            "🚨".red(),
            critical_count,
            if critical_count == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
    }

    if !critical_only && high_count > 0 {
        println!(
            "  {} {} high-severity {}",
            "⚠️".yellow(),
            high_count,
            if high_count == 1 { "issue" } else { "issues" }
        );
    }

    if critical_count > 0 || high_count > 0 {
        println!();
        println!("  Run {} for details", "bazbom status".green());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_dependency_files() {
        // This test would need a temp directory with test files
        // Skipping for now
    }

    #[test]
    fn test_compute_file_hash() {
        // Create a temp file and test hashing
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"test content").unwrap();

        let hash1 = compute_file_hash(temp_file.path()).unwrap();
        thread::sleep(Duration::from_millis(10));
        temp_file.write_all(b"more content").unwrap();
        let hash2 = compute_file_hash(temp_file.path()).unwrap();

        assert_ne!(hash1, hash2);
    }
}
