# PowerShell script to build filter_plotter for WASM
# Run from the filter_plotter directory

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "filter_plotter WASM Build Script" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Check if trunk is installed
Write-Host "Checking prerequisites..." -ForegroundColor Yellow
$trunkInstalled = Get-Command trunk -ErrorAction SilentlyContinue
if (-not $trunkInstalled) {
    Write-Host "❌ Trunk is not installed." -ForegroundColor Red
    Write-Host "Installing trunk..." -ForegroundColor Yellow
    cargo install trunk
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to install trunk" -ForegroundColor Red
        exit 1
    }
}
Write-Host "✅ Trunk is installed" -ForegroundColor Green

# Check if wasm32 target is installed
$wasmTarget = rustup target list | Select-String "wasm32-unknown-unknown \(installed\)"
if (-not $wasmTarget) {
    Write-Host "❌ wasm32-unknown-unknown target not installed." -ForegroundColor Red
    Write-Host "Installing wasm32 target..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to install wasm32 target" -ForegroundColor Red
        exit 1
    }
}
Write-Host "✅ wasm32 target is installed" -ForegroundColor Green

Write-Host ""
Write-Host "Building WASM application..." -ForegroundColor Yellow
Write-Host ""

# Build based on argument
if ($args[0] -eq "release") {
    Write-Host "Building RELEASE version..." -ForegroundColor Cyan
    trunk build --release
} elseif ($args[0] -eq "serve") {
    Write-Host "Starting development server..." -ForegroundColor Cyan
    Write-Host "App will be available at http://127.0.0.1:8080" -ForegroundColor Green
    trunk serve --open
} else {
    Write-Host "Building DEBUG version..." -ForegroundColor Cyan
    trunk build
}

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ Build successful!" -ForegroundColor Green
    Write-Host ""
    if ($args[0] -ne "serve") {
        Write-Host "Output is in the 'dist' directory" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "To serve locally:" -ForegroundColor Yellow
        Write-Host "  .\build_wasm.ps1 serve" -ForegroundColor White
        Write-Host ""
        Write-Host "To build release:" -ForegroundColor Yellow
        Write-Host "  .\build_wasm.ps1 release" -ForegroundColor White
    }
} else {
    Write-Host ""
    Write-Host "❌ Build failed!" -ForegroundColor Red
    Write-Host "Check the error messages above" -ForegroundColor Yellow
    exit 1
}
