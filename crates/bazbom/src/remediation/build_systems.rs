// Build system-specific fix application

use anyhow::Result;
use std::fs;
use std::path::Path;

use super::types::RemediationSuggestion;

/// Apply Maven fix to pom.xml
pub fn apply_maven_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let pom_path = project_root.join("pom.xml");
    if !pom_path.exists() {
        anyhow::bail!("pom.xml not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&pom_path)?;

    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    let mut updated = content.clone();
    let mut match_found = false;

    let lines: Vec<&str> = content.lines().collect();
    for i in 0..lines.len() {
        let line = lines[i];

        if line.contains("<artifactId>")
            && line.contains(artifact)
            && line.contains("</artifactId>")
        {
            for version_line in lines.iter().skip(i + 1).take(4) {
                if version_line.contains("<version>")
                    && version_line.contains(&suggestion.current_version)
                {
                    let new_line = version_line.replace(&suggestion.current_version, fixed_version);
                    updated = updated.replace(version_line, &new_line);
                    match_found = true;
                    println!(
                        "  [+] Updated {}: {} → {}",
                        artifact, suggestion.current_version, fixed_version
                    );
                    break;
                }
            }
            if match_found {
                break;
            }
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in pom.xml",
            artifact,
            suggestion.current_version
        );
    }

    fs::write(&pom_path, updated)?;
    Ok(())
}

/// Apply Gradle fix to build.gradle or build.gradle.kts
pub fn apply_gradle_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let build_gradle = project_root.join("build.gradle");
    let build_gradle_kts = project_root.join("build.gradle.kts");

    let gradle_file = if build_gradle.exists() {
        build_gradle
    } else if build_gradle_kts.exists() {
        build_gradle_kts
    } else {
        anyhow::bail!("No build.gradle or build.gradle.kts found in project root");
    };

    let content = fs::read_to_string(&gradle_file)?;

    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        if line.contains(artifact) && line.contains(&suggestion.current_version) {
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                artifact, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in {}",
            artifact,
            suggestion.current_version,
            gradle_file.display()
        );
    }

    fs::write(&gradle_file, updated)?;
    Ok(())
}

/// Apply Bazel fix to MODULE.bazel or WORKSPACE
pub fn apply_bazel_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let module_bazel = project_root.join("MODULE.bazel");
    let workspace = project_root.join("WORKSPACE");

    let bazel_file = if module_bazel.exists() {
        module_bazel
    } else if workspace.exists() {
        workspace
    } else {
        anyhow::bail!("No MODULE.bazel or WORKSPACE found in project root");
    };

    let content = fs::read_to_string(&bazel_file)?;

    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        if line.contains(artifact) && line.contains(&suggestion.current_version) {
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                artifact, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in {}",
            artifact,
            suggestion.current_version,
            bazel_file.display()
        );
    }

    fs::write(&bazel_file, updated)?;

    println!("  [!] Remember to run: bazel run @maven//:pin");

    Ok(())
}
