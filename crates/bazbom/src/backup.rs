// Backup and rollback functionality for safe automated remediation
// Creates backups before applying fixes and restores on test failures

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Backup strategy to use
#[derive(Debug, Clone, Copy)]
pub enum BackupStrategy {
    /// Use git stash (requires clean working tree)
    GitStash,
    /// Copy files to backup directory
    FileCopy,
    /// Use git branch
    GitBranch,
}

/// Handle for managing backups
pub struct BackupHandle {
    strategy: BackupStrategy,
    backup_dir: Option<PathBuf>,
    git_stash_id: Option<String>,
    git_branch: Option<String>,
    project_root: PathBuf,
}

impl BackupHandle {
    /// Create a new backup of the project
    pub fn create(project_root: &Path, strategy: BackupStrategy) -> Result<Self> {
        match strategy {
            BackupStrategy::GitStash => Self::create_git_stash(project_root),
            BackupStrategy::FileCopy => Self::create_file_copy(project_root),
            BackupStrategy::GitBranch => Self::create_git_branch(project_root),
        }
    }

    fn create_git_stash(project_root: &Path) -> Result<Self> {
        println!("[bazbom] Creating git stash backup...");

        // Check if git is available and we're in a repo
        let is_git_repo = project_root.join(".git").exists();
        if !is_git_repo {
            anyhow::bail!("Not a git repository, cannot use git stash");
        }

        // Create stash with message
        let output = Command::new("git")
            .args([
                "stash",
                "push",
                "-m",
                "bazbom-backup",
                "--include-untracked",
            ])
            .current_dir(project_root)
            .output()
            .context("Failed to create git stash")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git stash failed: {}", err);
        }

        // Get stash ID
        let list_output = Command::new("git")
            .args(["stash", "list", "-n", "1"])
            .current_dir(project_root)
            .output()
            .context("Failed to list git stash")?;

        let stash_id = String::from_utf8_lossy(&list_output.stdout)
            .lines()
            .next()
            .unwrap_or("stash@{0}")
            .split(':')
            .next()
            .unwrap_or("stash@{0}")
            .to_string();

        println!("[bazbom] Backup created: {}", stash_id);

        Ok(Self {
            strategy: BackupStrategy::GitStash,
            backup_dir: None,
            git_stash_id: Some(stash_id),
            git_branch: None,
            project_root: project_root.to_path_buf(),
        })
    }

    fn create_file_copy(project_root: &Path) -> Result<Self> {
        println!("[bazbom] Creating file copy backup...");

        // Create backup directory
        let backup_dir = project_root.join(".bazbom/backup");
        fs::create_dir_all(&backup_dir).context("Failed to create backup directory")?;

        // Copy critical files
        let files_to_backup = vec![
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "MODULE.bazel",
            "WORKSPACE",
            "maven_install.json",
        ];

        for file in files_to_backup {
            let src = project_root.join(file);
            if src.exists() {
                let dst = backup_dir.join(file);
                fs::copy(&src, &dst).with_context(|| format!("Failed to backup {}", file))?;
                println!("[bazbom]   Backed up: {}", file);
            }
        }

        Ok(Self {
            strategy: BackupStrategy::FileCopy,
            backup_dir: Some(backup_dir),
            git_stash_id: None,
            git_branch: None,
            project_root: project_root.to_path_buf(),
        })
    }

    fn create_git_branch(project_root: &Path) -> Result<Self> {
        println!("[bazbom] Creating git branch backup...");

        let is_git_repo = project_root.join(".git").exists();
        if !is_git_repo {
            anyhow::bail!("Not a git repository, cannot use git branch");
        }

        // Create a temporary branch
        let branch_name = format!("bazbom-backup-{}", chrono::Utc::now().timestamp());

        let output = Command::new("git")
            .args(["branch", &branch_name])
            .current_dir(project_root)
            .output()
            .context("Failed to create git branch")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git branch creation failed: {}", err);
        }

        println!("[bazbom] Backup branch created: {}", branch_name);

        Ok(Self {
            strategy: BackupStrategy::GitBranch,
            backup_dir: None,
            git_stash_id: None,
            git_branch: Some(branch_name),
            project_root: project_root.to_path_buf(),
        })
    }

    /// Restore from backup
    pub fn restore(&self) -> Result<()> {
        match self.strategy {
            BackupStrategy::GitStash => self.restore_git_stash(),
            BackupStrategy::FileCopy => self.restore_file_copy(),
            BackupStrategy::GitBranch => self.restore_git_branch(),
        }
    }

    fn restore_git_stash(&self) -> Result<()> {
        println!("[bazbom] Restoring from git stash...");

        let stash_id = self
            .git_stash_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No stash ID found"))?;

        // Reset working directory
        Command::new("git")
            .args(["reset", "--hard", "HEAD"])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to reset working directory")?;

        // Apply stash
        let output = Command::new("git")
            .args(["stash", "apply", stash_id])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to apply git stash")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git stash apply failed: {}", err);
        }

        println!("[bazbom] Restored from: {}", stash_id);
        Ok(())
    }

    fn restore_file_copy(&self) -> Result<()> {
        println!("[bazbom] Restoring from file copy...");

        let backup_dir = self
            .backup_dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No backup directory found"))?;

        // Restore backed up files
        for entry in fs::read_dir(backup_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let src = entry.path();
            let dst = self.project_root.join(&file_name);

            fs::copy(&src, &dst).with_context(|| format!("Failed to restore {:?}", file_name))?;
            println!("[bazbom]   Restored: {:?}", file_name);
        }

        Ok(())
    }

    fn restore_git_branch(&self) -> Result<()> {
        println!("[bazbom] Restoring from git branch...");

        let branch_name = self
            .git_branch
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No branch name found"))?;

        // Reset to branch
        let output = Command::new("git")
            .args(["reset", "--hard", branch_name])
            .current_dir(&self.project_root)
            .output()
            .context("Failed to reset to branch")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git reset failed: {}", err);
        }

        println!("[bazbom] Restored from branch: {}", branch_name);
        Ok(())
    }

    /// Clean up backup (call after successful apply)
    pub fn cleanup(&self) -> Result<()> {
        match self.strategy {
            BackupStrategy::GitStash => {
                // Drop the stash
                if let Some(stash_id) = &self.git_stash_id {
                    Command::new("git")
                        .args(["stash", "drop", stash_id])
                        .current_dir(&self.project_root)
                        .output()
                        .context("Failed to drop git stash")?;
                }
            }
            BackupStrategy::FileCopy => {
                // Remove backup directory
                if let Some(backup_dir) = &self.backup_dir {
                    fs::remove_dir_all(backup_dir).context("Failed to remove backup directory")?;
                }
            }
            BackupStrategy::GitBranch => {
                // Delete the backup branch
                if let Some(branch_name) = &self.git_branch {
                    Command::new("git")
                        .args(["branch", "-D", branch_name])
                        .current_dir(&self.project_root)
                        .output()
                        .context("Failed to delete backup branch")?;
                }
            }
        }

        println!("[bazbom] Backup cleaned up");
        Ok(())
    }
}

/// Determine the best backup strategy for the project
pub fn choose_backup_strategy(project_root: &Path) -> BackupStrategy {
    if project_root.join(".git").exists() {
        // Check if working tree is clean
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(project_root)
            .output();

        if let Ok(output) = status_output {
            let is_clean = output.stdout.is_empty();
            if is_clean {
                return BackupStrategy::GitBranch;
            }
        }

        // If not clean, use stash
        BackupStrategy::GitStash
    } else {
        // Not a git repo, use file copy
        BackupStrategy::FileCopy
    }
}
