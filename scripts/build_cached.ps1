<#
Build helper that enables sccache when available and builds the workspace with features.

Usage (PowerShell):
  ./scripts/build_cached.ps1 -Features "gui" -Jobs 6

#>
param(
    [string] $Features = "",
    [int] $Jobs = 6
)

Write-Host "Preparing cached build..."

# Resolve sccache: prefer command in PATH, otherwise check common cargo/bin locations.
$sccachePath = $null
try {
    $cmd = Get-Command sccache -ErrorAction SilentlyContinue
    if ($cmd) { $sccachePath = $cmd.Source }
} catch { }

if (-not $sccachePath) {
    $candidates = @()
    if ($env:CARGO_HOME) { $candidates += Join-Path $env:CARGO_HOME 'bin\sccache.exe' }
    $candidates += Join-Path $env:USERPROFILE '.cargo\bin\sccache.exe'
    $candidates += 'C:\Program Files\sccache\sccache.exe'
    $candidates += 'C:\Program Files (x86)\sccache\sccache.exe'

    foreach ($p in $candidates) {
        if (Test-Path $p) { $sccachePath = (Resolve-Path $p).Path; break }
    }
}

if ($sccachePath) {
    Write-Host "sccache found: $sccachePath -- enabling rustc wrapper."
    $env:RUSTC_WRAPPER = $sccachePath
} else {
    Write-Host "sccache not found (PATH or common locations) — continuing without rustc cache."
}

# Ensure we run from repository root (script path is in scripts/)
# Run from repository root (script is in scripts/)
Push-Location -LiteralPath (Join-Path $PSScriptRoot "..")

if ($Features -ne "") {
    Write-Host "Running: cargo build --features $Features -j $Jobs"
    cargo build --features $Features -j $Jobs
} else {
    Write-Host "Running: cargo build -j $Jobs"
    cargo build -j $Jobs
}

Pop-Location

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
} else {
    Write-Host "Build finished successfully." -ForegroundColor Green
}
