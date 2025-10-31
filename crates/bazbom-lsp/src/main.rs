// BazBOM Language Server Protocol implementation
// Provides real-time vulnerability scanning for IDE integration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct ScanResult {
    vulnerabilities: Vec<Vulnerability>,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Vulnerability {
    id: String,
    severity: String,
    package_name: String,
    current_version: String,
    fixed_version: Option<String>,
    summary: String,
}

struct BazBomLanguageServer {
    client: Client,
    #[allow(dead_code)]
    scan_cache: Arc<Mutex<HashMap<String, ScanResult>>>,
}

impl BazBomLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            scan_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn scan_and_publish_diagnostics(&self, uri: &Url) {
        let path = uri.path();
        
        // Only scan build files
        if !Self::is_build_file(path) {
            return;
        }

        tracing::info!("Scanning file: {}", path);

        // Run bazbom scan in the project directory
        let project_dir = std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".");

        let diagnostics = match self.run_bazbom_scan(project_dir).await {
            Ok(vulns) => self.convert_to_diagnostics(&vulns, uri),
            Err(e) => {
                tracing::error!("Scan failed: {}", e);
                Vec::new()
            }
        };

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    fn is_build_file(path: &str) -> bool {
        path.ends_with("pom.xml")
            || path.ends_with("build.gradle")
            || path.ends_with("build.gradle.kts")
            || path.ends_with("BUILD")
            || path.ends_with("BUILD.bazel")
    }

    async fn run_bazbom_scan(&self, project_dir: &str) -> Result<Vec<Vulnerability>> {
        // Create a temporary output directory
        let temp_dir = tempfile::tempdir()?;
        let out_dir = temp_dir.path();

        // Run bazbom scan command
        let output = tokio::process::Command::new("bazbom")
            .args(&[
                "scan",
                "--fast",  // Use fast mode for quick feedback
                "--out-dir",
                out_dir.to_str().unwrap(),
                project_dir,
            ])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "bazbom scan failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Read findings from output
        let findings_path = out_dir.join("sca_findings.json");
        if !findings_path.exists() {
            return Ok(Vec::new());
        }

        let findings_content = std::fs::read_to_string(findings_path)?;
        let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

        // Extract vulnerabilities
        let mut vulnerabilities = Vec::new();
        if let Some(vulns) = findings["vulnerabilities"].as_array() {
            for vuln in vulns {
                if let (Some(id), Some(severity)) = (
                    vuln["id"].as_str(),
                    vuln["severity"].as_object().and_then(|s| s["level"].as_str()),
                ) {
                    // Extract package info from affected packages
                    if let Some(affected) = vuln["affected"].as_array() {
                        for pkg in affected {
                            if let Some(package_name) = pkg["package"].as_str() {
                                let current_version = "unknown".to_string();
                                let fixed_version = pkg["ranges"]
                                    .as_array()
                                    .and_then(|ranges| {
                                        ranges.iter().find_map(|r| {
                                            r["events"].as_array().and_then(|events| {
                                                events.iter().find_map(|e| {
                                                    e["fixed"].as_str().map(|s| s.to_string())
                                                })
                                            })
                                        })
                                    });

                                let summary = vuln["summary"]
                                    .as_str()
                                    .unwrap_or("No summary available")
                                    .to_string();

                                vulnerabilities.push(Vulnerability {
                                    id: id.to_string(),
                                    severity: severity.to_string(),
                                    package_name: package_name.to_string(),
                                    current_version,
                                    fixed_version,
                                    summary,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(vulnerabilities)
    }

    fn convert_to_diagnostics(&self, vulnerabilities: &[Vulnerability], _uri: &Url) -> Vec<Diagnostic> {
        vulnerabilities
            .iter()
            .map(|vuln| {
                let severity = match vuln.severity.as_str() {
                    "Critical" => DiagnosticSeverity::ERROR,
                    "High" => DiagnosticSeverity::ERROR,
                    "Medium" => DiagnosticSeverity::WARNING,
                    "Low" => DiagnosticSeverity::INFORMATION,
                    _ => DiagnosticSeverity::HINT,
                };

                let message = if let Some(ref fixed) = vuln.fixed_version {
                    format!(
                        "{} ({}): {} in {} - Fixed in version {}",
                        vuln.id, vuln.severity, vuln.summary, vuln.package_name, fixed
                    )
                } else {
                    format!(
                        "{} ({}): {} in {} - No fix available",
                        vuln.id, vuln.severity, vuln.summary, vuln.package_name
                    )
                };

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    severity: Some(severity),
                    code: Some(NumberOrString::String(vuln.id.clone())),
                    source: Some("BazBOM".to_string()),
                    message,
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                }
            })
            .collect()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BazBomLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        tracing::info!("Initializing BazBOM Language Server");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "bazbom-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("BazBOM Language Server initialized");
        self.client
            .log_message(MessageType::INFO, "BazBOM Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        tracing::info!("Shutting down BazBOM Language Server");
        Ok(())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        tracing::info!("File saved: {}", params.text_document.uri);
        self.scan_and_publish_diagnostics(&params.text_document.uri)
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::info!("File opened: {}", params.text_document.uri);
        self.scan_and_publish_diagnostics(&params.text_document.uri)
            .await;
    }

    async fn code_action(&self, params: CodeActionParams) -> jsonrpc::Result<Option<CodeActionResponse>> {
        tracing::info!("Code action requested for: {}", params.text_document.uri);
        
        let mut actions = Vec::new();
        
        // Check if there are diagnostics in the range
        for diagnostic in &params.context.diagnostics {
            // Only create actions for BazBOM diagnostics
            if diagnostic.source.as_deref() != Some("BazBOM") {
                continue;
            }
            
            // Try to extract vulnerability info from diagnostic
            if let Some(NumberOrString::String(ref cve_id)) = diagnostic.code {
                // Extract fixed version from message if available
                let fixed_version = Self::extract_fixed_version(&diagnostic.message);
                
                if let Some(fixed) = fixed_version {
                    // Create a code action to upgrade dependency
                    let title = format!("Upgrade to safe version {}", fixed);
                    
                    let action = CodeActionOrCommand::CodeAction(CodeAction {
                        title,
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diagnostic.clone()]),
                        edit: None, // We'll handle this via command
                        command: Some(Command {
                            title: "Upgrade Dependency".to_string(),
                            command: "bazbom.upgrade".to_string(),
                            arguments: Some(vec![
                                serde_json::to_value(&params.text_document.uri).unwrap(),
                                serde_json::to_value(cve_id).unwrap(),
                                serde_json::to_value(fixed).unwrap(),
                            ]),
                        }),
                        is_preferred: Some(true),
                        disabled: None,
                        data: None,
                    });
                    
                    actions.push(action);
                }
            }
        }
        
        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

impl BazBomLanguageServer {
    fn extract_fixed_version(message: &str) -> Option<String> {
        // Try to extract "Fixed in version X.Y.Z" from message
        if let Some(idx) = message.find("Fixed in version ") {
            let version_start = idx + "Fixed in version ".len();
            let version_part = &message[version_start..];
            // Take until first space or end of string
            let version = version_part.split_whitespace().next()?;
            Some(version.to_string())
        } else {
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting BazBOM Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| BazBomLanguageServer::new(client));

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_build_file() {
        assert!(BazBomLanguageServer::is_build_file("/path/to/pom.xml"));
        assert!(BazBomLanguageServer::is_build_file("/path/to/build.gradle"));
        assert!(BazBomLanguageServer::is_build_file("/path/to/build.gradle.kts"));
        assert!(BazBomLanguageServer::is_build_file("/path/to/BUILD"));
        assert!(BazBomLanguageServer::is_build_file("/path/to/BUILD.bazel"));
        assert!(!BazBomLanguageServer::is_build_file("/path/to/Main.java"));
        assert!(!BazBomLanguageServer::is_build_file("/path/to/README.md"));
    }

    #[test]
    fn test_vulnerability_serialization() {
        let vuln = Vulnerability {
            id: "CVE-2021-44228".to_string(),
            severity: "Critical".to_string(),
            package_name: "org.apache.logging.log4j:log4j-core".to_string(),
            current_version: "2.14.1".to_string(),
            fixed_version: Some("2.21.1".to_string()),
            summary: "Remote code execution via JNDI".to_string(),
        };

        let json = serde_json::to_string(&vuln).unwrap();
        assert!(json.contains("CVE-2021-44228"));
        assert!(json.contains("Critical"));
    }
}
