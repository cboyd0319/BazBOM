//! Module resolution for Node.js-style imports

use super::error::{JsReachabilityError, Result};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Resolve module imports following Node.js resolution algorithm
pub struct ModuleResolver {
    project_root: PathBuf,
}

impl ModuleResolver {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Resolve a module specifier to an absolute file path
    ///
    /// Handles:
    /// - Relative imports: ./foo, ../bar
    /// - Absolute imports: /foo/bar
    /// - node_modules: express, @types/node
    /// - Extension resolution: .js, .ts, .jsx, .tsx
    /// - index files: index.js, index.ts
    pub fn resolve(&self, specifier: &str, from_file: &Path) -> Result<PathBuf> {
        debug!("Resolving '{}' from {:?}", specifier, from_file);

        // 1. Relative imports
        if specifier.starts_with("./") || specifier.starts_with("../") {
            return self.resolve_relative(specifier, from_file);
        }

        // 2. Absolute imports
        if specifier.starts_with('/') {
            return self.resolve_absolute(specifier);
        }

        // 3. node_modules
        self.resolve_node_modules(specifier, from_file)
    }

    /// Resolve relative import (./foo, ../bar)
    fn resolve_relative(&self, specifier: &str, from_file: &Path) -> Result<PathBuf> {
        let from_dir =
            from_file
                .parent()
                .ok_or_else(|| JsReachabilityError::ModuleResolutionError {
                    module: specifier.to_string(),
                    reason: "Invalid from_file path".to_string(),
                })?;

        let resolved = from_dir.join(specifier);
        self.resolve_with_extensions(&resolved, specifier)
    }

    /// Resolve absolute import (/foo/bar)
    fn resolve_absolute(&self, specifier: &str) -> Result<PathBuf> {
        let path = PathBuf::from(specifier);
        self.resolve_with_extensions(&path, specifier)
    }

    /// Resolve from node_modules
    fn resolve_node_modules(&self, specifier: &str, from_file: &Path) -> Result<PathBuf> {
        // Start from the directory of the importing file
        let mut current_dir = from_file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.project_root.clone());

        loop {
            let node_modules = current_dir.join("node_modules");

            if node_modules.exists() {
                let package_dir = node_modules.join(specifier);

                // Check if package exists
                if package_dir.exists() {
                    // Try to resolve via package.json
                    let package_json = package_dir.join("package.json");

                    if package_json.exists() {
                        if let Ok(entry) = self.resolve_package_json(&package_json) {
                            return Ok(entry);
                        }
                    }

                    // Fall back to index.js
                    let index_js = package_dir.join("index.js");
                    if index_js.exists() {
                        return Ok(index_js);
                    }

                    let index_ts = package_dir.join("index.ts");
                    if index_ts.exists() {
                        return Ok(index_ts);
                    }
                }
            }

            // Move up to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(JsReachabilityError::ModuleResolutionError {
            module: specifier.to_string(),
            reason: "Not found in node_modules".to_string(),
        })
    }

    /// Resolve entry point from package.json
    fn resolve_package_json(&self, package_json_path: &Path) -> Result<PathBuf> {
        let content = std::fs::read_to_string(package_json_path)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let package_dir = package_json_path.parent().ok_or_else(|| {
            JsReachabilityError::InvalidPath("Invalid package.json path".to_string())
        })?;

        // Check "main" field
        if let Some(main) = package_json["main"].as_str() {
            let main_path = package_dir.join(main);
            if main_path.exists() {
                return Ok(main_path);
            }
        }

        // Check "exports" field (modern)
        if let Some(exports) = package_json["exports"].as_str() {
            let exports_path = package_dir.join(exports);
            if exports_path.exists() {
                return Ok(exports_path);
            }
        }

        Err(JsReachabilityError::ModuleResolutionError {
            module: package_json_path.display().to_string(),
            reason: "No valid entry point in package.json".to_string(),
        })
    }

    /// Try to resolve with common extensions
    fn resolve_with_extensions(&self, base_path: &Path, specifier: &str) -> Result<PathBuf> {
        // Try exact path first
        if base_path.exists() && base_path.is_file() {
            return Ok(base_path.to_path_buf());
        }

        // Try with extensions
        let extensions = vec!["js", "ts", "jsx", "tsx", "mjs", "cjs"];

        for ext in &extensions {
            let with_ext = base_path.with_extension(ext);
            if with_ext.exists() && with_ext.is_file() {
                return Ok(with_ext);
            }
        }

        // Try index files
        if base_path.is_dir() {
            for ext in &extensions {
                let index_file = base_path.join(format!("index.{}", ext));
                if index_file.exists() {
                    return Ok(index_file);
                }
            }
        }

        Err(JsReachabilityError::ModuleResolutionError {
            module: specifier.to_string(),
            reason: format!("File not found: {}", base_path.display()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_relative() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(src_dir.join("index.js"), "").unwrap();
        fs::write(src_dir.join("helper.js"), "").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let from_file = src_dir.join("index.js");

        let resolved = resolver.resolve("./helper", &from_file).unwrap();
        assert!(resolved.ends_with("helper.js"));
    }

    #[test]
    fn test_resolve_with_extension() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.ts"), "").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let from_file = temp_dir.path().join("index.js");

        let resolved = resolver.resolve("./test", &from_file).unwrap();
        assert!(resolved.ends_with("test.ts"));
    }

    #[test]
    fn test_resolve_index() {
        let temp_dir = TempDir::new().unwrap();
        let utils_dir = temp_dir.path().join("utils");
        fs::create_dir(&utils_dir).unwrap();
        fs::write(utils_dir.join("index.js"), "").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let from_file = temp_dir.path().join("app.js");

        let resolved = resolver.resolve("./utils", &from_file).unwrap();
        assert!(resolved.ends_with("index.js"));
    }
}
