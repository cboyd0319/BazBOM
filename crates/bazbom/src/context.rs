use std::path::PathBuf;

pub struct Context {
    pub workspace: PathBuf,
    pub out_dir: PathBuf,
    pub sbom_dir: PathBuf,
    pub findings_dir: PathBuf,
    pub enrich_dir: PathBuf,
    pub fixes_dir: PathBuf,
    pub tool_cache: PathBuf,
}

impl Context {
    pub fn new(workspace: PathBuf, out_dir: PathBuf) -> anyhow::Result<Self> {
        let sbom_dir = out_dir.join("sbom");
        let findings_dir = out_dir.join("findings");
        let enrich_dir = out_dir.join("enrich");
        let fixes_dir = out_dir.join("fixes");

        // Create output directories
        std::fs::create_dir_all(&sbom_dir)?;
        std::fs::create_dir_all(&findings_dir)?;
        std::fs::create_dir_all(&enrich_dir)?;
        std::fs::create_dir_all(&fixes_dir)?;

        // Use .bazbom/tools for tool cache
        let tool_cache = workspace.join(".bazbom").join("tools");
        std::fs::create_dir_all(&tool_cache)?;

        Ok(Self {
            workspace,
            out_dir,
            sbom_dir,
            findings_dir,
            enrich_dir,
            fixes_dir,
            tool_cache,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_context_creation() -> anyhow::Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");

        let ctx = Context::new(workspace.clone(), out_dir.clone())?;

        assert_eq!(ctx.workspace, workspace);
        assert_eq!(ctx.out_dir, out_dir);
        assert!(ctx.sbom_dir.exists());
        assert!(ctx.findings_dir.exists());
        assert!(ctx.enrich_dir.exists());
        assert!(ctx.fixes_dir.exists());
        assert!(ctx.tool_cache.exists());

        Ok(())
    }
}
