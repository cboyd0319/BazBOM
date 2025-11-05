# BazBOM Chocolatey Installation Script
$ErrorActionPreference = 'Stop'

$packageName = 'bazbom'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$version = '0.5.1'

# Download URLs from GitHub releases
$url64 = "https://github.com/cboyd0319/BazBOM/releases/download/v$version/bazbom-x86_64-pc-windows-msvc.zip"

# Package parameters
$packageArgs = @{
  PackageName   = $packageName
  UnzipLocation = $toolsDir
  Url64bit      = $url64
  
  # Checksums - UPDATE THESE BEFORE PUBLISHING
  Checksum64    = 'CHECKSUM_PLACEHOLDER'
  ChecksumType64= 'sha256'
  
  # Download options
  ValidExitCodes= @(0)
}

# Install package
Write-Host "Installing BazBOM $version..." -ForegroundColor Cyan
Install-ChocolateyZipPackage @packageArgs

# Verify installation
$exePath = Join-Path $toolsDir "bazbom.exe"
if (Test-Path $exePath) {
    Write-Host "✓ BazBOM installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Get started with:" -ForegroundColor Yellow
    Write-Host "  bazbom --version" -ForegroundColor White
    Write-Host "  bazbom scan ." -ForegroundColor White
    Write-Host ""
    Write-Host "Full documentation:" -ForegroundColor Yellow
    Write-Host "  https://github.com/cboyd0319/BazBOM" -ForegroundColor White
} else {
    Write-Error "Installation failed: bazbom.exe not found"
}
