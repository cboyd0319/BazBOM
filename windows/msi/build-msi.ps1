# BazBOM MSI Installer Build Script
# Requires WiX Toolset 3.11+ to be installed

param(
    [Parameter(Mandatory=$false)]
    [string]$SourceDir = "..\..\target\x86_64-pc-windows-msvc\release",
    
    [Parameter(Mandatory=$false)]
    [string]$OutputDir = ".\output",
    
    [Parameter(Mandatory=$false)]
    [string]$Version = "0.5.1"
)

$ErrorActionPreference = "Stop"

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "   BazBOM MSI Installer Builder   " -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Check if WiX is installed
Write-Host "Checking for WiX Toolset..." -ForegroundColor Yellow
$wixPath = "${env:WIX}bin"
if (-not (Test-Path "$wixPath\candle.exe")) {
    Write-Error "WiX Toolset not found. Please install from: https://wixtoolset.org/releases/"
    exit 1
}
Write-Host "✓ WiX Toolset found at: $wixPath" -ForegroundColor Green

# Check if bazbom.exe exists
Write-Host "Checking for bazbom.exe..." -ForegroundColor Yellow
$exePath = Join-Path $SourceDir "bazbom.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "bazbom.exe not found at: $exePath"
    Write-Error "Please run 'cargo build --release --target x86_64-pc-windows-msvc' first"
    exit 1
}
Write-Host "✓ Found bazbom.exe" -ForegroundColor Green

# Create output directory
Write-Host "Creating output directory..." -ForegroundColor Yellow
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}
Write-Host "✓ Output directory ready" -ForegroundColor Green

# Copy files to staging directory
Write-Host "Preparing staging directory..." -ForegroundColor Yellow
$stagingDir = Join-Path $OutputDir "staging"
if (Test-Path $stagingDir) {
    Remove-Item -Recurse -Force $stagingDir
}
New-Item -ItemType Directory -Path $stagingDir | Out-Null

Copy-Item $exePath -Destination $stagingDir
Copy-Item "..\..\README.md" -Destination (Join-Path $stagingDir "README.txt")
Copy-Item "..\..\LICENSE" -Destination (Join-Path $stagingDir "LICENSE.txt")

# Create a simple RTF license file for WiX
$licenseRtf = @"
{\rtf1\ansi\deff0
{\fonttbl{\f0 Courier New;}}
\f0\fs20
BazBOM License Agreement\par
\par
See LICENSE.txt for full license terms.\par
\par
MIT License\par
\par
Copyright (c) 2025 BazBOM Project\par
}
"@
Set-Content -Path (Join-Path (Get-Location) "license.rtf") -Value $licenseRtf -Encoding ASCII

Write-Host "✓ Staging directory prepared" -ForegroundColor Green

# Run candle (WiX compiler)
Write-Host "Compiling WiX source..." -ForegroundColor Yellow
$wixObj = Join-Path $OutputDir "bazbom.wixobj"
& "$wixPath\candle.exe" `
    -arch x64 `
    -dSourceDir="$stagingDir" `
    -dVersion="$Version" `
    -out $wixObj `
    bazbom.wxs

if ($LASTEXITCODE -ne 0) {
    Write-Error "candle.exe failed with exit code $LASTEXITCODE"
    exit 1
}
Write-Host "✓ WiX source compiled" -ForegroundColor Green

# Run light (WiX linker)
Write-Host "Linking MSI package..." -ForegroundColor Yellow
$msiPath = Join-Path $OutputDir "bazbom-$Version-x86_64.msi"
& "$wixPath\light.exe" `
    -ext WixUIExtension `
    -out $msiPath `
    $wixObj

if ($LASTEXITCODE -ne 0) {
    Write-Error "light.exe failed with exit code $LASTEXITCODE"
    exit 1
}
Write-Host "✓ MSI package created" -ForegroundColor Green

# Calculate checksum
Write-Host "Calculating SHA256 checksum..." -ForegroundColor Yellow
$hash = Get-FileHash -Path $msiPath -Algorithm SHA256
$checksumPath = Join-Path $OutputDir "bazbom-$Version-x86_64.msi.sha256"
Set-Content -Path $checksumPath -Value "$($hash.Hash)  bazbom-$Version-x86_64.msi"
Write-Host "✓ Checksum calculated" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "     Build Complete!              " -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "MSI Package: $msiPath" -ForegroundColor White
Write-Host "SHA256:      $checksumPath" -ForegroundColor White
Write-Host "Size:        $([math]::Round((Get-Item $msiPath).Length / 1MB, 2)) MB" -ForegroundColor White
Write-Host ""
Write-Host "To test installation:" -ForegroundColor Yellow
Write-Host "  msiexec /i $msiPath /qb" -ForegroundColor White
Write-Host ""
Write-Host "To uninstall:" -ForegroundColor Yellow
Write-Host "  msiexec /x $msiPath /qb" -ForegroundColor White
Write-Host ""
