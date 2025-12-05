<#
Copies built plugin directories from target/debug/plugins/ to the repository-level plugins/ directory.
This is a convenience for development: after cargo build, run this to ensure repo/plugins/ contains the same folders (DLLs + manifest).

Usage (PowerShell):
  pwsh .\scripts\sync_plugins_to_repo.ps1 [-SourcePath <path>] [-TargetPath <path>] [-WhatIf]

Defaults:
  SourcePath = ./target/debug/plugins
  TargetPath = ./plugins
#>

param(
    [string]$SourcePath = "target/debug/plugins",
    [string]$TargetPath = "plugins"
)

Write-Host "Source: $SourcePath"
Write-Host "Target: $TargetPath"

if (-not (Test-Path $SourcePath)) {
    Write-Error "Source path does not exist: $SourcePath"
    exit 2
}

# Ensure target exists
if (-not (Test-Path $TargetPath)) {
    New-Item -ItemType Directory -Path $TargetPath | Out-Null
}

$dirs = Get-ChildItem -Path $SourcePath -Directory -ErrorAction Stop
foreach ($d in $dirs) {
    $src = Join-Path $SourcePath $d.Name
    $dst = Join-Path $TargetPath $d.Name

    Write-Host "Syncing $($d.Name)..."
    if (Test-Path $dst) {
        # remove before copy to avoid stale files
        Remove-Item -Path $dst -Recurse -Force -ErrorAction SilentlyContinue
    }
    Copy-Item -Path $src -Destination $dst -Recurse -Force
}

Write-Host "Done. Plugins copied to: $TargetPath"
