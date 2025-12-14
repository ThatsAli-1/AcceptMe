# AcceptMe - Run Script
# This script refreshes PATH and runs the Tauri app

Write-Host "Starting AcceptMe..." -ForegroundColor Green

# Refresh PATH to include Rust/Cargo
$env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Verify Cargo is available
$cargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCheck) {
    Write-Host "ERROR: Cargo not found in PATH!" -ForegroundColor Red
    Write-Host "Please restart PowerShell or run this command first:" -ForegroundColor Yellow
    Write-Host '$env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")' -ForegroundColor Cyan
    exit 1
}

Write-Host "Cargo found: $(cargo --version)" -ForegroundColor Green
Write-Host ""
Write-Host "Starting Tauri dev server..." -ForegroundColor Yellow
Write-Host "This may take a few minutes on first run..." -ForegroundColor Yellow
Write-Host ""

# Run Tauri
npm run tauri dev

