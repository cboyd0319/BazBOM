//! Rust entrypoint detection

use super::ast_parser::parse_file;
use super::error::Result;
use super::models::{Entrypoint, EntrypointType};
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::{Attribute, ItemFn};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Detects Rust entrypoints in a project
pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Detect all entrypoints in the project
    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        info!("Detecting Rust entrypoints");

        let mut entrypoints = Vec::new();

        // Walk through all Rust files
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !self.should_skip(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if self.is_rust_file(path) {
                    if let Ok(file_entrypoints) = self.detect_in_file(path) {
                        entrypoints.extend(file_entrypoints);
                    }
                }
            }
        }

        info!("Found {} Rust entrypoints", entrypoints.len());

        Ok(entrypoints)
    }

    /// Detect entrypoints in a single Rust file
    fn detect_in_file(&self, file_path: &Path) -> Result<Vec<Entrypoint>> {
        let ast = parse_file(file_path)?;
        let mut detector = EntrypointVisitor::new(file_path.to_path_buf());
        detector.visit_file(&ast);

        Ok(detector.entrypoints)
    }

    /// Check if directory should be skipped
    fn should_skip(&self, entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["target", ".git", "node_modules", "vendor"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    /// Check if file is a Rust file
    fn is_rust_file(&self, path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("rs")
    }
}

struct EntrypointVisitor {
    file_path: PathBuf,
    entrypoints: Vec<Entrypoint>,
}

impl EntrypointVisitor {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            entrypoints: Vec::new(),
        }
    }

    fn check_entrypoint_type(func_name: &str, attrs: &[Attribute]) -> Option<EntrypointType> {
        // Check for test functions
        if Self::has_attribute(attrs, "test") {
            debug!("Found #[test] function: {}", func_name);
            return Some(EntrypointType::Test);
        }

        // Check for benchmark functions
        if Self::has_attribute(attrs, "bench") {
            debug!("Found #[bench] function: {}", func_name);
            return Some(EntrypointType::Benchmark);
        }

        // Check for tokio::main
        if Self::has_attribute(attrs, "tokio::main") || Self::has_attribute(attrs, "tokio::test") {
            debug!(
                "Found #[tokio::main] or #[tokio::test] function: {}",
                func_name
            );
            return Some(EntrypointType::TokioMain);
        }

        // Check for actix_web::main
        if Self::has_attribute(attrs, "actix_web::main")
            || Self::has_attribute(attrs, "actix_rt::main")
        {
            debug!("Found #[actix_web::main] function: {}", func_name);
            return Some(EntrypointType::ActixMain);
        }

        // Check for standard main function
        if func_name == "main" {
            debug!("Found main function");
            return Some(EntrypointType::Main);
        }

        None
    }

    fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
        attrs.iter().any(|attr| {
            attr.path()
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
                == name
        })
    }
}

impl<'ast> Visit<'ast> for EntrypointVisitor {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        let func_name = func.sig.ident.to_string();

        if let Some(entrypoint_type) = Self::check_entrypoint_type(&func_name, &func.attrs) {
            self.entrypoints.push(Entrypoint {
                file: self.file_path.clone(),
                function_name: func_name,
                entrypoint_type,
            });
        }
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        // Visit inline modules
        if let Some((_, items)) = &module.content {
            for item in items {
                self.visit_item(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_main() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.rs");

        let code = r#"
fn helper() {
    println!("helper");
}

fn main() {
    helper();
}
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::Main));
    }

    #[test]
    fn test_detect_test_functions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("lib.rs");

        let code = r#"
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_subtraction() {
    assert_eq!(5 - 3, 2);
}
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(entrypoints.len() >= 2);
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "test_addition"));
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "test_subtraction"));
    }

    #[test]
    fn test_detect_tokio_main() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("async_main.rs");

        let code = r#"
#[tokio::main]
async fn main() {
    println!("Async main");
}
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::TokioMain));
    }

    #[test]
    fn test_skip_target_dir() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target");
        fs::create_dir(&target_dir).unwrap();

        let file_in_target = target_dir.join("debug.rs");
        fs::write(&file_in_target, "fn main() {}").unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        // Should not find entrypoints in target directory
        assert!(entrypoints.is_empty());
    }
}
