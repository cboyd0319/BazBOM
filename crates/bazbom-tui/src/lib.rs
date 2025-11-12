//! Terminal User Interface for interactive dependency exploration
//!
//! This module provides an interactive TUI for exploring dependency graphs,
//! filtering vulnerabilities, and viewing detailed information about dependencies.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io;

/// Create a clickable hyperlink using OSC 8 escape sequences
///
/// This works in terminals that support OSC 8:
/// - iTerm2 (macOS)
/// - kitty
/// - Windows Terminal
/// - GNOME Terminal
/// - Alacritty
/// - WezTerm
///
/// Format: \x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\
fn make_clickable_link(text: &str, url: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}

/// Generate NVD URL for a CVE ID
fn cve_to_nvd_url(cve_id: &str) -> String {
    format!("https://nvd.nist.gov/vuln/detail/{}", cve_id)
}

/// Search mode for filtering dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Simple substring search (case-insensitive)
    Substring,
    /// Regular expression search
    Regex,
    /// Glob pattern search
    Glob,
}

impl SearchMode {
    pub fn as_str(&self) -> &str {
        match self {
            SearchMode::Substring => "Substring",
            SearchMode::Regex => "Regex",
            SearchMode::Glob => "Glob",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SearchMode::Substring => SearchMode::Regex,
            SearchMode::Regex => SearchMode::Glob,
            SearchMode::Glob => SearchMode::Substring,
        }
    }
}

/// View mode for TUI display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// List view (default)
    List,
    /// Graph view with ASCII dependency tree
    Graph,
}

impl ViewMode {
    pub fn as_str(&self) -> &str {
        match self {
            ViewMode::List => "List",
            ViewMode::Graph => "Graph",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            ViewMode::List => ViewMode::Graph,
            ViewMode::Graph => ViewMode::List,
        }
    }
}

/// Main TUI application state
pub struct App {
    /// List of dependencies to display
    dependencies: Vec<Dependency>,
    /// Current selection state
    list_state: ListState,
    /// Search/filter query
    search_query: String,
    /// Search mode (substring, regex, glob)
    search_mode: SearchMode,
    /// Whether search is case-insensitive (for regex/glob)
    case_insensitive: bool,
    /// Filter by severity
    severity_filter: Option<String>,
    /// Whether to show help screen
    show_help: bool,
    /// Export message to display
    export_message: Option<String>,
    /// Current view mode (list or graph)
    view_mode: ViewMode,
}

/// Simplified dependency representation for TUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub scope: String,
    pub vulnerabilities: Vec<Vulnerability>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve: String,
    pub severity: String,
    pub cvss: f32,
    pub fixed_version: Option<String>,
}

impl App {
    /// Create a new TUI app with dependencies
    pub fn new(dependencies: Vec<Dependency>) -> Self {
        let mut list_state = ListState::default();
        if !dependencies.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            dependencies,
            list_state,
            search_query: String::new(),
            search_mode: SearchMode::Substring,
            case_insensitive: true,
            severity_filter: None,
            show_help: false,
            export_message: None,
            view_mode: ViewMode::List,
        }
    }

    /// Get filtered dependencies based on current search and filters
    fn filtered_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|dep| {
                // Search filter
                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    match self.search_mode {
                        SearchMode::Substring => {
                            // Simple substring search (case-insensitive)
                            dep.name
                                .to_lowercase()
                                .contains(&self.search_query.to_lowercase())
                        }
                        SearchMode::Regex => {
                            // Regex search
                            let pattern = if self.case_insensitive {
                                format!("(?i){}", self.search_query)
                            } else {
                                self.search_query.clone()
                            };
                            Regex::new(&pattern)
                                .ok()
                                .and_then(|re| re.is_match(&dep.name).then_some(true))
                                .unwrap_or(false)
                        }
                        SearchMode::Glob => {
                            // Glob pattern search
                            // Convert glob to regex: * -> .*, ? -> .
                            let mut pattern = self.search_query
                                .replace(".", r"\.")
                                .replace("*", ".*")
                                .replace("?", ".");

                            if self.case_insensitive {
                                pattern = format!("(?i)^{}$", pattern);
                            } else {
                                pattern = format!("^{}$", pattern);
                            }

                            Regex::new(&pattern)
                                .ok()
                                .and_then(|re| re.is_match(&dep.name).then_some(true))
                                .unwrap_or(false)
                        }
                    }
                };

                // Severity filter
                let matches_severity = if let Some(ref severity) = self.severity_filter {
                    dep.vulnerabilities.iter().any(|v| v.severity == *severity)
                } else {
                    true
                };

                matches_search && matches_severity
            })
            .collect()
    }

    /// Move selection down
    fn next(&mut self) {
        let filtered = self.filtered_dependencies();
        if filtered.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= filtered.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Move selection up
    fn previous(&mut self) {
        let filtered = self.filtered_dependencies();
        if filtered.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Get currently selected dependency
    fn selected_dependency(&self) -> Option<&Dependency> {
        let filtered = self.filtered_dependencies();
        self.list_state
            .selected()
            .and_then(|i| filtered.get(i))
            .copied()
    }

    /// Toggle help screen
    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Export filtered dependencies to JSON file
    fn export_to_json(&mut self, filename: &str) -> Result<()> {
        let filtered = self.filtered_dependencies();
        let data: Vec<&Dependency> = filtered.into_iter().collect();

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(filename, json)?;

        self.export_message = Some(format!(
            "Exported {} dependencies to {}",
            data.len(),
            filename
        ));
        Ok(())
    }

    /// Clear export message
    fn clear_export_message(&mut self) {
        self.export_message = None;
    }
}

/// Run the TUI application
pub fn run(dependencies: Vec<Dependency>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(dependencies);

    // Run app loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

/// Main application loop
fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if handle_key_event(key, app)? {
                return Ok(());
            }
        }
    }
}

/// Handle keyboard events
fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<bool> {
    // Clear export message on any key if it's showing
    if app.export_message.is_some() {
        app.clear_export_message();
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Char('?') | KeyCode::F(1) => app.toggle_help(),
        KeyCode::Char('c') => app.severity_filter = Some("CRITICAL".to_string()),
        KeyCode::Char('h') => app.severity_filter = Some("HIGH".to_string()),
        KeyCode::Char('m') => app.severity_filter = Some("MEDIUM".to_string()),
        KeyCode::Char('l') => app.severity_filter = Some("LOW".to_string()),
        KeyCode::Char('a') => app.severity_filter = None,
        KeyCode::Char('e') => {
            // Export filtered dependencies to JSON
            let filename = "bazbom_filtered_deps.json";
            if let Err(e) = app.export_to_json(filename) {
                app.export_message = Some(format!("Export failed: {}", e));
            }
        }
        KeyCode::Char('x') => {
            // Export all dependencies to JSON
            let all_deps: Vec<&Dependency> = app.dependencies.iter().collect();
            let json = serde_json::to_string_pretty(&all_deps)?;
            std::fs::write("bazbom_all_deps.json", json)?;
            app.export_message = Some(format!(
                "Exported {} dependencies to bazbom_all_deps.json",
                all_deps.len()
            ));
        }
        KeyCode::Char('r') => {
            // Toggle search mode: Substring -> Regex -> Glob -> Substring
            app.search_mode = app.search_mode.next();
            app.export_message = Some(format!("Search mode: {}", app.search_mode.as_str()));
        }
        KeyCode::Char('i') => {
            // Toggle case sensitivity
            app.case_insensitive = !app.case_insensitive;
            let status = if app.case_insensitive { "ON" } else { "OFF" };
            app.export_message = Some(format!("Case-insensitive search: {}", status));
        }
        KeyCode::Char('g') => {
            // Toggle view mode: List <-> Graph
            app.view_mode = app.view_mode.toggle();
            app.export_message = Some(format!("View mode: {}", app.view_mode.as_str()));
        }
        _ => {}
    }
    Ok(false)
}

/// Render the UI
fn ui(f: &mut Frame, app: &mut App) {
    if app.show_help {
        render_help(f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    render_header(f, chunks[0], app);

    // Main content - switch between list and graph views
    match app.view_mode {
        ViewMode::List => {
            // Split into list and details
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            render_dependency_list(f, content_chunks[0], app);
            render_details(f, content_chunks[1], app);
        }
        ViewMode::Graph => {
            // Full-width graph view
            render_graph(f, chunks[1], app);
        }
    }

    // Footer
    render_footer(f, chunks[2], app);
}

/// Render header with title and filter info
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let filter_text = if let Some(ref severity) = app.severity_filter {
        format!("Filter: {}", severity)
    } else {
        "Filter: ALL".to_string()
    };

    let case_text = if app.case_insensitive { "i" } else { "" };
    let search_mode_text = format!("Search: {}{}", app.search_mode.as_str(), case_text);

    let header = Paragraph::new(format!(
        "BazBOM Dependency Explorer - {} dependencies | {} | {}",
        app.dependencies.len(),
        filter_text,
        search_mode_text
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));

    f.render_widget(header, area);
}

/// Render dependency list
fn render_dependency_list(f: &mut Frame, area: Rect, app: &mut App) {
    let filtered = app.filtered_dependencies();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|dep| {
            let vuln_count = dep.vulnerabilities.len();
            let critical_count = dep
                .vulnerabilities
                .iter()
                .filter(|v| v.severity == "CRITICAL")
                .count();
            let high_count = dep
                .vulnerabilities
                .iter()
                .filter(|v| v.severity == "HIGH")
                .count();

            let (icon, color) = if critical_count > 0 {
                ("X", Color::Red)
            } else if high_count > 0 {
                ("!", Color::Yellow)
            } else if vuln_count > 0 {
                ("i", Color::Blue)
            } else {
                ("+", Color::Green)
            };

            let line = if vuln_count > 0 {
                Line::from(vec![
                    Span::raw(format!("[{}] ", icon)),
                    Span::styled(
                        format!("{}:{}", dep.name, dep.version),
                        Style::default().fg(color),
                    ),
                    Span::raw(format!(" ({} vulns)", vuln_count)),
                ])
            } else {
                Line::from(vec![
                    Span::raw(format!("[{}] ", icon)),
                    Span::raw(format!("{}:{}", dep.name, dep.version)),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Dependencies"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

/// Render details panel for selected dependency
fn render_details(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(dep) = app.selected_dependency() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&dep.name),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&dep.version),
            ]),
            Line::from(vec![
                Span::styled("Scope: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&dep.scope),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("Vulnerabilities: {}", dep.vulnerabilities.len()),
                Style::default().add_modifier(Modifier::BOLD),
            )]),
        ];

        for vuln in &dep.vulnerabilities {
            let color = match vuln.severity.as_str() {
                "CRITICAL" => Color::Red,
                "HIGH" => Color::Yellow,
                "MEDIUM" => Color::Blue,
                "LOW" => Color::Green,
                _ => Color::White,
            };

            lines.push(Line::from(""));

            // Create clickable CVE link using OSC 8
            let cve_url = cve_to_nvd_url(&vuln.cve);
            let clickable_cve = make_clickable_link(&format!("  {} ", vuln.cve), &cve_url);

            lines.push(Line::from(vec![
                Span::styled(
                    clickable_cve,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("(CVSS {})", vuln.cvss)),
            ]));

            if let Some(ref fixed) = vuln.fixed_version {
                lines.push(Line::from(vec![
                    Span::raw("    Fix: Upgrade to "),
                    Span::styled(
                        fixed,
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }

        lines
    } else {
        vec![Line::from("No dependency selected")]
    };

    let paragraph =
        Paragraph::new(content).block(Block::default().borders(Borders::ALL).title("Details"));

    f.render_widget(paragraph, area);
}

/// Render footer with keyboard shortcuts
fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if let Some(ref msg) = app.export_message {
        msg.clone()
    } else {
        "[↑↓/jk] Navigate [c/h/m/l/a] Filter [g] Graph View [r] Search mode [i] Case [e] Export [?] Help [q] Quit"
            .to_string()
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(footer, area);
}

/// Render dependency graph in ASCII tree format
fn render_graph(f: &mut Frame, area: Rect, app: &App) {
    let filtered_deps = app.filtered_dependencies();

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "Dependency Graph (Tree View)",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if filtered_deps.is_empty() {
        lines.push(Line::from("No dependencies to display"));
    } else {
        // Build a simple tree structure grouped by scope
        use std::collections::HashMap;
        let mut scopes: HashMap<String, Vec<&Dependency>> = HashMap::new();

        for dep in &filtered_deps {
            scopes.entry(dep.scope.clone()).or_default().push(dep);
        }

        let mut scope_names: Vec<_> = scopes.keys().cloned().collect();
        scope_names.sort();

        for (scope_idx, scope) in scope_names.iter().enumerate() {
            let is_last_scope = scope_idx == scope_names.len() - 1;
            let scope_prefix = if is_last_scope { "└─" } else { "├─" };

            // Scope header
            lines.push(Line::from(vec![
                Span::raw(scope_prefix),
                Span::styled(
                    format!(" {} ({} deps)", scope, scopes[scope].len()),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));

            let deps = &scopes[scope];
            for (dep_idx, dep) in deps.iter().enumerate() {
                let is_last_dep = dep_idx == deps.len() - 1;
                let scope_connector = if is_last_scope { "  " } else { "│ " };
                let dep_prefix = if is_last_dep { "└─" } else { "├─" };

                // Dependency name and version
                let vuln_indicator = if !dep.vulnerabilities.is_empty() {
                    let critical = dep.vulnerabilities.iter().filter(|v| v.severity == "CRITICAL").count();
                    let high = dep.vulnerabilities.iter().filter(|v| v.severity == "HIGH").count();

                    if critical > 0 {
                        format!(" [🔴 {} CRITICAL]", critical)
                    } else if high > 0 {
                        format!(" [🟠 {} HIGH]", high)
                    } else {
                        format!(" [⚠️  {} vulns]", dep.vulnerabilities.len())
                    }
                } else {
                    String::new()
                };

                lines.push(Line::from(vec![
                    Span::raw(format!("{}  {}", scope_connector, dep_prefix)),
                    Span::styled(
                        &dep.name,
                        if !dep.vulnerabilities.is_empty() {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Green)
                        },
                    ),
                    Span::styled(
                        format!(" @ {}", dep.version),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        vuln_indicator,
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]));

                // Show vulnerability details if present
                if !dep.vulnerabilities.is_empty() {
                    let vuln_connector = if is_last_dep { "   " } else { "│  " };
                    for (vuln_idx, vuln) in dep.vulnerabilities.iter().take(3).enumerate() {
                        let is_last_vuln = vuln_idx == dep.vulnerabilities.len().min(3) - 1;
                        let vuln_prefix = if is_last_vuln { "└─" } else { "├─" };

                        let severity_color = match vuln.severity.as_str() {
                            "CRITICAL" => Color::Red,
                            "HIGH" => Color::LightRed,
                            "MEDIUM" => Color::Yellow,
                            _ => Color::Gray,
                        };

                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  {}   {}", scope_connector, vuln_connector, vuln_prefix)),
                            Span::styled(
                                &vuln.cve,
                                Style::default().fg(severity_color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!(" ({}, CVSS: {:.1})", vuln.severity, vuln.cvss),
                                Style::default().fg(Color::Gray),
                            ),
                        ]));
                    }

                    if dep.vulnerabilities.len() > 3 {
                        lines.push(Line::from(vec![
                            Span::raw(format!("{}  {}   ", scope_connector, if is_last_dep { " " } else { "│" })),
                            Span::styled(
                                format!("... {} more vulnerabilities", dep.vulnerabilities.len() - 3),
                                Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
                            ),
                        ]));
                    }
                }
            }

            if !is_last_scope {
                lines.push(Line::from(""));
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Dependency Graph"));

    f.render_widget(paragraph, area);
}

/// Render help screen
fn render_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "BazBOM Dependency Explorer - Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Up/k       Move selection up"),
        Line::from("  Down/j     Move selection down"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Filtering:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  c          Show only CRITICAL vulnerabilities"),
        Line::from("  h          Show only HIGH vulnerabilities"),
        Line::from("  m          Show only MEDIUM vulnerabilities"),
        Line::from("  l          Show only LOW vulnerabilities"),
        Line::from("  a          Show ALL dependencies (clear filter)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  r          Toggle search mode (Substring/Regex/Glob)"),
        Line::from("  i          Toggle case-insensitive search"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Display:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  g          Toggle Graph View (List ↔ Tree)"),
        Line::from("  Left       Dependency list (List mode)"),
        Line::from("  Right      Vulnerability details (List mode)"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Export:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  e          Export filtered dependencies to JSON"),
        Line::from("  x          Export all dependencies to JSON"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?/F1       Toggle this help screen"),
        Line::from("  q/Esc      Quit application"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Symbols:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  [X] ", Style::default().fg(Color::Red)),
            Span::raw("Critical vulnerabilities"),
        ]),
        Line::from(vec![
            Span::styled("  [!] ", Style::default().fg(Color::Yellow)),
            Span::raw("High vulnerabilities"),
        ]),
        Line::from(vec![
            Span::styled("  [i] ", Style::default().fg(Color::Blue)),
            Span::raw("Medium/Low vulnerabilities"),
        ]),
        Line::from(vec![
            Span::styled("  [+] ", Style::default().fg(Color::Green)),
            Span::raw("No vulnerabilities"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press any key to return...",
            Style::default().fg(Color::Cyan),
        )]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, f.area());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let deps = vec![Dependency {
            name: "test-dep".to_string(),
            version: "1.0.0".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![],
        }];

        let app = App::new(deps);
        assert_eq!(app.dependencies.len(), 1);
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_navigation() {
        let deps = vec![
            Dependency {
                name: "dep1".to_string(),
                version: "1.0.0".to_string(),
                scope: "compile".to_string(),
                vulnerabilities: vec![],
            },
            Dependency {
                name: "dep2".to_string(),
                version: "2.0.0".to_string(),
                scope: "compile".to_string(),
                vulnerabilities: vec![],
            },
        ];

        let mut app = App::new(deps);
        assert_eq!(app.list_state.selected(), Some(0));

        app.next();
        assert_eq!(app.list_state.selected(), Some(1));

        app.next();
        assert_eq!(app.list_state.selected(), Some(0));

        app.previous();
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn test_filtering() {
        let deps = vec![
            Dependency {
                name: "vulnerable-dep".to_string(),
                version: "1.0.0".to_string(),
                scope: "compile".to_string(),
                vulnerabilities: vec![Vulnerability {
                    cve: "CVE-2024-1234".to_string(),
                    severity: "CRITICAL".to_string(),
                    cvss: 9.8,
                    fixed_version: Some("2.0.0".to_string()),
                }],
            },
            Dependency {
                name: "safe-dep".to_string(),
                version: "1.0.0".to_string(),
                scope: "compile".to_string(),
                vulnerabilities: vec![],
            },
        ];

        let mut app = App::new(deps);

        // No filter - should show both
        assert_eq!(app.filtered_dependencies().len(), 2);

        // Filter by CRITICAL
        app.severity_filter = Some("CRITICAL".to_string());
        assert_eq!(app.filtered_dependencies().len(), 1);

        // Filter by HIGH (none exist)
        app.severity_filter = Some("HIGH".to_string());
        assert_eq!(app.filtered_dependencies().len(), 0);
    }
}
