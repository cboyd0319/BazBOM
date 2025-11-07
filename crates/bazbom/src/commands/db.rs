use anyhow::{Context, Result};
use std::path::PathBuf;
use bazbom::cli::DbCmd;

/// Handle the `bazbom db` command
pub fn handle_db(action: DbCmd) -> Result<()> {
    match action {
        DbCmd::Sync {} => {
            println!("[bazbom] db sync");
            let cache_dir = PathBuf::from(".bazbom/cache");
            let offline = std::env::var("BAZBOM_OFFLINE").is_ok();
            let manifest = bazbom_advisories::db_sync(&cache_dir, offline)
                .context("failed advisory DB sync")?;
            println!(
                "[bazbom] advisories cached at {:?} ({} files)",
                cache_dir,
                manifest.files.len()
            );
            Ok(())
        }
    }
}
