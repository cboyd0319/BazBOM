use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct ToolDescriptor {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub executable: bool,
    pub archive: bool,
    /// Path to the executable within the archive (e.g., "codeql/codeql" or "syft")
    pub executable_path: Option<String>,
}

pub struct ToolCache {
    root: PathBuf,
}

impl ToolCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn ensure(&self, desc: &ToolDescriptor) -> Result<PathBuf> {
        let dir = self.root.join(&desc.name).join(&desc.version);
        let marker = dir.join(".ok");
        let bin = dir.join(self.binary_name(&desc.name));

        // If already cached and verified, return the path
        if marker.exists() && bin.exists() {
            return Ok(bin);
        }

        fs::create_dir_all(&dir).context("create tool cache dir")?;

        println!(
            "[bazbom] downloading {} {} from {}",
            desc.name, desc.version, desc.url
        );

        // Download to a temporary file
        let tmp_file =
            tempfile::NamedTempFile::new_in(&dir).context("create temp file for download")?;
        let tmp_path = tmp_file.path();

        // Download the file
        let resp = ureq::get(&desc.url).call().context("HTTP request failed")?;

        let mut file = fs::File::create(tmp_path).context("create temp file")?;
        let mut hasher = Sha256::new();
        let mut reader = resp.into_reader();
        let mut buffer = [0; 8192];

        loop {
            let n = std::io::Read::read(&mut reader, &mut buffer).context("read response body")?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n]).context("write to temp file")?;
            hasher.update(&buffer[..n]);
        }

        file.sync_all().context("sync temp file")?;
        drop(file);

        // Verify SHA256
        let digest = format!("{:x}", hasher.finalize());
        if digest != desc.sha256 {
            bail!(
                "SHA256 mismatch for {}: expected {}, got {}",
                desc.name,
                desc.sha256,
                digest
            );
        }

        println!("[bazbom] SHA256 verified for {}", desc.name);

        // Handle archives
        let final_bin = if desc.archive {
            let archive_path = tmp_path.to_path_buf();

            // Determine if it's a zip or tar.gz based on the URL
            if desc.url.ends_with(".zip") {
                // Extract ZIP archive
                let archive = fs::File::open(&archive_path).context("open zip archive")?;
                let mut zip = zip::ZipArchive::new(archive).context("read zip archive")?;

                // Extract all files to maintain directory structure
                for i in 0..zip.len() {
                    let mut file = zip.by_index(i).context("read zip entry")?;
                    let outpath = dir.join(file.name());

                    if file.is_dir() {
                        fs::create_dir_all(&outpath).context("create directory")?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            fs::create_dir_all(p).context("create parent directory")?;
                        }
                        let mut outfile = fs::File::create(&outpath).context("create file")?;
                        std::io::copy(&mut file, &mut outfile).context("extract file")?;

                        // Set executable permissions on Unix for the extracted file
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let mut perms = fs::metadata(&outpath)
                                .context("get file metadata")?
                                .permissions();
                            // If the file had executable bit in the zip, preserve it
                            if let Some(mode) = file.unix_mode() {
                                if mode & 0o111 != 0 {
                                    perms.set_mode(mode);
                                    fs::set_permissions(&outpath, perms)
                                        .context("set permissions")?;
                                }
                            }
                        }
                    }
                }
            } else if desc.url.ends_with(".tar.gz") || desc.url.ends_with(".tgz") {
                // Extract tar.gz archive
                use std::process::Command;
                let status = Command::new("tar")
                    .args(&[
                        "-xzf",
                        archive_path.to_str().unwrap(),
                        "-C",
                        dir.to_str().unwrap(),
                    ])
                    .status()
                    .context("run tar extraction")?;

                if !status.success() {
                    bail!("tar extraction failed");
                }
            } else {
                bail!("unsupported archive format: {}", desc.url);
            }

            // Find the executable based on executable_path or tool name
            let exec_path = desc
                .executable_path
                .as_ref()
                .map(|p| dir.join(p))
                .unwrap_or_else(|| dir.join(self.binary_name(&desc.name)));

            if !exec_path.exists() {
                bail!("executable not found at {:?} after extraction", exec_path);
            }

            exec_path
        } else {
            // Move the downloaded file to the final location
            fs::rename(tmp_path, &bin).context("move downloaded file")?;
            bin.clone()
        };

        // Set executable permissions on Unix
        #[cfg(unix)]
        if desc.executable {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&final_bin)
                .context("get file metadata")?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&final_bin, perms).context("set executable permissions")?;
        }

        // Write marker file to indicate successful download and verification
        fs::write(&marker, b"ok").context("write marker file")?;

        println!(
            "[bazbom] cached {} {} to {:?}",
            desc.name, desc.version, final_bin
        );

        Ok(final_bin)
    }

    fn binary_name(&self, name: &str) -> String {
        if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
    }

    pub fn get_tool_path(&self, name: &str, version: &str) -> PathBuf {
        let dir = self.root.join(name).join(version);
        dir.join(self.binary_name(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_binary_name() {
        let temp = tempdir().unwrap();
        let cache = ToolCache::new(temp.path().to_path_buf());

        #[cfg(windows)]
        assert_eq!(cache.binary_name("semgrep"), "semgrep.exe");

        #[cfg(not(windows))]
        assert_eq!(cache.binary_name("semgrep"), "semgrep");
    }

    #[test]
    fn test_get_tool_path() {
        let temp = tempdir().unwrap();
        let cache = ToolCache::new(temp.path().to_path_buf());
        let path = cache.get_tool_path("semgrep", "1.78.0");

        assert!(path.to_string_lossy().contains("semgrep"));
        assert!(path.to_string_lossy().contains("1.78.0"));
    }
}
