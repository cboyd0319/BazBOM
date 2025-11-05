# BazBOM Windows Distribution

This directory contains Windows-specific packaging and distribution files for BazBOM.

## Distribution Channels

### 1. MSI Installer (Windows Installer)

**Directory:** `msi/`

The MSI installer provides a traditional Windows installation experience with:
- Automatic PATH configuration
- Start menu shortcuts
- Proper uninstaller
- Registry entries

**Requirements:**
- WiX Toolset 3.11+ (https://wixtoolset.org/)
- Windows 10 or later
- Administrator privileges (for machine-wide installation)

**Building:**

```powershell
cd msi
.\build-msi.ps1
```

This will:
1. Check for WiX Toolset installation
2. Verify bazbom.exe exists in release build
3. Compile WiX source files
4. Link into MSI package
5. Generate SHA256 checksum

**Output:**
- `msi/output/bazbom-{VERSION}-x86_64.msi`
- `msi/output/bazbom-{VERSION}-x86_64.msi.sha256`

**Testing:**

```powershell
# Install
msiexec /i bazbom-0.5.1-x86_64.msi /qb

# Verify
bazbom --version

# Uninstall
msiexec /x bazbom-0.5.1-x86_64.msi /qb
```

### 2. Chocolatey Package

**Directory:** `chocolatey/`

Chocolatey is the Windows package manager, similar to apt/yum on Linux or Homebrew on macOS.

**Files:**
- `bazbom.nuspec` - Package metadata and dependencies
- `tools/chocolateyinstall.ps1` - Installation script
- `tools/chocolateyuninstall.ps1` - Uninstallation script

**Building:**

```powershell
cd chocolatey

# Update checksum in tools/chocolateyinstall.ps1 first!
# Get checksum from GitHub release

# Pack the package
choco pack bazbom.nuspec

# Test installation locally
choco install bazbom -s . -y

# Verify
bazbom --version

# Uninstall
choco uninstall bazbom -y
```

**Publishing to Chocolatey.org:**

1. Create account at https://community.chocolatey.org/
2. Get API key from your profile
3. Push package:

```powershell
choco apikey --key YOUR-API-KEY --source https://push.chocolatey.org/
choco push bazbom.0.5.1.nupkg --source https://push.chocolatey.org/
```

**User Installation:**

```powershell
choco install bazbom
```

### 3. winget Package (Windows Package Manager)

**Directory:** `winget/`

winget is Microsoft's official package manager, built into Windows 11 and available for Windows 10.

**Files:**
- `BazBOM.BazBOM.yaml` - Package manifest

**Publishing to winget:**

1. Fork https://github.com/microsoft/winget-pkgs
2. Create directory: `manifests/b/BazBOM/BazBOM/0.5.1/`
3. Copy manifest file to this directory
4. Update InstallerSha256 with actual MSI checksum
5. Commit and create pull request

**User Installation:**

```powershell
winget install BazBOM.BazBOM
```

## Cross-Compilation from Linux/macOS

To build Windows binaries from Linux or macOS:

```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Install cross-compilation tools (Linux)
sudo apt install mingw-w64

# Build for Windows
cargo build --release --target x86_64-pc-windows-msvc

# Output will be in:
# target/x86_64-pc-windows-msvc/release/bazbom.exe
```

**Note:** Some dependencies may not cross-compile easily. Native Windows builds are recommended for production releases.

## GitHub Actions CI

To automate Windows builds in CI:

```yaml
# .github/workflows/windows-build.yml
name: Windows Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: x86_64-pc-windows-msvc
      
      - name: Build Release
        run: cargo build --release --target x86_64-pc-windows-msvc
      
      - name: Install WiX
        run: choco install wixtoolset -y
      
      - name: Build MSI
        run: |
          cd windows/msi
          .\build-msi.ps1
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: windows-installers
          path: |
            windows/msi/output/*.msi
            windows/msi/output/*.sha256
```

## Testing Checklist

Before releasing Windows packages:

- [ ] Build MSI installer successfully
- [ ] Install MSI on clean Windows 10 VM
- [ ] Verify bazbom.exe is in PATH
- [ ] Run `bazbom --version`
- [ ] Run `bazbom scan` on sample project
- [ ] Verify Start menu shortcut works
- [ ] Test uninstaller
- [ ] Build Chocolatey package
- [ ] Test Chocolatey installation
- [ ] Verify checksums match
- [ ] Test on Windows 11
- [ ] Test on Windows Server 2019/2022

## Known Issues

### Windows Defender / Antivirus

Some antivirus software may flag the binary. To resolve:

1. Submit binary to Microsoft for analysis: https://www.microsoft.com/en-us/wdsi/filesubmission
2. Code sign the binary with EV certificate (recommended for official releases)

### Long Path Support

Windows has a 260-character path limit by default. For projects with deep directory structures:

1. Enable long path support in Windows:
   ```
   Registry: HKLM\SYSTEM\CurrentControlSet\Control\FileSystem
   Set: LongPathsEnabled = 1 (DWORD)
   ```

2. Or use the `\\?\` prefix for absolute paths

## Code Signing (Optional)

For production releases, code signing improves trust and reduces security warnings:

1. Obtain code signing certificate (EV certificate recommended)
2. Sign the MSI:
   ```powershell
   signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 bazbom.msi
   ```

## Support

For Windows-specific issues:
- GitHub Issues: https://github.com/cboyd0319/BazBOM/issues
- Label: `platform: windows`

## Version History

### 0.5.1 (Current)
- Initial Windows distribution support
- MSI installer
- Chocolatey package
- winget manifest

## Contributing

To improve Windows support:
1. Test on various Windows versions
2. Report compatibility issues
3. Improve installer UX
4. Add Windows-specific features
5. Update documentation

## License

MIT License - See LICENSE file in repository root
