# Phase 7: Supply Chain Threat Intelligence

**Status:** Planned
**Priority:**  P1 - High Impact
**Timeline:** Months 4-6 (10 weeks)
**Team Size:** 1 security researcher + 1 developer
**Dependencies:** Phase 0-3 complete, Phase 4 (IDE integration) recommended

---

## Executive Summary

**Goal:** Proactively detect supply chain attacks before they compromise your application.

**Current Gap:** BazBOM detects known CVEs. Doesn't detect malicious packages, typosquatting, or suspicious behavior.

**Target State:**
- Detect malicious packages using heuristics (obfuscation, suspicious network calls, crypto mining)
- Typosquatting detection (log4j vs. log4jj)
- Behavioral analysis (sudden version changes, maintainer switches)
- Continuous monitoring (`bazbom watch` command)

**Success Metrics:**
-  Detect 95% of known supply chain attacks (test against historical incidents)
-  Mean time to detection (MTTD) < 4 hours for new CVEs
-  False positive rate < 5%
-  Zero missed typosquatting in top 1000 Maven packages

**Competitive Context:** Checkmarx SCA and Socket.dev lead in this area. BazBOM should match their detection capabilities.

---

## Threat Landscape (2024-2025)

### Recent Supply Chain Attacks

**2024:**
- **XZ Utils Backdoor** (March 2024) - Nation-state backdoor in compression library
- **PyPI Malware** (Ongoing) - 1000+ malicious Python packages
- **NPM Typosquatting** (Ongoing) - Packages mimicking popular libraries

**2023:**
- **3CX Supply Chain Attack** - Compromised Windows/macOS installers
- **CircleCI Security Incident** - Stolen secrets from CI/CD

**2022:**
- **Log4Shell** (CVE-2021-44228) - Zero-day RCE in log4j
- **Spring4Shell** (CVE-2022-22965) - RCE in Spring Framework

### Attack Vectors

1. **Malicious Packages** - Intentionally harmful code uploaded to Maven Central, npm, PyPI
2. **Typosquatting** - Packages with similar names to popular libraries (commons-io vs. common-io)
3. **Dependency Confusion** - Internal package names exploited by public packages
4. **Compromised Maintainers** - Legitimate maintainer accounts hijacked
5. **Build System Attacks** - Malicious code injected during build (SolarWinds)

---

## 7.1 Malicious Package Detection

### Heuristic Analysis Engine

**Goal:** Detect malicious behavior without signatures

**Heuristics:**

#### 7.1.1 Code Obfuscation Detection

**Indicator:** Heavily obfuscated bytecode or string encryption

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/obfuscation.rs
pub struct ObfuscationDetector;

impl ObfuscationDetector {
    pub fn analyze_jar(&self, jar_path: &Path) -> Result<ObfuscationScore> {
        let mut score = 0.0;
        let jar = ZipArchive::new(File::open(jar_path)?)?;

        for i in 0..jar.len() {
            let mut file = jar.by_index(i)?;
            if file.name().ends_with(".class") {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;

                // Check for obfuscation indicators
                let class_reader = ClassReader::new(&bytes);

                // 1. Single-letter class/method names (e.g., class A { void a() })
                if class_reader.class_name.len() == 1 {
                    score += 0.2;
                }

                // 2. High ratio of goto/jsr instructions (control flow obfuscation)
                let goto_ratio = class_reader.count_gotos() as f32 / class_reader.total_instructions() as f32;
                if goto_ratio > 0.3 {
                    score += 0.3;
                }

                // 3. String encryption patterns (XOR loops, base64)
                if class_reader.has_string_encryption() {
                    score += 0.4;
                }

                // 4. Reflection abuse (Class.forName everywhere)
                let reflection_calls = class_reader.count_reflection_calls();
                if reflection_calls > 10 {
                    score += 0.3;
                }
            }
        }

        Ok(ObfuscationScore {
            score: score.min(1.0),
            risk: if score > 0.7 { Risk::High } else if score > 0.4 { Risk::Medium } else { Risk::Low },
        })
    }
}
```

#### 7.1.2 Suspicious Network Behavior

**Indicator:** HTTP/S requests to unknown domains, especially at startup

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/network.rs
pub struct NetworkBehaviorAnalyzer;

impl NetworkBehaviorAnalyzer {
    pub fn analyze_network_calls(&self, class_bytes: &[u8]) -> Result<NetworkRisk> {
        let class_reader = ClassReader::new(class_bytes);
        let mut suspicious_urls = Vec::new();

        // Find all string constants that look like URLs
        for constant in class_reader.constant_pool {
            if let Constant::String(s) = constant {
                if s.starts_with("http://") || s.starts_with("https://") {
                    // Check against whitelist of known-good domains
                    if !self.is_whitelisted_domain(&s) {
                        suspicious_urls.push(s.clone());
                    }
                }
            }
        }

        // Check for URL.openConnection() calls in static initializers
        // (malware often runs code on class load)
        let static_init_network = class_reader.has_network_in_static_init();

        Ok(NetworkRisk {
            suspicious_urls,
            calls_network_on_init: static_init_network,
            risk: if static_init_network { Risk::High } else if !suspicious_urls.is_empty() { Risk::Medium } else { Risk::Low },
        })
    }

    fn is_whitelisted_domain(&self, url: &str) -> bool {
        let known_good = [
            "maven.apache.org",
            "repo1.maven.org",
            "central.sonatype.org",
            "github.com",
            "gitlab.com",
            "bitbucket.org",
        ];

        known_good.iter().any(|domain| url.contains(domain))
    }
}
```

#### 7.1.3 Cryptocurrency Mining Detection

**Indicator:** CPU-intensive loops, known mining pool URLs

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/crypto_mining.rs
pub struct CryptoMiningDetector;

impl CryptoMiningDetector {
    pub fn detect_mining(&self, class_bytes: &[u8]) -> Result<MiningRisk> {
        let class_reader = ClassReader::new(class_bytes);

        // Known mining pool domains
        let mining_pools = [
            "pool.supportxmr.com",
            "xmr.pool.minergate.com",
            "monerohash.com",
            "minexmr.com",
        ];

        // Check for mining pool URLs in constants
        let has_mining_url = class_reader.constant_pool.iter().any(|c| {
            if let Constant::String(s) = c {
                mining_pools.iter().any(|pool| s.contains(pool))
            } else {
                false
            }
        });

        // Check for CPU-intensive loops (cryptographic operations)
        let has_crypto_loops = class_reader.methods.iter().any(|m| {
            // Look for tight loops with bitwise operations (typical of mining)
            m.has_pattern(&[
                Opcode::ILOAD,
                Opcode::IXOR,  // XOR operations
                Opcode::ISTORE,
                Opcode::GOTO,  // Loop
            ])
        });

        Ok(MiningRisk {
            has_mining_url,
            has_crypto_loops,
            risk: if has_mining_url { Risk::Critical } else if has_crypto_loops { Risk::Medium } else { Risk::Low },
        })
    }
}
```

#### 7.1.4 File System Access Patterns

**Indicator:** Reading/writing files outside application directory

**Patterns:**
- `new File("/etc/passwd")` - Reading system files
- `new File(System.getProperty("user.home"))` - Accessing user files
- `Runtime.exec("rm -rf /")` - Destructive commands

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/filesystem.rs
pub struct FileSystemAnalyzer;

impl FileSystemAnalyzer {
    pub fn analyze_file_access(&self, class_bytes: &[u8]) -> Result<FileSystemRisk> {
        let class_reader = ClassReader::new(class_bytes);

        let suspicious_paths = [
            "/etc/passwd",
            "/etc/shadow",
            "~/.ssh",
            "~/.aws",
            "/proc/",
            "C:\\Windows\\System32",
        ];

        let suspicious_file_access = class_reader.constant_pool.iter().any(|c| {
            if let Constant::String(s) = c {
                suspicious_paths.iter().any(|path| s.contains(path))
            } else {
                false
            }
        });

        // Check for Runtime.exec() calls
        let has_runtime_exec = class_reader.has_method_call("java/lang/Runtime", "exec");

        Ok(FileSystemRisk {
            suspicious_file_access,
            has_runtime_exec,
            risk: if suspicious_file_access || has_runtime_exec { Risk::High } else { Risk::Low },
        })
    }
}
```

### Malicious Package Database

**Curated List:** Known malicious packages from historical incidents

**Sources:**
- Sonatype OSS Index (malicious packages)
- Checkmarx Supply Chain Security reports
- Manual research (XZ Utils, 3CX, etc.)

**Implementation:**
```json
// crates/bazbom/data/malicious-packages.json
{
  "malicious_packages": [
    {
      "purl": "pkg:maven/com.example/malicious-lib@1.0.0",
      "reason": "Contains cryptocurrency miner",
      "discovered": "2024-03-15",
      "source": "Sonatype OSS Index"
    },
    {
      "purl": "pkg:maven/org.xz/xz-java@5.6.0",
      "reason": "Backdoor in compression library (CVE-2024-3094)",
      "discovered": "2024-03-29",
      "source": "Researcher disclosure"
    }
  ]
}
```

**CLI Integration:**
```bash
# Scan for malicious packages
bazbom scan --detect-malicious

# Output:
#  MALICIOUS PACKAGE DETECTED!
# Package: org.xz:xz-java@5.6.0
# Reason: Backdoor in compression library (CVE-2024-3094)
# Risk: CRITICAL
# Action: Remove immediately from dependencies
```

---

## 7.2 Typosquatting Detection

### Algorithm: Levenshtein Distance

**Goal:** Detect packages with names similar to popular packages

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/typosquat.rs
use strsim::levenshtein;

pub struct TyposquatDetector {
    popular_packages: Vec<String>,  // Top 10K Maven packages
}

impl TyposquatDetector {
    pub fn check_dependency(&self, dep_name: &str) -> Option<TyposquatWarning> {
        for popular in &self.popular_packages {
            let distance = levenshtein(dep_name, popular);

            // Typosquatting likely if:
            // 1. Edit distance = 1-2 (e.g., commons-io vs. common-io)
            // 2. Not exact match
            if distance >= 1 && distance <= 2 && dep_name != popular {
                return Some(TyposquatWarning {
                    detected_package: dep_name.to_string(),
                    similar_to: popular.clone(),
                    edit_distance: distance,
                    risk: if distance == 1 { Risk::High } else { Risk::Medium },
                });
            }
        }

        None
    }

    pub fn load_popular_packages() -> Vec<String> {
        // Load from Maven Central top downloads
        include_str!("../data/maven-top-10k.txt")
            .lines()
            .map(|s| s.to_string())
            .collect()
    }
}
```

**Data Source:**
```bash
# Fetch top Maven packages from Maven Central stats
curl https://search.maven.org/stats/top-packages > maven-top-10k.txt
```

**Example Detections:**
- `commons-io` → `common-io` (edit distance = 1)  HIGH RISK
- `log4j-core` → `log4jj-core` (edit distance = 1)  HIGH RISK
- `jackson-databind` → `jakson-databind` (edit distance = 2)  MEDIUM RISK

---

## 7.3 Behavioral Analysis

### 7.3.1 Version Anomaly Detection

**Indicators:**
- Sudden version jump (e.g., 1.0.0 → 99.0.0)
- Rapid version releases (10 versions in 1 day)
- Version retraction (removed from Maven Central)

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/version_anomaly.rs
pub struct VersionAnomalyDetector;

impl VersionAnomalyDetector {
    pub fn analyze_version_history(&self, package: &str) -> Result<Vec<Anomaly>> {
        let versions = self.fetch_version_history(package)?;
        let mut anomalies = Vec::new();

        // Check for sudden version jumps
        for window in versions.windows(2) {
            let (prev, curr) = (&window[0], &window[1]);
            let prev_major = prev.version.split('.').next().unwrap().parse::<i32>()?;
            let curr_major = curr.version.split('.').next().unwrap().parse::<i32>()?;

            if curr_major - prev_major > 10 {
                anomalies.push(Anomaly {
                    type_: AnomalyType::SuddenVersionJump,
                    description: format!("Version jumped from {} to {}", prev.version, curr.version),
                    risk: Risk::Medium,
                });
            }
        }

        // Check for rapid releases
        let releases_last_24h = versions.iter()
            .filter(|v| v.release_date > Utc::now() - Duration::days(1))
            .count();

        if releases_last_24h > 5 {
            anomalies.push(Anomaly {
                type_: AnomalyType::RapidReleases,
                description: format!("{} releases in last 24 hours", releases_last_24h),
                risk: Risk::High,
            });
        }

        Ok(anomalies)
    }
}
```

### 7.3.2 Maintainer Change Detection

**Indicator:** Package maintainer changed, especially for popular packages

**Implementation:**
```rust
// crates/bazbom/src/threat_intelligence/maintainer.rs
pub struct MaintainerChangeDetector;

impl MaintainerChangeDetector {
    pub fn detect_maintainer_change(&self, package: &str) -> Result<Option<MaintainerChange>> {
        let current_maintainer = self.fetch_current_maintainer(package)?;
        let historical_maintainers = self.fetch_maintainer_history(package)?;

        // Check if maintainer recently changed
        if let Some(previous) = historical_maintainers.last() {
            if previous.name != current_maintainer.name {
                let days_since_change = (Utc::now() - previous.change_date).num_days();

                return Ok(Some(MaintainerChange {
                    package: package.to_string(),
                    old_maintainer: previous.name.clone(),
                    new_maintainer: current_maintainer.name.clone(),
                    days_since_change,
                    risk: if days_since_change < 30 { Risk::High } else { Risk::Low },
                }));
            }
        }

        Ok(None)
    }
}
```

**Data Source:** Maven Central POM metadata, GitHub repository ownership

---

## 7.4 Continuous Monitoring

### `bazbom watch` Command

**Goal:** Monitor dependencies for new vulnerabilities 24/7

**Implementation:**
```rust
// crates/bazbom/src/commands/watch.rs
pub struct WatchCommand {
    interval: Duration,
    notification_channel: NotificationChannel,
}

impl WatchCommand {
    pub async fn run(&self) -> Result<()> {
        println!("Starting BazBOM continuous monitoring...");
        println!("Checking for new vulnerabilities every {:?}", self.interval);

        let mut last_scan_results = self.initial_scan().await?;

        loop {
            tokio::time::sleep(self.interval).await;

            println!("[{}] Checking for new vulnerabilities...", Utc::now().format("%Y-%m-%d %H:%M:%S"));

            let current_scan_results = self.scan_dependencies().await?;

            // Compare with previous scan
            let new_vulnerabilities = self.diff_scans(&last_scan_results, &current_scan_results);

            if !new_vulnerabilities.is_empty() {
                println!(" Found {} new vulnerabilities!", new_vulnerabilities.len());

                for vuln in &new_vulnerabilities {
                    println!("  - {} in {} ({})", vuln.id, vuln.package, vuln.severity);
                }

                // Send notifications
                self.notify(&new_vulnerabilities).await?;
            } else {
                println!(" No new vulnerabilities detected.");
            }

            last_scan_results = current_scan_results;
        }
    }

    async fn notify(&self, vulns: &[Vulnerability]) -> Result<()> {
        match &self.notification_channel {
            NotificationChannel::Slack(webhook) => {
                let message = format!(
                    " BazBOM Alert: {} new vulnerabilities detected!\n\n{}",
                    vulns.len(),
                    vulns.iter()
                        .map(|v| format!("• {} in {} ({})", v.id, v.package, v.severity))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
                send_slack_notification(webhook, &message).await?;
            }
            NotificationChannel::Email(addresses) => {
                // Send email alerts
                send_email_notification(addresses, vulns).await?;
            }
            NotificationChannel::Stdout => {
                // Just print to console (default)
            }
        }

        Ok(())
    }
}
```

**CLI Usage:**
```bash
# Monitor every hour (default)
bazbom watch

# Monitor every 15 minutes
bazbom watch --interval 15m

# With Slack notifications
bazbom watch --notify-slack https://hooks.slack.com/services/XXX

# With email notifications
bazbom watch --notify-email security-team@company.com

# Run as systemd service (Linux)
sudo systemctl enable bazbom-watch
sudo systemctl start bazbom-watch
```

**Systemd Service:**
```ini
# /etc/systemd/system/bazbom-watch.service
[Unit]
Description=BazBOM Continuous Vulnerability Monitoring
After=network.target

[Service]
Type=simple
User=bazbom
WorkingDirectory=/opt/app
ExecStart=/usr/local/bin/bazbom watch --interval 1h --notify-slack https://hooks.slack.com/services/XXX
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

---

## 7.5 Dependency Update Intelligence

### "Should I Upgrade?" Decision Engine

**Goal:** Balance security (fix CVE) vs. stability (avoid breaking changes)

**Factors:**
- **Severity:** CRITICAL/HIGH = upgrade ASAP
- **Exploitability:** CISA KEV, EPSS score
- **Breaking Changes:** Major version bump (1.x → 2.x)
- **Community Adoption:** How many others already upgraded?
- **Test Coverage:** Do you have tests to catch regressions?

**Implementation:**
```rust
// crates/bazbom/src/intelligence/upgrade_advisor.rs
pub struct UpgradeAdvisor;

impl UpgradeAdvisor {
    pub fn should_upgrade(&self, dep: &Dependency, vuln: &Vulnerability) -> UpgradeRecommendation {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Factor 1: Vulnerability severity
        match vuln.severity.as_str() {
            "CRITICAL" => { score += 1.0; reasons.push("CRITICAL vulnerability"); }
            "HIGH" => { score += 0.7; reasons.push("HIGH severity"); }
            "MEDIUM" => { score += 0.4; reasons.push("MEDIUM severity"); }
            _ => { score += 0.1; }
        }

        // Factor 2: Exploitability
        if vuln.cisa_kev {
            score += 0.5;
            reasons.push("Actively exploited (CISA KEV)");
        }

        if vuln.epss >= 0.5 {
            score += 0.3;
            reasons.push(format!("High exploit probability (EPSS: {:.1}%)", vuln.epss * 100.0));
        }

        // Factor 3: Breaking changes
        let current_version = semver::Version::parse(&dep.version)?;
        let target_version = semver::Version::parse(&vuln.fixed_version)?;

        if target_version.major > current_version.major {
            score -= 0.3;
            reasons.push(" Major version upgrade (may have breaking changes)");
        } else if target_version.minor > current_version.minor {
            score -= 0.1;
            reasons.push("Minor version upgrade (check release notes)");
        }

        // Factor 4: Community adoption
        let adoption_rate = self.check_adoption_rate(&dep.name, &vuln.fixed_version)?;
        if adoption_rate > 0.5 {
            score += 0.2;
            reasons.push(format!("{}% of users already upgraded", (adoption_rate * 100.0) as i32));
        }

        // Decision threshold
        let recommendation = if score >= 0.8 {
            UpgradeDecision::UpgradeImmediately
        } else if score >= 0.5 {
            UpgradeDecision::UpgradeSoon
        } else if score >= 0.3 {
            UpgradeDecision::UpgradeWhenConvenient
        } else {
            UpgradeDecision::MonitorOnly
        };

        UpgradeRecommendation {
            decision: recommendation,
            score,
            reasons,
            risk_without_upgrade: vuln.severity.clone(),
            breaking_change_risk: if target_version.major > current_version.major { Risk::High } else { Risk::Low },
        }
    }
}
```

**CLI Output:**
```bash
bazbom upgrade-advisor

# Output:
# Upgrade Recommendations:
#
# 1. log4j-core: 2.17.0 → 2.21.1
#    Decision: UPGRADE IMMEDIATELY (score: 0.95)
#    Reasons:
#      - CRITICAL vulnerability (CVE-2021-44832)
#      - Actively exploited (CISA KEV)
#      - Patch version (no breaking changes)
#      - 85% of users already upgraded
#
# 2. spring-web: 5.3.20 → 6.0.0
#    Decision: UPGRADE SOON (score: 0.65)
#    Reasons:
#      - HIGH severity (CVE-2024-xxxx)
#      -  Major version upgrade (may have breaking changes)
#      - 35% of users already upgraded
#    Recommendation: Review Spring 6.0 migration guide first
```

---

## Success Criteria

### Phase 7 Completion Checklist

- [ ] Malicious package detection engine implemented (obfuscation, network, crypto mining, filesystem)
- [ ] Typosquatting detection with Levenshtein distance (<= 2 edits)
- [ ] Behavioral anomaly detection (version jumps, maintainer changes)
- [ ] `bazbom watch` command runs continuously
- [ ] Notifications work (Slack, email, stdout)
- [ ] Upgrade advisor provides actionable recommendations
- [ ] Test suite includes historical supply chain attacks
- [ ] False positive rate < 5%
- [ ] Detection rate > 95% on known attacks

### Performance Benchmarks

| Metric | Target |
|--------|--------|
| **Malicious package scan (1K deps)** | <10 seconds |
| **Typosquatting check (10K packages)** | <5 seconds |
| **Watch mode latency (new CVE)** | <4 hours |
| **False positive rate** | <5% |
| **Detection rate (known attacks)** | >95% |

---

## Resource Requirements

**Team:** 1 security researcher + 1 developer for 10 weeks
**Skills:** Security research, bytecode analysis, data analysis
**Budget:** $40K-60K (mix of internal + contractor)

**Data Sources:**
- Maven Central API (package metadata)
- Sonatype OSS Index (malicious packages)
- CISA KEV (actively exploited CVEs)
- Historical attack database (XZ Utils, 3CX, etc.)

---

## Competitive Benchmark

**After Phase 7:**

| Feature | Checkmarx SCA | Socket.dev | BazBOM |
|---------|--------------|------------|--------|
| **Malicious Package Detection** |  Advanced |  Advanced |  Good |
| **Typosquatting** |  Yes |  Yes |  Yes |
| **Behavioral Analysis** |  Advanced |  Advanced |  Basic |
| **Continuous Monitoring** |  Yes |  Yes |  Yes |
| **Cost** | $200+/dev/year | $50/dev/year | **FREE** |

**BazBOM Advantage:** Only open source tool with comprehensive threat intelligence.

---

**Last Updated:** 2025-10-30
**Next:** Phase 9 (Ecosystem Expansion)
