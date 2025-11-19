//! Scanner registry for managing and discovering ecosystem scanners
//!
//! This module provides a registry that holds all available scanners and
//! can automatically detect which scanners apply to a given directory.

use std::collections::HashMap;
use std::path::Path;

use crate::scanner::Scanner;

/// Registry of all available ecosystem scanners
pub struct ScannerRegistry {
    scanners: HashMap<String, Box<dyn Scanner>>,
}

impl ScannerRegistry {
    /// Create a new empty scanner registry
    pub fn new() -> Self {
        Self {
            scanners: HashMap::new(),
        }
    }

    /// Register a scanner
    pub fn register(&mut self, scanner: Box<dyn Scanner>) {
        self.scanners.insert(scanner.name().to_string(), scanner);
    }

    /// Detect all scanners that apply to the given directory
    ///
    /// Returns a list of scanners that detected their ecosystem in the directory.
    pub fn detect_all(&self, root: &Path) -> Vec<&dyn Scanner> {
        self.scanners
            .values()
            .filter(|s| s.detect(root))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Get a specific scanner by name
    pub fn get(&self, name: &str) -> Option<&dyn Scanner> {
        self.scanners.get(name).map(|s| s.as_ref())
    }

    /// Get the number of registered scanners
    pub fn len(&self) -> usize {
        self.scanners.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.scanners.is_empty()
    }

    /// Get all scanner names
    pub fn scanner_names(&self) -> Vec<&str> {
        self.scanners.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ScannerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{ScanContext, Scanner};
    use crate::types::EcosystemScanResult;
    use async_trait::async_trait;

    struct TestScanner {
        name: String,
        detect_file: String,
    }

    impl TestScanner {
        fn new(name: &str, detect_file: &str) -> Self {
            Self {
                name: name.to_string(),
                detect_file: detect_file.to_string(),
            }
        }
    }

    #[async_trait]
    impl Scanner for TestScanner {
        fn name(&self) -> &str {
            &self.name
        }

        fn detect(&self, root: &Path) -> bool {
            root.join(&self.detect_file).exists()
        }

        async fn scan(&self, ctx: &ScanContext) -> anyhow::Result<EcosystemScanResult> {
            Ok(EcosystemScanResult::new(
                self.name.clone(),
                ctx.root.display().to_string(),
            ))
        }
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = ScannerRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());

        registry.register(Box::new(TestScanner::new("test", "test.txt")));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());

        let scanner = registry.get("test").unwrap();
        assert_eq!(scanner.name(), "test");
    }

    #[test]
    fn test_registry_detect() {
        let mut registry = ScannerRegistry::new();
        registry.register(Box::new(TestScanner::new("npm", "package.json")));
        registry.register(Box::new(TestScanner::new("python", "requirements.txt")));

        // Test with temp dir that has package.json
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("package.json"), "{}").unwrap();

        let detected = registry.detect_all(temp.path());
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].name(), "npm");
    }
}
