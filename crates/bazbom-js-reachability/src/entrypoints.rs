//! Entrypoint detection for JavaScript/TypeScript projects

use crate::error::Result;
use crate::models::{Entrypoint, EntrypointType};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Detect entrypoints in a JavaScript/TypeScript project
pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Detect all entrypoints in the project
    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        let mut entrypoints = Vec::new();

        // 1. Check package.json for main and exports
        if let Ok(main_entrypoints) = self.detect_from_package_json() {
            entrypoints.extend(main_entrypoints);
        }

        // 2. Detect HTTP handlers (Express, Fastify, etc.)
        if let Ok(http_entrypoints) = self.detect_http_handlers() {
            entrypoints.extend(http_entrypoints);
        }

        // 3. Detect test files
        if let Ok(test_entrypoints) = self.detect_test_files() {
            entrypoints.extend(test_entrypoints);
        }

        info!("Detected {} entrypoints", entrypoints.len());
        Ok(entrypoints)
    }

    /// Detect entrypoints from package.json
    fn detect_from_package_json(&self) -> Result<Vec<Entrypoint>> {
        let package_json_path = self.project_root.join("package.json");

        if !package_json_path.exists() {
            debug!("No package.json found");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&package_json_path)?;
        let package_json: serde_json::Value = serde_json::from_str(&content)?;

        let mut entrypoints = Vec::new();

        // Check "main" field
        if let Some(main) = package_json["main"].as_str() {
            let main_path = self.project_root.join(main);
            if main_path.exists() {
                entrypoints.push(Entrypoint {
                    file: main_path,
                    function_name: "main".to_string(),
                    entrypoint_type: EntrypointType::Main,
                });
            }
        }

        // Check "exports" field
        if let Some(exports) = package_json["exports"].as_object() {
            for (_, value) in exports {
                if let Some(export_path) = value.as_str() {
                    let full_path = self.project_root.join(export_path);
                    if full_path.exists() {
                        entrypoints.push(Entrypoint {
                            file: full_path,
                            function_name: "export".to_string(),
                            entrypoint_type: EntrypointType::Export,
                        });
                    }
                }
            }
        }

        Ok(entrypoints)
    }

    /// Detect HTTP handler entrypoints (Express, Fastify, etc.)
    fn detect_http_handlers(&self) -> Result<Vec<Entrypoint>> {
        let mut entrypoints = Vec::new();

        // Look for common patterns:
        // - app.get(), app.post(), app.use(), etc. (Express)
        // - fastify.get(), fastify.post(), etc. (Fastify)
        // - Files in routes/ or api/ directories

        let routes_dir = self.project_root.join("routes");
        if routes_dir.exists() && routes_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&routes_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if self.is_js_or_ts_file(path) {
                        entrypoints.push(Entrypoint {
                            file: path.to_path_buf(),
                            function_name: "handler".to_string(),
                            entrypoint_type: EntrypointType::HttpHandler,
                        });
                    }
                }
            }
        }

        let api_dir = self.project_root.join("api");
        if api_dir.exists() && api_dir.is_dir() {
            for entry in walkdir::WalkDir::new(&api_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if self.is_js_or_ts_file(path) {
                        entrypoints.push(Entrypoint {
                            file: path.to_path_buf(),
                            function_name: "handler".to_string(),
                            entrypoint_type: EntrypointType::HttpHandler,
                        });
                    }
                }
            }
        }

        Ok(entrypoints)
    }

    /// Detect test file entrypoints
    fn detect_test_files(&self) -> Result<Vec<Entrypoint>> {
        let mut entrypoints = Vec::new();

        // Look for common test patterns:
        // - *.test.js, *.spec.js
        // - __tests__/ directory
        // - test/ directory

        let patterns = vec![
            self.project_root.join("test"),
            self.project_root.join("__tests__"),
            self.project_root.join("tests"),
        ];

        for test_dir in patterns {
            if test_dir.exists() && test_dir.is_dir() {
                for entry in walkdir::WalkDir::new(&test_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if self.is_test_file(path) {
                            entrypoints.push(Entrypoint {
                                file: path.to_path_buf(),
                                function_name: "test".to_string(),
                                entrypoint_type: EntrypointType::Test,
                            });
                        }
                    }
                }
            }
        }

        Ok(entrypoints)
    }

    fn is_js_or_ts_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
        } else {
            false
        }
    }

    fn is_test_file(&self, path: &Path) -> bool {
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            (filename.contains(".test.") || filename.contains(".spec."))
                && self.is_js_or_ts_file(path)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_from_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");

        let content = r#"{
            "name": "test-project",
            "main": "index.js"
        }"#;

        fs::write(&package_json, content).unwrap();
        fs::write(temp_dir.path().join("index.js"), "console.log('hello');").unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_from_package_json().unwrap();

        assert_eq!(entrypoints.len(), 1);
        assert_eq!(entrypoints[0].entrypoint_type, EntrypointType::Main);
    }

    #[test]
    fn test_detect_test_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test");
        fs::create_dir(&test_dir).unwrap();

        fs::write(
            test_dir.join("example.test.js"),
            "test('example', () => {});",
        )
        .unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_test_files().unwrap();

        assert_eq!(entrypoints.len(), 1);
        assert_eq!(entrypoints[0].entrypoint_type, EntrypointType::Test);
    }
}
