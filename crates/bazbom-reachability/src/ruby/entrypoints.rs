//! Ruby entrypoint detection

use crate::ast_parser::parse_file;
use crate::error::Result;
use crate::models::{Entrypoint, EntrypointType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Detects Ruby entrypoints in a project
pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Detect all entrypoints in the project
    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        info!("Detecting Ruby entrypoints");

        let mut entrypoints = Vec::new();

        // Walk through all Ruby files
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::should_skip(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if Self::is_ruby_file(path) {
                    if let Ok(file_entrypoints) = self.detect_in_file(path) {
                        entrypoints.extend(file_entrypoints);
                    }
                }
            }
        }

        info!("Found {} Ruby entrypoints", entrypoints.len());

        Ok(entrypoints)
    }

    /// Detect entrypoints in a single Ruby file
    fn detect_in_file(&self, file_path: &Path) -> Result<Vec<Entrypoint>> {
        let source = std::fs::read(file_path)?;
        let source_str = String::from_utf8_lossy(&source);
        let tree = parse_file(file_path)?;

        let mut entrypoints = Vec::new();
        let root_node = tree.root_node();

        // Check file path for framework patterns
        let path_str = file_path.to_string_lossy();

        // Rails controllers
        if path_str.contains("app/controllers/") {
            Self::extract_controller_actions(&root_node, &source, file_path, &mut entrypoints);
        }

        // Rails jobs
        if path_str.contains("app/jobs/") {
            Self::extract_job_perform(&root_node, &source, file_path, &mut entrypoints);
        }

        // Rails mailers
        if path_str.contains("app/mailers/") {
            Self::extract_mailer_actions(&root_node, &source, file_path, &mut entrypoints);
        }

        // RSpec tests
        if path_str.ends_with("_spec.rb") {
            Self::extract_rspec_tests(&root_node, &source, file_path, &mut entrypoints);
        }

        // Minitest tests
        if path_str.ends_with("_test.rb") || path_str.contains("test/") {
            Self::extract_minitest_tests(&root_node, &source, file_path, &mut entrypoints);
        }

        // Sinatra routes
        if source_str.contains("Sinatra")
            || source_str.contains("get ")
            || source_str.contains("post ")
        {
            Self::extract_sinatra_routes(&root_node, &source, file_path, &mut entrypoints);
        }

        // Rake tasks
        if path_str.ends_with(".rake") || source_str.contains("task ") {
            Self::extract_rake_tasks(&root_node, &source, file_path, &mut entrypoints);
        }

        Ok(entrypoints)
    }

    fn extract_controller_actions(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_public_methods(
            node,
            source,
            file_path,
            EntrypointType::RailsController,
            entrypoints,
        );
    }

    fn extract_job_perform(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_method_name(
            node,
            source,
            file_path,
            "perform",
            EntrypointType::RailsJob,
            entrypoints,
        );
    }

    fn extract_mailer_actions(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_public_methods(
            node,
            source,
            file_path,
            EntrypointType::RailsMailer,
            entrypoints,
        );
    }

    fn extract_rspec_tests(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_blocks(
            node,
            source,
            file_path,
            &["it", "specify", "example"],
            EntrypointType::RSpecTest,
            entrypoints,
        );
    }

    fn extract_minitest_tests(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_method_prefix(
            node,
            source,
            file_path,
            "test_",
            EntrypointType::MinitestTest,
            entrypoints,
        );
    }

    fn extract_sinatra_routes(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_blocks(
            node,
            source,
            file_path,
            &["get", "post", "put", "patch", "delete"],
            EntrypointType::SinatraRoute,
            entrypoints,
        );
    }

    fn extract_rake_tasks(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        Self::visit_for_blocks(
            node,
            source,
            file_path,
            &["task"],
            EntrypointType::RakeTask,
            entrypoints,
        );
    }

    fn visit_for_public_methods(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entry_type: EntrypointType,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "method" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let method_name = get_node_text(&name_node, source);
                debug!(
                    "Found {} method: {}",
                    entry_type_str(&entry_type),
                    method_name
                );
                entrypoints.push(Entrypoint {
                    file: file_path.to_path_buf(),
                    function_name: method_name,
                    entrypoint_type: entry_type.clone(),
                    metadata: HashMap::new(),
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::visit_for_public_methods(
                &child,
                source,
                file_path,
                entry_type.clone(),
                entrypoints,
            );
        }
    }

    fn visit_for_method_name(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        target_name: &str,
        entry_type: EntrypointType,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "method" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let method_name = get_node_text(&name_node, source);
                if method_name == target_name {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: method_name,
                        entrypoint_type: entry_type.clone(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::visit_for_method_name(
                &child,
                source,
                file_path,
                target_name,
                entry_type.clone(),
                entrypoints,
            );
        }
    }

    fn visit_for_method_prefix(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        prefix: &str,
        entry_type: EntrypointType,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "method" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let method_name = get_node_text(&name_node, source);
                if method_name.starts_with(prefix) {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: method_name,
                        entrypoint_type: entry_type.clone(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::visit_for_method_prefix(
                &child,
                source,
                file_path,
                prefix,
                entry_type.clone(),
                entrypoints,
            );
        }
    }

    fn visit_for_blocks(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        block_names: &[&str],
        entry_type: EntrypointType,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "call" {
            if let Some(method_node) = node.child_by_field_name("method") {
                let method_name = get_node_text(&method_node, source);
                if block_names.contains(&method_name.as_str()) {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: format!("{}:{}", entry_type_str(&entry_type), method_name),
                        entrypoint_type: entry_type.clone(),
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::visit_for_blocks(
                &child,
                source,
                file_path,
                block_names,
                entry_type.clone(),
                entrypoints,
            );
        }
    }

    fn should_skip(entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["vendor", ".git", "node_modules", "tmp", "log"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    fn is_ruby_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "rb" || ext == "rake"
        } else {
            false
        }
    }
}

fn get_node_text(node: &tree_sitter::Node, source: &[u8]) -> String {
    node.utf8_text(source).unwrap_or("").to_string()
}

fn entry_type_str(entry_type: &EntrypointType) -> &'static str {
    match entry_type {
        EntrypointType::RailsController => "Rails Controller",
        EntrypointType::RailsJob => "Rails Job",
        EntrypointType::RailsMailer => "Rails Mailer",
        EntrypointType::RSpecTest => "RSpec Test",
        EntrypointType::MinitestTest => "Minitest",
        EntrypointType::SinatraRoute => "Sinatra Route",
        EntrypointType::RakeTask => "Rake Task",
        EntrypointType::Main => "Main",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rspec_tests() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("example_spec.rb");

        let code = r#"
RSpec.describe "Example" do
  it "does something" do
    expect(1 + 1).to eq(2)
  end

  it "does another thing" do
    expect(2 + 2).to eq(4)
  end
end
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(entrypoints.len() >= 2);
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::RSpecTest));
    }

    #[test]
    fn test_detect_rails_controller() {
        let temp_dir = TempDir::new().unwrap();
        let controllers_dir = temp_dir.path().join("app/controllers");
        fs::create_dir_all(&controllers_dir).unwrap();

        let file_path = controllers_dir.join("users_controller.rb");

        let code = r#"
class UsersController < ApplicationController
  def index
    @users = User.all
  end

  def show
    @user = User.find(params[:id])
  end
end
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::RailsController));
    }

    #[test]
    fn test_detect_rake_task() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("tasks.rake");

        let code = r#"
namespace :db do
  task :seed do
    puts "Seeding database"
  end
end
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::RakeTask));
    }
}
