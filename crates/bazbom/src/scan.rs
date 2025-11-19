//! Scan logic extracted from main.rs to improve modularity

use anyhow::{Context, Result};
use bazbom_core::{detect_build_system, write_stub_sbom};
use std::path::PathBuf;

/// Handle legacy scan command
#[allow(dead_code)] // Used from commands module
#[allow(clippy::too_many_arguments)]
pub fn handle_legacy_scan(
    path: String,
    reachability: bool,
    _fast: bool,
    format: String,
    out_dir: String,
    _bazel_targets_query: Option<String>,
    _bazel_targets: Option<Vec<String>>,
    _bazel_affected_by_files: Option<Vec<String>>,
    _bazel_universe: String,
    _bazel_exclude_targets: Option<Vec<String>>,
    _bazel_workspace_path: Option<String>,
    _include_path: Option<Vec<String>>,
    _languages: Option<Vec<String>>,
    _bazel_rc_path: Option<String>,
    _bazel_flags: Option<String>,
    _bazel_show_internal_targets: bool,
    _bazel_vendor_manifest_path: Option<String>,
    _incremental: bool,
    _base: String,
    _benchmark: bool,
    _ml_risk: bool,
    include_cicd: bool,
    fetch_checksums: bool,
) -> Result<()> {
    let root = PathBuf::from(&path);
    let system = detect_build_system(&root);
    let out = PathBuf::from(&out_dir);

    println!(
        "[bazbom] scan path={} reachability={} format={} system={:?}",
        path, reachability, format, system
    );

    // Scan for polyglot ecosystems (npm, Python, Go, Rust, etc.)
    let root_path = std::path::Path::new(&path);
    let mut polyglot_results = Vec::new();

    if !_fast {
        tracing::info!("Scanning for polyglot ecosystems in {}", path);
        println!("[bazbom] scanning for polyglot ecosystems...");

        // Use lightweight SBOM-only scanning (no vulnerabilities, no reachability)
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                polyglot_results = tokio::task::block_in_place(|| {
                    handle.block_on(bazbom_scanner::scan_directory_sbom_only(&path, include_cicd))
                })?;
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()?;
                polyglot_results = rt.block_on(bazbom_scanner::scan_directory_sbom_only(&path, include_cicd))?;
            }
        }

        // Apply language filtering if requested
        if let Some(langs) = _languages {
            let before_count = polyglot_results.len();
            polyglot_results.retain(|result| {
                // Map language names to ecosystem names
                let ecosystem_lower = result.ecosystem.to_lowercase();
                langs.iter().any(|lang| {
                    let lang_lower = lang.to_lowercase();
                    match lang_lower.as_str() {
                        "java" | "jvm" => ecosystem_lower.contains("maven") || ecosystem_lower.contains("gradle"),
                        "javascript" | "js" | "typescript" | "ts" | "node" => ecosystem_lower.contains("npm") || ecosystem_lower.contains("yarn") || ecosystem_lower.contains("pnpm"),
                        "python" | "py" | "pip" => ecosystem_lower.contains("python") || ecosystem_lower.contains("pip"),
                        "go" | "golang" => ecosystem_lower.contains("go"),
                        "rust" | "cargo" => ecosystem_lower.contains("rust") || ecosystem_lower.contains("cargo"),
                        "ruby" | "rb" | "gem" => ecosystem_lower.contains("ruby") || ecosystem_lower.contains("bundler"),
                        "php" | "composer" => ecosystem_lower.contains("php") || ecosystem_lower.contains("composer"),
                        _ => ecosystem_lower.contains(&lang_lower),
                    }
                })
            });
            let filtered_count = before_count - polyglot_results.len();
            if filtered_count > 0 {
                tracing::info!("Language filtering: excluded {} ecosystems (keeping only: {})",
                    filtered_count, langs.join(", "));
                println!("[bazbom] language filter: excluded {} ecosystems, keeping only: {}",
                    filtered_count, langs.join(", "));
            }
        }

        if !polyglot_results.is_empty() {
            let total_packages: usize = polyglot_results.iter().map(|r| r.packages.len()).sum();
            tracing::info!("Found {} packages across {} polyglot ecosystems",
                total_packages, polyglot_results.len());
            println!("[bazbom] found {} packages across {} polyglot ecosystems",
                total_packages, polyglot_results.len());
        }
    } else {
        tracing::debug!("Skipping polyglot scanning (fast mode enabled)");
    }

    // Handle Bazel projects - extract Maven dependencies and merge with polyglot
    tracing::debug!("Build system check: {:?}, is_bazel={}", system, system == bazbom_core::BuildSystem::Bazel);
    if system == bazbom_core::BuildSystem::Bazel {
        tracing::debug!("Detected Bazel project, checking for maven_install.json");
        let maven_install_json = root_path.join("maven_install.json");

        if maven_install_json.exists() {
            tracing::info!("Found maven_install.json at {:?}", maven_install_json);
            std::fs::create_dir_all(&out)?;
            let deps_json_path = out.join("bazel-deps.json");

            // Check if any Bazel-specific flags are set
            let has_bazel_flags = _bazel_targets.is_some()
                || _bazel_targets_query.is_some()
                || _bazel_affected_by_files.is_some()
                || _bazel_exclude_targets.is_some()
                || _bazel_show_internal_targets;

            tracing::debug!("Bazel flags check: targets={:?}, query={:?}, affected={:?}, exclude={:?}, show_internal={}, has_flags={}",
                _bazel_targets.is_some(), _bazel_targets_query.is_some(), _bazel_affected_by_files.is_some(),
                _bazel_exclude_targets.is_some(), _bazel_show_internal_targets, has_bazel_flags);

            let extraction_result = if has_bazel_flags {
                // Use targeted extraction with Bazel query
                tracing::info!("Using targeted Bazel extraction (flags specified)");

                // Determine workspace path
                let workspace = if let Some(ref ws_path) = _bazel_workspace_path {
                    std::path::PathBuf::from(ws_path)
                } else {
                    root_path.to_path_buf()
                };

                // Execute Bazel query to get targets
                let targets = if let Some(ref targets_list) = _bazel_targets {
                    // Explicit targets provided
                    tracing::info!("Using explicit targets: {:?}", targets_list);
                    targets_list.clone()
                } else if let Some(ref query) = _bazel_targets_query {
                    // Query expression provided
                    tracing::info!("Executing Bazel query: {}", query);
                    crate::bazel::query_bazel_targets(
                        &workspace,
                        Some(query),
                        None,
                        None,
                        &_bazel_universe,
                        _bazel_exclude_targets.as_deref(),
                        _bazel_rc_path.as_deref(),
                        _bazel_flags.as_deref(),
                    )?
                } else if let Some(ref files) = _bazel_affected_by_files {
                    // Affected files provided (rdeps query)
                    tracing::info!("Finding targets affected by {} files", files.len());
                    crate::bazel::query_bazel_targets(
                        &workspace,
                        None,
                        None,
                        Some(files),
                        &_bazel_universe,
                        _bazel_exclude_targets.as_deref(),
                        _bazel_rc_path.as_deref(),
                        _bazel_flags.as_deref(),
                    )?
                } else if _bazel_show_internal_targets {
                    // Show internal targets (libraries)
                    tracing::info!("Querying all library targets");
                    crate::bazel::query_bazel_targets(
                        &workspace,
                        Some("kind(\".*_library\", //...)"),
                        None,
                        None,
                        &_bazel_universe,
                        _bazel_exclude_targets.as_deref(),
                        _bazel_rc_path.as_deref(),
                        _bazel_flags.as_deref(),
                    )?
                } else {
                    // Only exclude flags set, query all targets
                    tracing::info!("Querying all targets with exclusions");
                    crate::bazel::query_bazel_targets(
                        &workspace,
                        Some("//..."),
                        None,
                        None,
                        &_bazel_universe,
                        _bazel_exclude_targets.as_deref(),
                        _bazel_rc_path.as_deref(),
                        _bazel_flags.as_deref(),
                    )?
                };

                if targets.is_empty() {
                    tracing::warn!("No targets matched query/filters - falling back to full extraction");
                    crate::bazel::extract_bazel_dependencies(root_path, &deps_json_path)
                } else {
                    // Auto-build targets if needed (enabled by default for targeted scans)
                    let built_targets = crate::bazel::build_bazel_targets(
                        &workspace,
                        &targets,
                        true,
                        _bazel_rc_path.as_deref(),
                        _bazel_flags.as_deref(),
                    )?;

                    if built_targets.is_empty() {
                        tracing::warn!("No targets successfully built - falling back to full extraction");
                        crate::bazel::extract_bazel_dependencies(root_path, &deps_json_path)
                    } else {
                        crate::bazel::extract_bazel_dependencies_for_targets(&workspace, &built_targets, &deps_json_path)
                    }
                }
            } else {
                // No flags set, use full extraction
                tracing::info!("Using full Bazel extraction (no targeting flags)");
                crate::bazel::extract_bazel_dependencies(root_path, &deps_json_path)
            };

            match extraction_result {
                Ok(graph) => {
                    tracing::info!(
                        "Successfully extracted {} Maven packages from maven_install.json",
                        graph.components.len()
                    );
                    println!("[bazbom] found {} Maven packages from maven_install.json", graph.components.len());

                    // Convert Bazel Maven components to polyglot Package format
                    let maven_packages: Vec<bazbom_scanner::Package> = graph.components.iter().map(|component| {
                        bazbom_scanner::Package {
                            name: component.name.clone(),
                            version: component.version.clone(),
                            ecosystem: "Maven".to_string(),
                            namespace: Some(component.group.clone()),
                            dependencies: vec![],
                            license: None,
                            description: None,
                            homepage: None,
                            repository: if component.repository.is_empty() {
                                None
                            } else {
                                Some(component.repository.clone())
                            },
                        }
                    }).collect();

                    // Merge Maven packages into polyglot results
                    if !maven_packages.is_empty() {
                        let maven_result = bazbom_scanner::EcosystemScanResult {
                            ecosystem: "Maven (Bazel)".to_string(),
                            root_path: path.clone(),
                            packages: maven_packages,
                            vulnerabilities: vec![],
                            total_packages: graph.components.len(),
                            total_vulnerabilities: 0,
                            reachability: None,
                        };
                        polyglot_results.push(maven_result);
                        tracing::info!("Merged {} Maven packages into unified results", graph.components.len());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to extract Bazel dependencies: {}", e);
                    eprintln!("[bazbom] warning: failed to extract Bazel dependencies: {}", e);
                }
            }
        } else {
            tracing::info!("No maven_install.json found in Bazel workspace");
            println!("[bazbom] no maven_install.json found");
            println!("[bazbom] hint: run 'bazel run @maven//:pin' to generate maven_install.json");
        }

        // Detect CI/CD tooling for Bazel projects (if requested)
        if include_cicd {
            tracing::info!("Detecting CI/CD tooling in Bazel workspace");
            println!("[bazbom] detecting CI/CD dependencies...");
            match bazbom_scanner::cicd::detect_github_actions(root_path) {
                Ok(cicd_result) => {
                    if !cicd_result.packages.is_empty() {
                        tracing::info!("Found {} CI/CD packages", cicd_result.packages.len());
                        println!("[bazbom] found {} CI/CD packages", cicd_result.packages.len());
                        polyglot_results.push(cicd_result);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to scan GitHub Actions: {}", e);
                    eprintln!("[bazbom] warning: failed to scan GitHub Actions: {}", e);
                }
            }
        }
    }

    // Generate unified SBOM from all detected ecosystems
    if !polyglot_results.is_empty() {
        let total_packages: usize = polyglot_results.iter().map(|r| r.packages.len()).sum();
        tracing::info!("Generating unified SBOM with {} packages across {} ecosystems",
            total_packages, polyglot_results.len());
        println!("[bazbom] generating unified SBOM ({} packages from {} ecosystems)",
            total_packages, polyglot_results.len());

        std::fs::create_dir_all(&out)?;

        // Generate format-specific SBOM
        match format.as_str() {
            "cyclonedx-xml" => {
                // Generate CycloneDX 1.5 XML
                let mut cdx_doc = bazbom_formats::cyclonedx::CycloneDxBom::new(
                    "bazbom",
                    env!("CARGO_PKG_VERSION")
                );

                // Convert all packages from polyglot results to CycloneDX components
                for ecosystem_result in &polyglot_results {
                    for package in &ecosystem_result.packages {
                        let mut component = bazbom_formats::cyclonedx::Component::new(
                            &package.name,
                            "library"
                        )
                        .with_version(&package.version)
                        .with_purl(&package.purl());

                        if let Some(ref license) = package.license {
                            component = component.with_license(license);
                        }

                        // Add download URL if available
                        if let Some(download_url) = package.download_url() {
                            component = component.with_download_url(download_url);
                        }

                        cdx_doc.add_component(component);
                    }
                }

                let xml_content = cdx_doc.to_xml();
                let cdx_path = out.join("sbom.cyclonedx.xml");
                std::fs::write(&cdx_path, xml_content)?;
                tracing::info!("Wrote CycloneDX 1.5 XML SBOM to {:?}", cdx_path);
                println!("[bazbom] wrote CycloneDX 1.5 XML SBOM to {:?}", cdx_path);
            }
            "cyclonedx" => {
                // Generate CycloneDX 1.5 JSON
                let mut cdx_doc = bazbom_formats::cyclonedx::CycloneDxBom::new(
                    "bazbom",
                    env!("CARGO_PKG_VERSION")
                );

                // Convert all packages from polyglot results to CycloneDX components
                for ecosystem_result in &polyglot_results {
                    for package in &ecosystem_result.packages {
                        let mut component = bazbom_formats::cyclonedx::Component::new(
                            &package.name,
                            "library"
                        )
                        .with_version(&package.version)
                        .with_purl(&package.purl());

                        if let Some(ref license) = package.license {
                            component = component.with_license(license);
                        }

                        // Add download URL if available
                        if let Some(download_url) = package.download_url() {
                            component = component.with_download_url(download_url);
                        }

                        cdx_doc.add_component(component);
                    }
                }

                let cdx_path = out.join("sbom.cyclonedx.json");
                std::fs::write(
                    &cdx_path,
                    serde_json::to_string_pretty(&cdx_doc)?,
                )?;
                tracing::info!("Wrote CycloneDX 1.5 SBOM to {:?}", cdx_path);
                println!("[bazbom] wrote CycloneDX 1.5 SBOM to {:?}", cdx_path);
            }
            "github-snapshot" => {
                // Generate GitHub dependency snapshot format
                // Try to get git SHA and ref
                let sha = std::process::Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .output()
                    .ok()
                    .and_then(|output| {
                        if output.status.success() {
                            String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "0000000000000000000000000000000000000000".to_string());

                let ref_name = std::process::Command::new("git")
                    .args(["symbolic-ref", "HEAD"])
                    .output()
                    .ok()
                    .and_then(|output| {
                        if output.status.success() {
                            String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "refs/heads/main".to_string());

                let snapshot = bazbom_scanner::generate_github_snapshot(&polyglot_results, &sha, &ref_name)?;
                let snapshot_path = out.join("github-snapshot.json");
                std::fs::write(
                    &snapshot_path,
                    serde_json::to_string_pretty(&snapshot)?,
                )?;
                tracing::info!("Wrote GitHub dependency snapshot to {:?}", snapshot_path);
                println!("[bazbom] wrote GitHub dependency snapshot to {:?}", snapshot_path);
            }
            "spdx-tagvalue" => {
                // Generate SPDX 2.3 tag-value format
                if fetch_checksums {
                    println!("[bazbom] fetching SHA256 checksums from package registries (this may take a moment)...");
                    tracing::info!("Fetching checksums for {} packages", total_packages);
                }

                // Generate JSON first
                let unified_sbom = match tokio::runtime::Handle::try_current() {
                    Ok(handle) => {
                        tokio::task::block_in_place(|| {
                            handle.block_on(bazbom_scanner::generate_polyglot_sbom(&polyglot_results, fetch_checksums))
                        })?
                    }
                    Err(_) => {
                        let rt = tokio::runtime::Runtime::new()?;
                        rt.block_on(bazbom_scanner::generate_polyglot_sbom(&polyglot_results, fetch_checksums))?
                    }
                };

                // Convert to tag-value format
                let tag_value_content = bazbom_scanner::spdx_json_to_tag_value(&unified_sbom)?;
                let spdx_path = out.join("sbom.spdx");
                std::fs::write(&spdx_path, tag_value_content)?;
                tracing::info!("Wrote SPDX 2.3 tag-value SBOM to {:?}", spdx_path);
                println!("[bazbom] wrote SPDX 2.3 tag-value SBOM to {:?}", spdx_path);
            }
            "spdx" | _ => {
                // Generate SPDX 2.3 JSON (default)
                if fetch_checksums {
                    println!("[bazbom] fetching SHA256 checksums from package registries (this may take a moment)...");
                    tracing::info!("Fetching checksums for {} packages", total_packages);
                }

                let unified_sbom = match tokio::runtime::Handle::try_current() {
                    Ok(handle) => {
                        tokio::task::block_in_place(|| {
                            handle.block_on(bazbom_scanner::generate_polyglot_sbom(&polyglot_results, fetch_checksums))
                        })?
                    }
                    Err(_) => {
                        // Create new runtime if not already in one
                        let rt = tokio::runtime::Runtime::new()?;
                        rt.block_on(bazbom_scanner::generate_polyglot_sbom(&polyglot_results, fetch_checksums))?
                    }
                };

                let spdx_path = out.join("sbom.spdx.json");
                std::fs::write(
                    &spdx_path,
                    serde_json::to_string_pretty(&unified_sbom)?,
                )?;
                tracing::info!("Wrote SPDX 2.3 SBOM to {:?}", spdx_path);
                println!("[bazbom] wrote SPDX 2.3 SBOM to {:?}", spdx_path);
            }
        }
    } else {
        // No packages detected - write stub SBOM
        tracing::warn!("No packages detected in any ecosystem");
        println!("[bazbom] no packages detected, writing stub SBOM");
        write_stub_sbom(&out, &format, system)
            .with_context(|| format!("failed writing stub SBOM to {:?}", out))?;
    }

    // Create stub findings files in both SARIF and JSON formats for compatibility
    let stub_findings = serde_json::json!({
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": []
    });
    let findings_json = serde_json::to_string_pretty(&stub_findings)?;

    // Write SARIF format (for SARIF consumers)
    std::fs::write(out.join("sca_findings.sarif"), &findings_json)?;

    // Also write as .json (for compatibility with other commands like fix, upgrade-intelligence)
    std::fs::write(out.join("sca_findings.json"), &findings_json)?;

    // JSON output mode
    if std::env::var("BAZBOM_JSON_MODE").is_ok() {
        let json_output = serde_json::json!({
            "scan_time": chrono::Utc::now().to_rfc3339(),
            "path": path,
            "build_system": format!("{:?}", system),
            "reachability_enabled": reachability,
            "format": format,
            "output_dir": out_dir,
            "status": "success",
            "sbom_generated": true,
            "sarif_generated": true
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    }

    Ok(())
}
