//! CI/CD configuration templates for easy integration
//!
//! Provides one-command CI setup for popular platforms

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// GitHub Actions workflow template
pub const GITHUB_ACTIONS_TEMPLATE: &str = r#"name: BazBOM Security Scan

on:
  push:
    branches: [ main, master, develop ]
  pull_request:
    branches: [ main, master, develop ]
  schedule:
    # Run daily at 2 AM UTC
    - cron: '0 2 * * *'

jobs:
  security-scan:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write  # For uploading SARIF
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Install BazBOM
        run: |
          cargo install bazbom  # Or download from releases
      
      - name: Run BazBOM CI scan
        run: bazbom ci -o ./scan-results
      
      - name: Upload SARIF to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: ./scan-results/sca_findings.sarif
      
      - name: Upload scan artifacts
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: bazbom-scan-results
          path: ./scan-results/
      
      - name: Fail on critical vulnerabilities
        run: |
          # Optional: fail build if critical vulns found
          # Parse JSON output and check severity
          if jq -e '.runs[].results[] | select(.level == "error")' ./scan-results/sca_findings.sarif > /dev/null; then
            echo "❌ Critical vulnerabilities found!"
            exit 1
          fi
"#;

/// GitLab CI template
pub const GITLAB_CI_TEMPLATE: &str = r#"# BazBOM Security Scan
# Add this to your .gitlab-ci.yml

bazbom_security_scan:
  stage: test
  image: rust:latest
  before_script:
    - cargo install bazbom  # Or use pre-built container
  script:
    - bazbom ci -o ./scan-results
  artifacts:
    when: always
    paths:
      - scan-results/
    reports:
      sast: scan-results/sca_findings.sarif
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
    - if: $CI_PIPELINE_SOURCE == "schedule"
  allow_failure: false

# Optional: Separate job for PR diff mode
bazbom_pr_diff:
  stage: test
  image: rust:latest
  before_script:
    - cargo install bazbom
  script:
    - bazbom pr --base $CI_MERGE_REQUEST_TARGET_BRANCH_NAME
  only:
    - merge_requests
  allow_failure: true
"#;

/// CircleCI template
pub const CIRCLECI_TEMPLATE: &str = r#"# BazBOM Security Scan
# Add this to your .circleci/config.yml

version: 2.1

jobs:
  security_scan:
    docker:
      - image: cimg/rust:1.75
    steps:
      - checkout
      - run:
          name: Install BazBOM
          command: cargo install bazbom
      - run:
          name: Run security scan
          command: bazbom ci -o ./scan-results
      - store_artifacts:
          path: ./scan-results
          destination: bazbom-scan
      - run:
          name: Check for critical vulnerabilities
          command: |
            if jq -e '.runs[].results[] | select(.level == "error")' ./scan-results/sca_findings.sarif > /dev/null; then
              echo "Critical vulnerabilities found!"
              exit 1
            fi

workflows:
  security_checks:
    jobs:
      - security_scan:
          filters:
            branches:
              only:
                - main
                - develop
"#;

/// Jenkins pipeline template
pub const JENKINS_TEMPLATE: &str = r#"// BazBOM Security Scan
// Add this to your Jenkinsfile

pipeline {
    agent any
    
    environment {
        BAZBOM_VERSION = 'latest'
    }
    
    stages {
        stage('Setup BazBOM') {
            steps {
                sh 'cargo install bazbom'
            }
        }
        
        stage('Security Scan') {
            steps {
                sh 'bazbom ci -o ./scan-results'
            }
        }
        
        stage('Archive Results') {
            steps {
                archiveArtifacts artifacts: 'scan-results/**/*', allowEmptyArchive: false
                
                // Publish SARIF if using Warnings Next Generation Plugin
                recordIssues(
                    tools: [sarif(pattern: 'scan-results/sca_findings.sarif')]
                )
            }
        }
        
        stage('Quality Gate') {
            steps {
                script {
                    def criticalCount = sh(
                        script: 'jq \'.runs[].results[] | select(.level == "error")\' ./scan-results/sca_findings.sarif | jq -s length',
                        returnStdout: true
                    ).trim().toInteger()
                    
                    if (criticalCount > 0) {
                        error("Found ${criticalCount} critical vulnerabilities!")
                    }
                }
            }
        }
    }
    
    post {
        always {
            junit 'scan-results/**/*.xml'  // If JUnit format available
        }
    }
}
"#;

/// Travis CI template
pub const TRAVIS_CI_TEMPLATE: &str = r#"# BazBOM Security Scan
# Add this to your .travis.yml

language: rust
rust:
  - stable

cache:
  cargo: true

before_script:
  - cargo install bazbom

script:
  - bazbom ci -o ./scan-results

after_success:
  - echo "✅ Security scan complete"

after_failure:
  - cat ./scan-results/scan_summary.json  # If available

deploy:
  provider: releases
  api_key: $GITHUB_TOKEN
  file: scan-results/sca_findings.sarif
  skip_cleanup: true
  on:
    tags: true
"#;

/// Install CI template for specified provider
pub fn install_ci_template(provider: &str) -> Result<()> {
    match provider {
        "github" | "github-actions" => {
            let dir = Path::new(".github/workflows");
            fs::create_dir_all(dir).context("Failed to create .github/workflows directory")?;

            let file_path = dir.join("bazbom.yml");
            fs::write(&file_path, GITHUB_ACTIONS_TEMPLATE)
                .context("Failed to write GitHub Actions workflow")?;

            println!("✅ Created GitHub Actions workflow: .github/workflows/bazbom.yml");
            println!("\n📋 Next steps:");
            println!("  1. Commit the workflow file: git add .github/workflows/bazbom.yml");
            println!("  2. Push to trigger the workflow");
            println!("  3. View results in GitHub Security tab");
        }

        "gitlab" => {
            let file_path = Path::new(".gitlab-ci.yml");

            let content = if file_path.exists() {
                let existing = fs::read_to_string(file_path)?;
                format!("{}\n\n{}", existing, GITLAB_CI_TEMPLATE)
            } else {
                GITLAB_CI_TEMPLATE.to_string()
            };

            fs::write(file_path, content).context("Failed to write GitLab CI configuration")?;

            println!("✅ Updated .gitlab-ci.yml with BazBOM job");
            println!("\n📋 Next steps:");
            println!("  1. Review changes: cat .gitlab-ci.yml");
            println!("  2. Commit and push");
            println!("  3. View results in GitLab Security Dashboard");
        }

        "circleci" => {
            let dir = Path::new(".circleci");
            fs::create_dir_all(dir).context("Failed to create .circleci directory")?;

            let file_path = dir.join("config.yml");
            let content = if file_path.exists() {
                let existing = fs::read_to_string(&file_path)?;
                format!("{}\n\n{}", existing, CIRCLECI_TEMPLATE)
            } else {
                CIRCLECI_TEMPLATE.to_string()
            };

            fs::write(&file_path, content).context("Failed to write CircleCI configuration")?;

            println!("✅ Updated .circleci/config.yml with BazBOM job");
        }

        "jenkins" => {
            let file_path = Path::new("Jenkinsfile.bazbom");
            fs::write(file_path, JENKINS_TEMPLATE).context("Failed to write Jenkinsfile")?;

            println!("✅ Created Jenkinsfile.bazbom");
            println!("\n📋 Next steps:");
            println!("  1. Review the file: cat Jenkinsfile.bazbom");
            println!("  2. Integrate into your main Jenkinsfile or use directly");
            println!("  3. Configure Jenkins pipeline to use this file");
        }

        "travis" => {
            let file_path = Path::new(".travis.yml");
            let content = if file_path.exists() {
                let existing = fs::read_to_string(file_path)?;
                format!("{}\n\n{}", existing, TRAVIS_CI_TEMPLATE)
            } else {
                TRAVIS_CI_TEMPLATE.to_string()
            };

            fs::write(file_path, content).context("Failed to write Travis CI configuration")?;

            println!("✅ Updated .travis.yml with BazBOM job");
        }

        _ => {
            anyhow::bail!("Unknown CI provider: {}\n\nSupported providers:\n  • github / github-actions\n  • gitlab\n  • circleci\n  • jenkins\n  • travis", provider);
        }
    }

    Ok(())
}

/// List available CI templates
pub fn list_templates() {
    println!("📦 Available CI templates:\n");
    println!("  • github       → GitHub Actions (.github/workflows/bazbom.yml)");
    println!("  • gitlab       → GitLab CI (.gitlab-ci.yml)");
    println!("  • circleci     → CircleCI (.circleci/config.yml)");
    println!("  • jenkins      → Jenkins (Jenkinsfile.bazbom)");
    println!("  • travis       → Travis CI (.travis.yml)");
    println!("\nUsage: bazbom install ci-<provider>");
    println!("Example: bazbom install ci-github");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_templates_not_empty() {
        assert!(!GITHUB_ACTIONS_TEMPLATE.is_empty());
        assert!(!GITLAB_CI_TEMPLATE.is_empty());
        assert!(!CIRCLECI_TEMPLATE.is_empty());
        assert!(!JENKINS_TEMPLATE.is_empty());
        assert!(!TRAVIS_CI_TEMPLATE.is_empty());
    }

    #[test]
    fn test_templates_contain_bazbom() {
        assert!(GITHUB_ACTIONS_TEMPLATE.contains("bazbom"));
        assert!(GITLAB_CI_TEMPLATE.contains("bazbom"));
        assert!(CIRCLECI_TEMPLATE.contains("bazbom"));
        assert!(JENKINS_TEMPLATE.contains("bazbom"));
        assert!(TRAVIS_CI_TEMPLATE.contains("bazbom"));
    }
}
