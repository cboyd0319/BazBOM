//! Demo showcasing BazBOM's beautiful UX improvements
//!
//! Run with: cargo run --release --example ux_demo

use std::thread;
use std::time::Duration;

fn main() {
    println!("\n🎨 BazBOM UX Demo - Developer-Friendly Security Analysis\n");

    demo_scan_summary();
    thread::sleep(Duration::from_secs(2));

    demo_container_scan();
    thread::sleep(Duration::from_secs(2));

    demo_upgrade_intelligence_preview();
}

fn demo_scan_summary() {
    use bazbom::summary::ScanSummary;

    println!("═══════════════════════════════════════════════════");
    println!(" DEMO 1: Scan Summary Dashboard");
    println!("═══════════════════════════════════════════════════\n");

    let summary = ScanSummary {
        dependencies_scanned: 1245,
        vulnerabilities_found: 15,
        critical_count: 2,
        high_count: 5,
        medium_count: 6,
        low_count: 2,
        license_issues: 3,
        policy_violations: 1,
        scan_duration: Duration::from_secs(135), // 2m 15s
        reports_dir: "./bazbom-findings".to_string(),
        uploaded_to_github: true,
        cache_hit: false,
        files_scanned: Some(456),
        targets_analyzed: Some(42),
    };

    summary.print();
}

fn demo_container_scan() {
    use bazbom::container_ux::{ContainerSummary, print_layer_breakdown};

    println!("═══════════════════════════════════════════════════");
    println!(" DEMO 2: Container Image Scanning");
    println!("═══════════════════════════════════════════════════\n");

    // Layer breakdown
    let layers = vec![
        ("sha256:5d0da3dc4634".to_string(), 77.8, 0, 0),   // Base layer
        ("sha256:9a7ddd6b8d0a".to_string(), 23.1, 0, 0),   // System deps
        ("sha256:1b2c3d4e5f6a".to_string(), 150.5, 25, 3), // App layer
        ("sha256:7g8h9i0j1k2l".to_string(), 45.2, 17, 2),  // Dependencies
    ];

    print_layer_breakdown(&layers);

    // Summary
    let summary = ContainerSummary {
        image_name: "mycompany/java-app:v1.2.3".to_string(),
        image_digest: "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        total_layers: 4,
        total_size_mb: 296.6,
        base_image: Some("eclipse-temurin:17-jre-alpine".to_string()),
        java_artifacts: 42,
        vulnerabilities: 5,
        critical_vulns: 0,
        high_vulns: 2,
        medium_vulns: 2,
        low_vulns: 1,
        scan_duration: Duration::from_secs(28),
    };

    summary.print();
}

fn demo_upgrade_intelligence_preview() {
    println!("═══════════════════════════════════════════════════");
    println!(" DEMO 3: Upgrade Intelligence (Preview)");
    println!("═══════════════════════════════════════════════════\n");

    println!("Run 'bazbom fix org.apache.logging.log4j:log4j-core --explain'");
    println!("to see the full Upgrade Intelligence feature!\n");
}
