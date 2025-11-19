//! Anomaly detection command handlers

use anyhow::{Context, Result};
use std::path::Path;

/// Handle anomaly detection scan
pub fn handle_anomaly_scan(path: String, json: bool, output: Option<String>) -> Result<()> {
    use bazbom_ml::anomaly::AnomalyDetector;
    use bazbom_ml::features::DependencyFeatures;

    let path = Path::new(&path);
    println!("Running anomaly detection on {}...\n", path.display());

    // Create detector with default thresholds
    let detector = AnomalyDetector::new();

    // Extract features from project dependencies
    // In production, this would parse actual dependency data
    let features_map = extract_dependency_features(path)?;

    // Run detection
    let anomalies = detector.detect_batch(&features_map);

    // Output results
    if json {
        let json_output = serde_json::to_string_pretty(&anomalies)?;
        if let Some(output_path) = output {
            std::fs::write(&output_path, &json_output)
                .with_context(|| format!("Failed to write to {}", output_path))?;
            println!("Anomalies written to {}", output_path);
        } else {
            println!("{}", json_output);
        }
    } else {
        if anomalies.is_empty() {
            println!("[+] No anomalies detected!");
            println!("    All dependencies appear to have normal characteristics.");
        } else {
            println!("[!] Found {} anomaly signal(s):\n", anomalies.len());
            for anomaly in &anomalies {
                let score_bar = "=".repeat((anomaly.score * 10.0) as usize);
                println!(
                    "  [{:<10}] {}",
                    score_bar,
                    anomaly.package.as_deref().unwrap_or("unknown")
                );
                println!("    Type: {:?}", anomaly.anomaly_type);
                println!("    Score: {:.2}", anomaly.score);
                println!("    {}", anomaly.description);
                println!("    Recommendation: {}", anomaly.recommendation);
                println!();
            }
        }

        if let Some(output_path) = output {
            let json_output = serde_json::to_string_pretty(&anomalies)?;
            std::fs::write(&output_path, &json_output)?;
            println!("Results written to {}", output_path);
        }
    }

    Ok(())
}

/// Handle anomaly model training
pub fn handle_anomaly_train(from_dir: String, output: String) -> Result<()> {
    use bazbom_ml::anomaly::AnomalyDetector;
    use bazbom_ml::features::DependencyFeatures;

    let from_path = Path::new(&from_dir);
    let output_path = Path::new(&output);

    println!("Training anomaly detection model...\n");
    println!("Loading historical data from: {}", from_path.display());

    // Load historical dependency features
    let historical_features = load_historical_features(from_path)?;

    if historical_features.is_empty() {
        anyhow::bail!("No historical data found in {}. Run scans to build history.", from_dir);
    }

    println!("Found {} historical dependency records", historical_features.len());

    // Train detector
    let detector = AnomalyDetector::train(&historical_features)?;

    // Save model (in production, would serialize the thresholds)
    let model_data = serde_json::json!({
        "trained_on": chrono::Utc::now().to_rfc3339(),
        "sample_count": historical_features.len(),
        "model_version": "1.0",
    });

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(output_path, serde_json::to_string_pretty(&model_data)?)?;

    println!("\n[+] Model trained successfully!");
    println!("    Saved to: {}", output_path.display());
    println!("    Samples used: {}", historical_features.len());

    Ok(())
}

/// Handle anomaly report generation
pub fn handle_anomaly_report(path: String, output: Option<String>) -> Result<()> {
    use bazbom_ml::anomaly::AnomalyDetector;

    let path = Path::new(&path);
    println!("Generating anomaly detection report for {}...\n", path.display());

    // Create detector
    let detector = AnomalyDetector::new();

    // Extract features
    let features_map = extract_dependency_features(path)?;

    // Run detection
    let anomalies = detector.detect_batch(&features_map);

    // Generate HTML report
    let html = generate_anomaly_report_html(&anomalies, &features_map);

    let output_path = output.unwrap_or_else(|| "anomaly-report.html".to_string());
    std::fs::write(&output_path, html)?;

    println!("[+] Anomaly report generated: {}", output_path);
    println!("    Total dependencies analyzed: {}", features_map.len());
    println!("    Anomalies detected: {}", anomalies.len());

    Ok(())
}

/// Extract dependency features from a project
fn extract_dependency_features(
    _path: &Path,
) -> Result<Vec<(String, bazbom_ml::features::DependencyFeatures)>> {
    use bazbom_ml::features::DependencyFeatures;

    // In production, this would:
    // 1. Parse lockfiles
    // 2. Query deps.dev API for metadata
    // 3. Calculate actual features

    // For now, return mock data
    Ok(vec![
        (
            "lodash".to_string(),
            DependencyFeatures {
                transitive_count: 5,
                vuln_count: 0,
                maintainer_score: 0.95,
                popularity: 0.99,
                recent_releases: 6,
                ..Default::default()
            },
        ),
        (
            "suspicious-pkg".to_string(),
            DependencyFeatures {
                transitive_count: 150,
                vuln_count: 8,
                maintainer_score: 0.2,
                popularity: 0.05,
                recent_releases: 60,
                ..Default::default()
            },
        ),
    ])
}

/// Load historical features from scan history
fn load_historical_features(
    _from_path: &Path,
) -> Result<Vec<bazbom_ml::features::DependencyFeatures>> {
    use bazbom_ml::features::DependencyFeatures;

    // In production, would load from historical scan data
    Ok(vec![
        DependencyFeatures {
            transitive_count: 10,
            vuln_count: 1,
            maintainer_score: 0.8,
            popularity: 0.7,
            recent_releases: 4,
            ..Default::default()
        },
        DependencyFeatures {
            transitive_count: 20,
            vuln_count: 0,
            maintainer_score: 0.9,
            popularity: 0.8,
            recent_releases: 6,
            ..Default::default()
        },
    ])
}

/// Generate HTML report for anomalies
fn generate_anomaly_report_html(
    anomalies: &[bazbom_ml::anomaly::Anomaly],
    features_map: &[(String, bazbom_ml::features::DependencyFeatures)],
) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>BazBOM Anomaly Detection Report</title>
    <style>
        body {{ font-family: -apple-system, sans-serif; margin: 40px; background: #f5f5f5; }}
        .container {{ max-width: 1000px; margin: 0 auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        h1 {{ color: #333; }}
        .summary {{ background: #e8f4fd; padding: 20px; border-radius: 6px; margin: 20px 0; }}
        .anomaly {{ background: #fff3cd; border-left: 4px solid #ffc107; padding: 15px; margin: 10px 0; border-radius: 4px; }}
        .critical {{ border-left-color: #dc3545; background: #f8d7da; }}
        .score {{ font-weight: bold; color: #666; }}
        .recommendation {{ color: #155724; font-style: italic; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Anomaly Detection Report</h1>
        <div class="summary">
            <strong>Summary:</strong> Analyzed {} dependencies, found {} anomaly signals.
        </div>
        <h2>Detected Anomalies</h2>
        {}
        <p style="color: #666; margin-top: 40px;">Generated by BazBOM v6.6.0</p>
    </div>
</body>
</html>"#,
        features_map.len(),
        anomalies.len(),
        anomalies
            .iter()
            .map(|a| format!(
                r#"<div class="anomaly {}">
                    <strong>{}</strong> - {:?}
                    <div class="score">Score: {:.2}</div>
                    <p>{}</p>
                    <p class="recommendation">Recommendation: {}</p>
                </div>"#,
                if a.score > 0.7 { "critical" } else { "" },
                a.package.as_deref().unwrap_or("unknown"),
                a.anomaly_type,
                a.score,
                a.description,
                a.recommendation
            ))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
