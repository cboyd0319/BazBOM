# Implementation Plan: UX Improvements for BazBOM

**Document Version:** 1.0  
**Date:** November 3, 2025  
**Priority:** P0 - Critical for adoption  
**Estimated Timeline:** 6-8 weeks total

## Overview

This document provides detailed specifications for implementing the Priority 0 UX improvements identified in the competitive analysis. These changes will dramatically improve the developer onboarding experience and make BazBOM easier to use than commercial alternatives.

## Phase 1: Quick Wins (Weeks 1-2)

### 1.1 Interactive `bazbom init` Command

**Goal:** Reduce time-to-first-scan from 15-20 minutes to < 5 minutes

**Implementation:**

```rust
// crates/bazbom/src/commands/init.rs
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};

pub struct InitCommand {
    interactive: bool,
    skip_scan: bool,
}

impl InitCommand {
    pub async fn execute(&self) -> Result<()> {
        println!("Welcome to BazBOM! \n");
        println!("Let's get your first scan running in under 5 minutes.\n");
        
        // Step 1: Detect build system
        let build_system = self.detect_build_system()?;
        let confirmed = self.confirm_build_system(&build_system)?;
        
        // Step 2: Select policy template
        let policy = self.select_policy_template()?;
        
        // Step 3: Initialize project
        self.create_config_file(&build_system, &policy)?;
        
        // Step 4: Run first scan (if not skipped)
        if !self.skip_scan && self.confirm_scan()? {
            self.run_first_scan().await?;
            self.show_next_steps()?;
        }
        
        Ok(())
    }
    
    fn detect_build_system(&self) -> Result<BuildSystem> {
        // Check for pom.xml, build.gradle, BUILD.bazel, etc.
        // Return detected build system with confidence score
    }
    
    fn select_policy_template(&self) -> Result<PolicyTemplate> {
        let templates = vec![
            "Corporate Standard (recommended)",
            "PCI-DSS Compliance",
            "HIPAA Security Rule",
            "FedRAMP Moderate",
            "SOC 2 Type II",
            "Development (permissive)",
            "Skip policy setup",
        ];
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Enable policy template?")
            .default(0)
            .items(&templates)
            .interact()?;
            
        PolicyTemplate::from_selection(selection)
    }
    
    async fn run_first_scan(&self) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        pb.set_message("Scanning dependencies...");
        
        let scan_result = scan::run_scan(ScanOptions::default()).await?;
        
        pb.finish_with_message(" Scan complete!");
        
        self.display_scan_summary(&scan_result)?;
        
        Ok(())
    }
    
    fn display_scan_summary(&self, result: &ScanResult) -> Result<()> {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Found {} vulnerabilities:\n", result.total_vulnerabilities);
        
        if result.critical > 0 {
            println!("   CRITICAL ({}): {}", 
                result.critical, 
                result.critical_cves.join(", "));
            println!("     Fix available: bazbom fix --apply");
        }
        
        if result.high > 0 {
            println!("   HIGH ({}): View with bazbom findings --interactive", 
                result.high);
        }
        
        if result.medium > 0 {
            println!("   MEDIUM ({}): View full report in sca_findings.json", 
                result.medium);
        }
        
        Ok(())
    }
    
    fn show_next_steps(&self) -> Result<()> {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Next steps:");
        println!("  1. Review findings: bazbom findings --interactive");
        println!("  2. Apply fixes: bazbom fix --apply");
        println!("  3. Set up pre-commit hooks: bazbom install-hooks");
        println!("  4. Install IDE plugin: https://docs.bazbom.com/ide-integration");
        println!("\nLearn more: bazbom quickstart");
        
        Ok(())
    }
}
```

**Dependencies:**
- `dialoguer = "0.11"` - Interactive prompts
- `indicatif = "0.17"` - Progress bars

**Files to Create/Modify:**
- `crates/bazbom/src/commands/init.rs` (NEW)
- `crates/bazbom/src/commands/mod.rs` (MODIFY - add init command)
- `crates/bazbom/src/main.rs` (MODIFY - add init subcommand)

**Testing:**
- Unit tests for build system detection
- Integration tests for full init flow
- Manual testing on sample projects (Maven, Gradle, Bazel)

**Documentation:**
- Update `docs/USAGE.md` with init command
- Create `docs/QUICKSTART.md` with guided walkthrough
- Add video demo to README.md

### 1.2 Policy Template Discovery

**Goal:** Make policy templates easily discoverable and applicable

**Implementation:**

```rust
// crates/bazbom/src/commands/policy/init.rs
pub struct PolicyInitCommand {
    list: bool,
    template: Option<String>,
    output: PathBuf,
}

impl PolicyInitCommand {
    pub fn execute(&self) -> Result<()> {
        if self.list {
            self.list_templates()?;
        } else if let Some(template_name) = &self.template {
            self.init_template(template_name)?;
        } else {
            self.interactive_init()?;
        }
        Ok(())
    }
    
    fn list_templates(&self) -> Result<()> {
        println!("Available Policy Templates:\n");
        
        println!("  Regulatory Compliance:");
        println!("  - pci-dss        PCI-DSS v4.0 (Payment Card Industry)");
        println!("  - hipaa          HIPAA Security Rule (Healthcare)");
        println!("  - fedramp        FedRAMP Moderate (Federal)");
        println!("  - soc2           SOC 2 Type II (SaaS/B2B)\n");
        
        println!("  Development:");
        println!("  - corporate-standard    Balanced security for most teams");
        println!("  - corporate-permissive  Lighter policy for dev environments");
        println!("  - strict-security       Maximum security for critical systems\n");
        
        println!("  Framework-Specific:");
        println!("  - spring-boot    Optimized for Spring Boot applications");
        println!("  - react          Optimized for React/Node.js projects");
        println!("  - android        Optimized for Android development\n");
        
        println!("Initialize a template:");
        println!("  bazbom policy init --template pci-dss\n");
        
        Ok(())
    }
    
    fn init_template(&self, template_name: &str) -> Result<()> {
        let template = PolicyTemplate::load(template_name)?;
        let config = template.to_config()?;
        
        let output_path = self.output.join("bazbom.yml");
        config.save(&output_path)?;
        
        println!(" Policy initialized: {}", output_path.display());
        println!("\nTemplate: {}", template.name);
        println!("Description: {}", template.description);
        
        if !template.customize_hints.is_empty() {
            println!("\nCustomization hints:");
            for hint in &template.customize_hints {
                println!("  • {}", hint);
            }
        }
        
        println!("\nValidate your policy: bazbom policy validate");
        println!("Test your policy: bazbom policy check --dry-run");
        
        Ok(())
    }
}

// crates/bazbom-policy/src/templates.rs
pub struct PolicyTemplate {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub config: PolicyConfig,
    pub customize_hints: Vec<String>,
}

impl PolicyTemplate {
    pub fn all() -> Vec<PolicyTemplate> {
        vec![
            Self::pci_dss(),
            Self::hipaa(),
            Self::fedramp(),
            Self::soc2(),
            Self::corporate_standard(),
            Self::corporate_permissive(),
            Self::strict_security(),
            Self::spring_boot(),
            Self::react(),
            Self::android(),
        ]
    }
    
    fn pci_dss() -> PolicyTemplate {
        PolicyTemplate {
            name: "PCI-DSS v4.0".to_string(),
            description: "Payment Card Industry Data Security Standard".to_string(),
            category: TemplateCategory::Regulatory,
            config: PolicyConfig {
                severity_threshold: Some(Severity::High),
                kev_gate: true,
                epss_threshold: Some(0.5),
                reachability_required: false,
                vex_auto_apply: true,
                license_allowlist: vec![
                    "MIT".to_string(),
                    "Apache-2.0".to_string(),
                    "BSD-3-Clause".to_string(),
                ],
                license_denylist: vec![
                    "GPL-2.0".to_string(),
                    "GPL-3.0".to_string(),
                    "AGPL-3.0".to_string(),
                ],
            },
            customize_hints: vec![
                "Consider enabling reachability_required for production".to_string(),
                "Review license allowlist for your specific needs".to_string(),
                "Set up Slack notifications for CRITICAL findings".to_string(),
            ],
        }
    }
    
    // Similar implementations for other templates...
}
```

**Files to Create/Modify:**
- `crates/bazbom/src/commands/policy/init.rs` (NEW)
- `crates/bazbom-policy/src/templates.rs` (NEW)
- `crates/bazbom-policy/src/templates/` (NEW DIR - template YAML files)
- `crates/bazbom/src/commands/policy/mod.rs` (MODIFY)

**Policy Template Files:**
Create template YAML files in `crates/bazbom-policy/src/templates/`:
- `pci-dss.yml`
- `hipaa.yml`
- `fedramp.yml`
- `soc2.yml`
- `corporate-standard.yml`
- `corporate-permissive.yml`
- `strict-security.yml`
- `spring-boot.yml`
- `react.yml`
- `android.yml`

### 1.3 Terminal-Based Interactive Graph

**Goal:** Visualize dependency tree without external tools

**Implementation:**

```rust
// crates/bazbom/src/commands/graph.rs
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

pub struct GraphCommand {
    interactive: bool,
    format: GraphFormat,
    output: Option<PathBuf>,
}

impl GraphCommand {
    pub async fn execute(&self) -> Result<()> {
        let graph = self.load_dependency_graph().await?;
        
        if self.interactive {
            self.show_interactive_graph(&graph)?;
        } else {
            self.export_graph(&graph)?;
        }
        
        Ok(())
    }
    
    fn show_interactive_graph(&self, graph: &DependencyGraph) -> Result<()> {
        enable_raw_mode()?;
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let mut app = GraphApp::new(graph.clone());
        
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ])
                    .split(f.size());
                
                // Header
                let header = self.render_header(&app);
                f.render_widget(header, chunks[0]);
                
                // Dependency tree
                let tree = self.render_tree(&app);
                f.render_widget(tree, chunks[1]);
                
                // Commands
                let commands = self.render_commands();
                f.render_widget(commands, chunks[2]);
            })?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::Enter => app.expand_selected(),
                    KeyCode::Char('v') => app.view_details(),
                    KeyCode::Char('f') => app.filter_prompt(),
                    KeyCode::Char('/') => app.search_prompt(),
                    _ => {}
                }
            }
        }
        
        disable_raw_mode()?;
        Ok(())
    }
    
    fn render_header(&self, app: &GraphApp) -> Paragraph {
        let title = format!(
            "┌─ {} ────────────────────────────────────────┐\n\
             │  Runtime: JVM 11                           │\n\
             │  Dependencies: {} ({} direct, {} transitive)│\n\
             │  Vulnerabilities: {}                      │\n\
             └────────────────────────────────────────────┘",
            app.root_package,
            app.total_deps,
            app.direct_deps,
            app.transitive_deps,
            app.vulnerabilities
        );
        
        Paragraph::new(title)
            .style(Style::default().fg(Color::Green))
    }
    
    fn render_tree(&self, app: &GraphApp) -> List {
        let items: Vec<ListItem> = app
            .visible_nodes()
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let prefix = self.tree_prefix(node);
                let icon = self.status_icon(node);
                let content = format!("{}{} {}", prefix, icon, node.name);
                
                let style = if i == app.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    self.node_style(node)
                };
                
                ListItem::new(content).style(style)
            })
            .collect();
        
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Dependencies"))
    }
    
    fn render_commands(&self) -> Paragraph {
        let text = "[↑↓] Navigate [Enter] Expand [v] View details [f] Filter [/] Search [q] Quit";
        Paragraph::new(text)
            .style(Style::default().fg(Color::DarkGray))
    }
    
    fn status_icon(&self, node: &DependencyNode) -> &str {
        match node.status {
            NodeStatus::Critical => "",
            NodeStatus::High => "",
            NodeStatus::Medium => "",
            NodeStatus::Low => "",
            NodeStatus::Safe => "",
        }
    }
    
    fn tree_prefix(&self, node: &DependencyNode) -> String {
        let indent = "  ".repeat(node.depth);
        let connector = if node.is_last {
            "└──"
        } else {
            "├──"
        };
        format!("{}{} ", indent, connector)
    }
}

struct GraphApp {
    graph: DependencyGraph,
    selected_index: usize,
    expanded_nodes: HashSet<String>,
    filter: Option<String>,
    // ... other state
}
```

**Dependencies:**
- `crossterm = "0.27"` - Terminal manipulation
- `tui = "0.19"` - Terminal UI framework
- `ratatui = "0.26"` - Modern TUI (alternative to tui-rs)

**Files to Create/Modify:**
- `crates/bazbom/src/commands/graph.rs` (MODIFY - add interactive mode)
- `crates/bazbom/src/ui/` (NEW DIR - TUI components)
- `crates/bazbom/src/ui/graph.rs` (NEW)
- `crates/bazbom/src/ui/mod.rs` (NEW)

**Testing:**
- Unit tests for tree rendering logic
- Integration tests with sample dependency graphs
- Manual testing with various terminal sizes

### 1.4 Enhanced `bazbom fix --interactive`

**Goal:** Make remediation workflow intuitive and safe

**Implementation:**

```rust
// crates/bazbom/src/commands/fix.rs
use dialoguer::{Confirm, Select};

pub struct FixCommand {
    interactive: bool,
    apply: bool,
    pr: bool,
    batch: bool,
    safe_only: bool,
}

impl FixCommand {
    pub async fn execute(&self) -> Result<()> {
        let findings = self.load_findings()?;
        let fixable = self.filter_fixable(&findings)?;
        
        if self.interactive {
            self.interactive_remediation(fixable).await?;
        } else if self.batch {
            self.batch_remediation(fixable, self.safe_only).await?;
        } else if self.apply {
            self.auto_apply(fixable).await?;
        } else {
            self.suggest_only(fixable)?;
        }
        
        Ok(())
    }
    
    async fn interactive_remediation(&self, fixable: Vec<Finding>) -> Result<()> {
        println!("Found {} fixable vulnerabilities. Let's fix them together!\n", 
            fixable.len());
        
        let mut fixed_count = 0;
        let mut skipped_count = 0;
        
        for (i, finding) in fixable.iter().enumerate() {
            println!("\n{}/{}: {} ({})", 
                i + 1, 
                fixable.len(), 
                finding.cve_id, 
                finding.package);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            self.display_finding_details(finding)?;
            
            let actions = vec![
                "Apply fix and test",
                "Apply without testing (faster)",
                "Skip for now",
                "View details",
            ];
            
            let selection = Select::new()
                .with_prompt("What would you like to do?")
                .items(&actions)
                .default(0)
                .interact()?;
            
            match selection {
                0 => {
                    self.apply_fix_with_testing(finding).await?;
                    fixed_count += 1;
                    
                    if i < fixable.len() - 1 {
                        let continue_fixing = Confirm::new()
                            .with_prompt(format!(
                                "Fixed {}/{} vulnerabilities. Continue?", 
                                fixed_count, 
                                fixable.len()
                            ))
                            .default(true)
                            .interact()?;
                        
                        if !continue_fixing {
                            break;
                        }
                    }
                }
                1 => {
                    self.apply_fix_without_testing(finding).await?;
                    fixed_count += 1;
                }
                2 => {
                    skipped_count += 1;
                    continue;
                }
                3 => {
                    self.view_finding_details(finding)?;
                    // Re-prompt for action
                }
                _ => unreachable!(),
            }
        }
        
        self.display_summary(fixed_count, skipped_count)?;
        
        Ok(())
    }
    
    fn display_finding_details(&self, finding: &Finding) -> Result<()> {
        println!("Severity: {} {} | Priority: {}", 
            self.severity_icon(&finding.severity),
            finding.severity,
            finding.priority);
        println!("Current: {} → Fixed: {}", 
            finding.current_version, 
            finding.fixed_version);
        
        println!("\nWHY THIS MATTERS:");
        if finding.in_kev {
            println!("• In CISA KEV (actively exploited in the wild)");
        }
        println!("• CVSS Score: {} ({})", 
            finding.cvss_score, 
            finding.severity);
        if let Some(epss) = finding.epss_score {
            println!("• EPSS: {:.1}% (exploitation probability)", epss * 100.0);
        }
        println!("• Impact: {}", finding.impact);
        
        if !finding.breaking_changes.is_empty() {
            println!("\n  BREAKING CHANGES:");
            for change in &finding.breaking_changes {
                println!("  • {}", change);
            }
        } else {
            println!("\nBREAKING CHANGES: None");
        }
        
        println!("COMPATIBILITY: {} Compatible with your {} version", 
            if finding.compatible { "" } else { "" },
            finding.framework);
        
        Ok(())
    }
    
    async fn apply_fix_with_testing(&self, finding: &Finding) -> Result<()> {
        print!("Applying fix... ");
        io::stdout().flush()?;
        
        self.update_dependency(finding).await?;
        
        println!("");
        
        print!("Running tests... ");
        io::stdout().flush()?;
        
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        
        let test_result = self.run_tests().await?;
        
        pb.finish_and_clear();
        
        if test_result.passed {
            println!(" Tests passed! ({:.1}s)", test_result.duration.as_secs_f64());
        } else {
            println!(" Tests failed!");
            println!("\nFailed tests:");
            for failure in &test_result.failures {
                println!("  • {}", failure);
            }
            
            let rollback = Confirm::new()
                .with_prompt("Rollback changes?")
                .default(true)
                .interact()?;
            
            if rollback {
                self.rollback_changes(finding).await?;
                println!(" Changes rolled back");
                return Err(anyhow!("Tests failed, changes rolled back"));
            }
        }
        
        Ok(())
    }
}
```

**Files to Create/Modify:**
- `crates/bazbom/src/commands/fix.rs` (MODIFY - add interactive mode)
- `crates/bazbom/src/remediation/mod.rs` (NEW)
- `crates/bazbom/src/remediation/testing.rs` (NEW - test execution)
- `crates/bazbom/src/remediation/rollback.rs` (NEW - change rollback)

## Phase 2: Visual Excellence (Weeks 3-4)

### 2.1 Web-Based Dashboard

**Goal:** Provide rich visualization without external dependencies

**Implementation:**

```rust
// crates/bazbom/src/commands/dashboard.rs
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

pub struct DashboardCommand {
    port: u16,
    share: bool,
    open_browser: bool,
}

impl DashboardCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.share {
            self.generate_static_html().await?;
        } else {
            self.start_server().await?;
        }
        
        Ok(())
    }
    
    async fn start_server(&self) -> Result<()> {
        let app_state = Arc::new(DashboardState {
            scan_results: self.load_scan_results()?,
            dependency_graph: self.load_dependency_graph()?,
        });
        
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/api/scan", get(scan_handler))
            .route("/api/graph", get(graph_handler))
            .nest_service("/static", ServeDir::new("static"))
            .with_state(app_state);
        
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        
        println!(" BazBOM Dashboard starting...");
        println!("   URL: http://localhost:{}", self.port);
        
        if self.open_browser {
            let url = format!("http://localhost:{}", self.port);
            if let Err(e) = open::that(&url) {
                eprintln!("Failed to open browser: {}", e);
            }
        }
        
        println!("\nPress Ctrl+C to stop");
        
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
        
        Ok(())
    }
    
    async fn generate_static_html(&self) -> Result<()> {
        println!("Generating shareable dashboard...");
        
        let html = self.render_dashboard_html()?;
        let output_path = PathBuf::from("bazbom-dashboard.html");
        
        fs::write(&output_path, html)?;
        
        println!(" Dashboard saved to: {}", output_path.display());
        println!("\nShare this file:");
        println!("  • Open in browser: file://{}", output_path.canonicalize()?.display());
        println!("  • Send via email");
        println!("  • Upload to file sharing service");
        
        Ok(())
    }
    
    fn render_dashboard_html(&self) -> Result<String> {
        let scan_results = self.load_scan_results()?;
        let dependency_graph = self.load_dependency_graph()?;
        
        let template = include_str!("../templates/dashboard.html");
        
        // Embed data as JSON in HTML
        let html = template
            .replace("{{SCAN_DATA}}", &serde_json::to_string(&scan_results)?)
            .replace("{{GRAPH_DATA}}", &serde_json::to_string(&dependency_graph)?);
        
        Ok(html)
    }
}

// Frontend: crates/bazbom/templates/dashboard.html
// React + D3.js visualization embedded in single HTML file
```

**Dependencies:**
- `axum = "0.7"` - Web framework
- `tower-http = "0.5"` - HTTP utilities
- `open = "5.0"` - Open browser
- React + D3.js (embedded in HTML template)

**Frontend Implementation:**
- Single-page React application
- D3.js for graph visualization
- All assets inlined (CSS, JS)
- No build step required
- Works offline

**Files to Create:**
- `crates/bazbom/src/commands/dashboard.rs` (NEW)
- `crates/bazbom/templates/dashboard.html` (NEW - React app)
- `crates/bazbom/static/` (NEW DIR - CSS/JS assets)

## Testing Strategy

### Unit Tests
- Test each command independently
- Mock external dependencies (filesystem, network)
- Verify error handling
- Test edge cases

### Integration Tests
```rust
#[test]
fn test_init_command_full_flow() {
    let temp_dir = TempDir::new().unwrap();
    
    // Create sample project
    create_sample_maven_project(&temp_dir);
    
    // Run init command
    let result = InitCommand {
        interactive: false,
        skip_scan: false,
    }.execute();
    
    assert!(result.is_ok());
    assert!(temp_dir.path().join("bazbom.yml").exists());
}

#[test]
fn test_policy_template_init() {
    let temp_dir = TempDir::new().unwrap();
    
    let result = PolicyInitCommand {
        list: false,
        template: Some("pci-dss".to_string()),
        output: temp_dir.path().to_path_buf(),
    }.execute();
    
    assert!(result.is_ok());
    
    let config = fs::read_to_string(temp_dir.path().join("bazbom.yml")).unwrap();
    assert!(config.contains("PCI-DSS"));
}
```

### Manual Testing Checklist
- [ ] Install on fresh system
- [ ] Run `bazbom init` on Maven project
- [ ] Run `bazbom init` on Gradle project
- [ ] Run `bazbom init` on Bazel project
- [ ] Test policy template discovery
- [ ] Test interactive graph navigation
- [ ] Test fix workflow with real vulnerabilities
- [ ] Test dashboard generation
- [ ] Verify all error messages are user-friendly

## Documentation Updates

### Files to Update
- [ ] `docs/USAGE.md` - Add new commands
- [ ] `docs/QUICKSTART.md` - Create guided walkthrough
- [ ] `README.md` - Update quickstart section
- [ ] `CHANGELOG.md` - Document new features

### New Documentation
- [ ] `docs/guides/ONBOARDING.md` - Complete onboarding guide
- [ ] `docs/guides/POLICY_TEMPLATES.md` - Policy template reference
- [ ] `docs/guides/INTERACTIVE_GRAPH.md` - Graph navigation guide
- [ ] `docs/guides/REMEDIATION_WORKFLOW.md` - Fix workflow guide

## Success Criteria

### Phase 1 (Weeks 1-2)
- [ ] `bazbom init` reduces setup time to < 5 minutes
- [ ] Policy templates are easily discoverable
- [ ] Interactive graph provides better UX than external tools
- [ ] Fix workflow is intuitive and safe
- [ ] All commands have helpful error messages
- [ ] Documentation is clear and complete

### Phase 2 (Weeks 3-4)
- [ ] Dashboard provides rich visualization
- [ ] Dashboard works offline (static HTML)
- [ ] Dashboard is shareable
- [ ] Graph visualization is interactive and responsive
- [ ] Performance is acceptable (< 3s to render 1000 nodes)

## Timeline

### Week 1
- Day 1-2: Implement `bazbom init` command
- Day 3-4: Implement policy template system
- Day 5: Testing and bug fixes

### Week 2
- Day 1-2: Implement interactive graph (terminal)
- Day 3-4: Enhance fix command with interactive mode
- Day 5: Testing and documentation

### Week 3
- Day 1-2: Set up web framework and routing
- Day 3-4: Implement dashboard backend
- Day 5: Testing and bug fixes

### Week 4
- Day 1-2: Build dashboard frontend (React + D3.js)
- Day 3: Implement static HTML export
- Day 4: Integration testing
- Day 5: Documentation and polish

## Risk Mitigation

### Technical Risks
- **TUI complexity:** Use well-tested libraries (crossterm, tui-rs)
- **Performance:** Profile and optimize graph rendering
- **Browser compatibility:** Test dashboard on major browsers
- **Terminal compatibility:** Test on Windows, macOS, Linux

### User Experience Risks
- **Confusion:** Extensive user testing with developers
- **Discoverability:** Clear help text and documentation
- **Errors:** Friendly error messages with suggestions

## Next Steps

After Phase 1-2 completion:
1. Gather user feedback
2. Measure adoption metrics
3. Iterate based on feedback
4. Plan Phase 3 (IDE polish)
5. Plan Phase 4 (team features)

## Conclusion

These improvements will make BazBOM significantly easier to use than commercial alternatives while maintaining its core advantages (free, privacy-preserving, Bazel support). The focus on quick wins ensures users see immediate value, while the roadmap provides a path to comprehensive feature parity.
