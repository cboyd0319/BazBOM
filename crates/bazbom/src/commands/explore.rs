use anyhow::Result;

/// Handle the `bazbom explore` command
pub fn handle_explore(sbom: Option<String>, findings: Option<String>) -> Result<()> {
    use bazbom::explore;

    // Load dependencies from SBOM/findings or use mock data
    let dependencies = explore::load_dependencies(sbom.as_deref(), findings.as_deref())?;

    if sbom.is_some() || findings.is_some() {
        println!("[bazbom] Loaded {} dependencies", dependencies.len());
    } else {
        println!("[bazbom] No SBOM/findings specified, using demo data");
        println!("[bazbom] Hint: Use --sbom=<file> or --findings=<file> to load your data");
    }

    bazbom_tui::run(dependencies)
}
