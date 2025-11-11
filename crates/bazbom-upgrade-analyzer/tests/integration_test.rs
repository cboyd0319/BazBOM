use bazbom_upgrade_analyzer::UpgradeAnalyzer;

/// Integration test for the upgrade analyzer
///
/// This test requires network access to deps.dev and GitHub APIs.
#[tokio::test]
#[ignore] // Requires network access
async fn test_log4j_upgrade_analysis() {
    let mut analyzer = UpgradeAnalyzer::new().expect("Failed to create analyzer");

    let analysis = analyzer
        .analyze_upgrade(
            "org.apache.logging.log4j:log4j-core",
            "2.17.0",
            "2.20.0",
        )
        .await
        .expect("Failed to analyze upgrade");

    // Should detect that this is a minor version upgrade
    assert_eq!(analysis.target_package, "org.apache.logging.log4j:log4j-core");
    assert_eq!(analysis.from_version, "2.17.0");
    assert_eq!(analysis.to_version, "2.20.0");

    // Should find required upgrades (log4j-api)
    assert!(!analysis.required_upgrades.is_empty(),
        "Should find required dependency upgrades");

    // Should find log4j-api as a required upgrade
    let log4j_api_upgrade = analysis.required_upgrades.iter()
        .find(|u| u.package.contains("log4j-api"));
    assert!(log4j_api_upgrade.is_some(),
        "Should require log4j-api upgrade");

    // Print results for manual verification
    println!("\n{:?}", analysis);
    println!("\nOverall risk: {:?}", analysis.overall_risk);
    println!("Total breaking changes: {}", analysis.total_breaking_changes());
    println!("Required upgrades: {}", analysis.required_upgrades.len());
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_spring_boot_major_upgrade() {
    let mut analyzer = UpgradeAnalyzer::new().expect("Failed to create analyzer");

    let analysis = analyzer
        .analyze_upgrade(
            "org.springframework.boot:spring-boot-starter-web",
            "2.7.0",
            "3.2.0",
        )
        .await
        .expect("Failed to analyze upgrade");

    // Major version upgrade should have high risk
    assert!(matches!(
        analysis.overall_risk,
        bazbom_upgrade_analyzer::RiskLevel::High | bazbom_upgrade_analyzer::RiskLevel::Critical
    ));

    // Should have significant effort estimate
    assert!(analysis.estimated_effort_hours > 4.0,
        "Major upgrade should require significant effort");

    // Print results
    println!("\n{:?}", analysis);
    println!("Estimated effort: {} hours", analysis.estimated_effort_hours);
}

#[test]
fn test_safe_upgrade_detection() {
    use bazbom_upgrade_analyzer::{RiskLevel, UpgradeAnalysis};

    let analysis = UpgradeAnalysis {
        target_package: "test-package".to_string(),
        from_version: "1.0.0".to_string(),
        to_version: "1.0.1".to_string(),
        direct_breaking_changes: vec![],
        required_upgrades: vec![],
        overall_risk: RiskLevel::Low,
        estimated_effort_hours: 0.5,
        github_repo: None,
        migration_guide_url: None,
        compatibility_notes: vec![],
        success_rate: None,
    };

    assert!(analysis.is_safe());
    assert_eq!(analysis.total_breaking_changes(), 0);
    assert_eq!(analysis.total_packages_affected(), 1);
}
