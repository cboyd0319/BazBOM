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

/// Apply npm/yarn fix to package.json
pub fn apply_npm_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let package_json = project_root.join("package.json");
    if !package_json.exists() {
        anyhow::bail!("package.json not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&package_json)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    let package_name = &suggestion.affected_package;
    let mut updated = false;

    // Try dependencies
    if let Some(deps) = json.get_mut("dependencies").and_then(|d| d.as_object_mut()) {
        if let Some(version) = deps.get_mut(package_name) {
            *version = serde_json::Value::String(format!("^{}", fixed_version));
            updated = true;
        }
    }

    // Try devDependencies
    if !updated {
        if let Some(deps) = json
            .get_mut("devDependencies")
            .and_then(|d| d.as_object_mut())
        {
            if let Some(version) = deps.get_mut(package_name) {
                *version = serde_json::Value::String(format!("^{}", fixed_version));
                updated = true;
            }
        }
    }

    if !updated {
        anyhow::bail!(
            "Dependency {} not found in package.json dependencies or devDependencies",
            package_name
        );
    }

    let updated_content = serde_json::to_string_pretty(&json)?;
    fs::write(&package_json, updated_content)?;

    println!(
        "  [+] Updated {}: {} → {}",
        package_name, suggestion.current_version, fixed_version
    );
    println!("  [!] Remember to run: npm install");

    Ok(())
}

/// Apply pip fix to requirements.txt
pub fn apply_pip_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let requirements_txt = project_root.join("requirements.txt");
    if !requirements_txt.exists() {
        anyhow::bail!("requirements.txt not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&requirements_txt)?;
    let package_name = &suggestion.affected_package;

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        // Match lines like "package==1.2.3" or "package>=1.2.3"
        if line.starts_with(package_name) && (line.contains("==") || line.contains(">=")) {
            let new_line = format!("{}=={}", package_name, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                package_name, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!("Dependency {} not found in requirements.txt", package_name);
    }

    fs::write(&requirements_txt, updated)?;
    println!("  [!] Remember to run: pip install -r requirements.txt");

    Ok(())
}

/// Apply Go fix to go.mod
pub fn apply_go_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let go_mod = project_root.join("go.mod");
    if !go_mod.exists() {
        anyhow::bail!("go.mod not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&go_mod)?;
    let package_name = &suggestion.affected_package;

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        // Match lines like "  github.com/foo/bar v1.2.3"
        if line.contains(package_name) && line.contains(&suggestion.current_version) {
            let new_line =
                line.replace(&suggestion.current_version, &format!("v{}", fixed_version));
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                package_name, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in go.mod",
            package_name,
            suggestion.current_version
        );
    }

    fs::write(&go_mod, updated)?;
    println!("  [!] Remember to run: go mod tidy");

    Ok(())
}

/// Apply Rust fix to Cargo.toml
pub fn apply_cargo_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let cargo_toml = project_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        anyhow::bail!("Cargo.toml not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&cargo_toml)?;
    let package_name = &suggestion.affected_package;

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        // Match lines like 'package_name = "1.2.3"' or 'package_name = { version = "1.2.3" }'
        if line.contains(package_name) && line.contains(&suggestion.current_version) {
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                package_name, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in Cargo.toml",
            package_name,
            suggestion.current_version
        );
    }

    fs::write(&cargo_toml, updated)?;
    println!("  [!] Remember to run: cargo update");

    Ok(())
}

/// Apply Ruby fix to Gemfile
pub fn apply_bundler_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let gemfile = project_root.join("Gemfile");
    if !gemfile.exists() {
        anyhow::bail!("Gemfile not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&gemfile)?;
    let package_name = &suggestion.affected_package;

    let mut updated = content.clone();
    let mut match_found = false;

    for line in content.lines() {
        // Match lines like "gem 'package_name', '~> 1.2.3'"
        if line.contains(package_name) && line.contains(&suggestion.current_version) {
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  [+] Updated {}: {} → {}",
                package_name, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in Gemfile",
            package_name,
            suggestion.current_version
        );
    }

    fs::write(&gemfile, updated)?;
    println!("  [!] Remember to run: bundle update {}", package_name);

    Ok(())
}

/// Apply PHP fix to composer.json
pub fn apply_composer_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let composer_json = project_root.join("composer.json");
    if !composer_json.exists() {
        anyhow::bail!("composer.json not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    let content = fs::read_to_string(&composer_json)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    let package_name = &suggestion.affected_package;
    let mut updated = false;

    // Try require
    if let Some(deps) = json.get_mut("require").and_then(|d| d.as_object_mut()) {
        if let Some(version) = deps.get_mut(package_name) {
            *version = serde_json::Value::String(format!("^{}", fixed_version));
            updated = true;
        }
    }

    // Try require-dev
    if !updated {
        if let Some(deps) = json.get_mut("require-dev").and_then(|d| d.as_object_mut()) {
            if let Some(version) = deps.get_mut(package_name) {
                *version = serde_json::Value::String(format!("^{}", fixed_version));
                updated = true;
            }
        }
    }

    if !updated {
        anyhow::bail!(
            "Dependency {} not found in composer.json require or require-dev",
            package_name
        );
    }

    let updated_content = serde_json::to_string_pretty(&json)?;
    fs::write(&composer_json, updated_content)?;

    println!(
        "  [+] Updated {}: {} → {}",
        package_name, suggestion.current_version, fixed_version
    );
    println!("  [!] Remember to run: composer update {}", package_name);

    Ok(())
}
