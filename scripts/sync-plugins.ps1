<#
Sync plugin source folders into the workspace target plugins directory for development or release.
Usage:
  pwsh -ExecutionPolicy Bypass -File .\scripts\sync-plugins.ps1 -Configuration Debug
  pwsh -ExecutionPolicy Bypass -File .\scripts\sync-plugins.ps1 -Configuration Release

This script copies each subfolder of `crates/plugins/` that contains `plugin.toml`
into `target\<configuration>\plugins/<plugin-name>` so the running executable can
discover them when executed via `cargo run` (Debug) or the release binary.
#>

param(
    [ValidateSet('Debug','Release')]
    [string]$Configuration = 'Debug'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repoRoot = Resolve-Path -LiteralPath (Join-Path $scriptDir '..') | Select-Object -ExpandProperty Path
$pluginsSrc = Join-Path $repoRoot 'crates\plugins'
$targetPlugins = Join-Path $repoRoot ("target\{0}\plugins" -f $Configuration.ToLower())

if (-not (Test-Path $pluginsSrc)) {
    Write-Error "Source plugins directory not found: $pluginsSrc"
    exit 1
}

# Ensure destination exists
if (-not (Test-Path $targetPlugins)) {
    New-Item -ItemType Directory -Path $targetPlugins -Force | Out-Null
}

$targetDir = Join-Path $repoRoot ("target\{0}" -f $Configuration.ToLower())

Get-ChildItem -Directory -Path $pluginsSrc | ForEach-Object {
    $pluginDir = $_.FullName
    $pluginName = $_.Name
    $pluginToml = Join-Path $pluginDir 'plugin.toml'
    
    if (-not (Test-Path $pluginToml)) {
        Write-Host "Skipping $pluginName (no plugin.toml)"
        return
    }

    # Find the compiled library
    # Plugin directory names use hyphens, but Rust library names use underscores
    $libName = $pluginName -replace '-','_'
    $libPattern = "*$libName*"
    $libFiles = Get-ChildItem -Path $targetDir -Filter "*.dll" -ErrorAction SilentlyContinue |
                Where-Object { $_.Name -like $libPattern }
    
    if (-not $libFiles) {
        $libFiles = Get-ChildItem -Path $targetDir -Filter "*.so" -ErrorAction SilentlyContinue |
                    Where-Object { $_.Name -like $libPattern }
    }
    
    if (-not $libFiles) {
        $libFiles = Get-ChildItem -Path $targetDir -Filter "*.dylib" -ErrorAction SilentlyContinue |
                    Where-Object { $_.Name -like $libPattern }
    }

    if (-not $libFiles) {
        Write-Warning "No compiled library found for plugin '$pluginName'. Run 'cargo build' first."
        return
    }

    $dest = Join-Path $targetPlugins $pluginName
    
    # Clean existing directory
    if (Test-Path $dest) {
        Remove-Item -Recurse -Force -LiteralPath $dest
    }
    
    New-Item -ItemType Directory -Path $dest -Force | Out-Null

    Write-Host "Copying plugin '$pluginName' -> $dest"
    
    # Copy plugin.toml
    Copy-Item -Force -LiteralPath $pluginToml -Destination $dest
    
    # Copy compiled library
    foreach ($lib in $libFiles) {
        Copy-Item -Force -LiteralPath $lib.FullName -Destination $dest
        Write-Host "  - $($lib.Name)"
    }
}

Write-Host "Plugin sync complete for configuration: $Configuration"
