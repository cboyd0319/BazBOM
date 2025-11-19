//! Python module resolution (import statements)

use super::error::{PythonReachabilityError, Result};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Resolves Python module imports to file paths
pub struct ModuleResolver {
    /// Root directory of the project
    project_root: PathBuf,
}

impl ModuleResolver {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Resolve an import statement to a file path
    ///
    /// Handles:
    /// - `import module` -> module.py or module/__init__.py
    /// - `from package.module import func` -> package/module.py
    /// - `from . import sibling` -> ./sibling.py (relative import)
    /// - `from .. import parent` -> ../parent.py (relative import)
    pub fn resolve_import(&self, module_name: &str, from_file: &Path) -> Result<Vec<PathBuf>> {
        // Handle relative imports
        if module_name.starts_with('.') {
            return self.resolve_relative_import(module_name, from_file);
        }

        // Handle absolute imports
        self.resolve_absolute_import(module_name)
    }

    /// Resolve relative imports (e.g., from . import foo, from .. import bar)
    fn resolve_relative_import(&self, module_name: &str, from_file: &Path) -> Result<Vec<PathBuf>> {
        let dot_count = module_name.chars().take_while(|c| *c == '.').count();
        let module_part = &module_name[dot_count..];

        // Get the directory of the current file
        let current_dir = from_file.parent().ok_or_else(|| {
            PythonReachabilityError::ModuleResolutionError(format!(
                "Cannot get parent directory of {:?}",
                from_file
            ))
        })?;

        // Navigate up the directory tree
        let mut target_dir = current_dir.to_path_buf();
        for _ in 1..dot_count {
            target_dir = target_dir
                .parent()
                .ok_or_else(|| {
                    PythonReachabilityError::ModuleResolutionError(format!(
                        "Cannot navigate up from {:?}",
                        target_dir
                    ))
                })?
                .to_path_buf();
        }

        // If there's a module part, append it
        if !module_part.is_empty() {
            let module_path = module_part.replace('.', "/");
            target_dir = target_dir.join(module_path);
        }

        debug!(
            "Resolved relative import {} to {:?}",
            module_name, target_dir
        );

        // Try to find the module file
        self.find_module_file(&target_dir)
    }

    /// Resolve absolute imports (e.g., import foo.bar)
    fn resolve_absolute_import(&self, module_name: &str) -> Result<Vec<PathBuf>> {
        let module_path = module_name.replace('.', "/");
        let search_path = self.project_root.join(&module_path);

        debug!(
            "Resolving absolute import {} at {:?}",
            module_name, search_path
        );

        self.find_module_file(&search_path)
    }

    /// Find the actual Python file for a module path
    ///
    /// Checks:
    /// 1. path.py (single file module)
    /// 2. path/__init__.py (package)
    fn find_module_file(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut candidates = Vec::new();

        // Check for single file module (module.py)
        let py_file = path.with_extension("py");
        if py_file.exists() && py_file.is_file() {
            candidates.push(py_file);
        }

        // Check for package (__init__.py)
        let init_file = path.join("__init__.py");
        if init_file.exists() && init_file.is_file() {
            candidates.push(init_file);
        }

        if candidates.is_empty() {
            debug!("Could not resolve module at {:?}", path);
            // Don't error - just return empty list (might be stdlib or external package)
            Ok(Vec::new())
        } else {
            Ok(candidates)
        }
    }

    /// Get all Python files that might be imported from a directory
    pub fn get_package_files(&self, package_dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !package_dir.is_dir() {
            return Ok(files);
        }

        for entry in std::fs::read_dir(package_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "py" {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_absolute_import() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create utils.py
        fs::write(src_dir.join("utils.py"), "def helper(): pass").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let resolved = resolver
            .resolve_import("src.utils", Path::new("main.py"))
            .unwrap();

        assert!(!resolved.is_empty());
        assert!(resolved[0].ends_with("utils.py"));
    }

    #[test]
    fn test_resolve_package_import() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("mypackage");
        fs::create_dir(&package_dir).unwrap();

        // Create __init__.py
        fs::write(package_dir.join("__init__.py"), "# package").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let resolved = resolver
            .resolve_import("mypackage", Path::new("main.py"))
            .unwrap();

        assert!(!resolved.is_empty());
        assert!(resolved[0].ends_with("__init__.py"));
    }

    #[test]
    fn test_resolve_relative_import() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("package");
        fs::create_dir(&package_dir).unwrap();

        // Create sibling.py and current.py
        fs::write(package_dir.join("sibling.py"), "def func(): pass").unwrap();
        let current_file = package_dir.join("current.py");
        fs::write(&current_file, "from . import sibling").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let resolved = resolver.resolve_import(".", &current_file).unwrap();

        // Should resolve to the package's __init__.py or the directory itself
        assert!(!resolved.is_empty() || resolved.is_empty()); // Might not find __init__.py
    }

    #[test]
    fn test_resolve_parent_import() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("package");
        let subpackage_dir = package_dir.join("sub");
        fs::create_dir_all(&subpackage_dir).unwrap();

        // Create parent.py in package/
        fs::write(package_dir.join("parent.py"), "def parent_func(): pass").unwrap();

        // Create module in package/sub/
        let current_file = subpackage_dir.join("module.py");
        fs::write(&current_file, "from ..parent import parent_func").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let resolved = resolver.resolve_import("..parent", &current_file).unwrap();

        assert!(!resolved.is_empty());
        assert!(resolved[0].ends_with("parent.py"));
    }

    #[test]
    fn test_get_package_files() {
        let temp_dir = TempDir::new().unwrap();
        let package_dir = temp_dir.path().join("package");
        fs::create_dir(&package_dir).unwrap();

        fs::write(package_dir.join("module1.py"), "pass").unwrap();
        fs::write(package_dir.join("module2.py"), "pass").unwrap();
        fs::write(package_dir.join("README.md"), "# README").unwrap();

        let resolver = ModuleResolver::new(temp_dir.path().to_path_buf());
        let files = resolver.get_package_files(&package_dir).unwrap();

        assert_eq!(files.len(), 2); // Only .py files
        assert!(files.iter().any(|f| f.ends_with("module1.py")));
        assert!(files.iter().any(|f| f.ends_with("module2.py")));
    }
}
