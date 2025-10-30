use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

pub struct ToolOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_tool(
    bin: &Path,
    args: &[&str],
    cwd: &Path,
    _timeout_secs: u64,
) -> Result<ToolOutput> {
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Minimal env: preserve PATH only for tool execution
    if let Some(path) = std::env::var_os("PATH") {
        cmd.env_clear();
        cmd.env("PATH", path);
    }

    let output = cmd
        .output()
        .with_context(|| format!("failed to execute {:?}", bin))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    Ok(ToolOutput {
        exit_code,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_run_echo_command() -> Result<()> {
        // Use 'echo' which should be available on all platforms
        let echo_path = if cfg!(windows) {
            PathBuf::from("C:\\Windows\\System32\\cmd.exe")
        } else {
            PathBuf::from("/bin/echo")
        };

        if !echo_path.exists() {
            // Skip test if echo doesn't exist at expected location
            return Ok(());
        }

        let args = if cfg!(windows) {
            vec!["/C", "echo", "test"]
        } else {
            vec!["test"]
        };

        let cwd = env::current_dir()?;
        let output = run_tool(&echo_path, &args, &cwd, 5)?;

        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("test"));

        Ok(())
    }
}
