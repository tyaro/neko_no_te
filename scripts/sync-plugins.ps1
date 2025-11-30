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

Get-ChildItem -Directory -Path $pluginsSrc | ForEach-Object {
    $pluginDir = $_.FullName
    $pluginToml = Join-Path $pluginDir 'plugin.toml'
    if (-not (Test-Path $pluginToml)) {
        Write-Host "Skipping $($_.Name) (no plugin.toml)"
        return
    }

    $dest = Join-Path $targetPlugins $_.Name
    if (Test-Path $dest) {
        Remove-Item -Recurse -Force -LiteralPath $dest
    }

    Write-Host "Copying plugin '$($_.Name)' -> $dest"
    Copy-Item -Recurse -Force -LiteralPath $pluginDir -Destination $dest
}

Write-Host "Plugin sync complete for configuration: $Configuration"
