# BazBOM Chocolatey Uninstallation Script
$ErrorActionPreference = 'Stop'

$packageName = 'bazbom'

Write-Host "Uninstalling BazBOM..." -ForegroundColor Cyan

# Chocolatey automatically removes files from tools directory
# This script is for cleanup of any additional files/settings

# Remove BazBOM cache directory (optional - ask user)
$cacheDir = Join-Path $env:USERPROFILE ".bazbom"
if (Test-Path $cacheDir) {
    $response = Read-Host "Remove BazBOM cache directory at $cacheDir? (y/N)"
    if ($response -eq 'y' -or $response -eq 'Y') {
        Remove-Item -Recurse -Force $cacheDir
        Write-Host "✓ Cache directory removed" -ForegroundColor Green
    } else {
        Write-Host "Cache directory preserved" -ForegroundColor Yellow
    }
}

Write-Host "✓ BazBOM uninstalled successfully!" -ForegroundColor Green
