use anyhow::Result;

/// Handle the `bazbom init` command
pub fn handle_init(path: &str) -> Result<()> {
    bazbom::init::run_init(path)
}
