<#
Build the workspace (debug and/or release) and sync plugins into the corresponding
`target/<configuration>/plugins/` directories so the compiled binary can discover them.

Usage examples:
  # Build debug and sync
  pwsh -ExecutionPolicy Bypass -File .\scripts\build_and_sync.ps1 -Configurations Debug

  # Build release and sync
  pwsh -ExecutionPolicy Bypass -File .\scripts\build_and_sync.ps1 -Configurations Release

  # Build both debug and release and sync
  pwsh -ExecutionPolicy Bypass -File .\scripts\build_and_sync.ps1 -Configurations Debug,Release
#>

param(
    [ValidateSet('Debug','Release')]
    [string[]]$Configurations = @('Debug')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repoRoot = Resolve-Path -LiteralPath (Join-Path $scriptDir '..') | Select-Object -ExpandProperty Path

# Run builds and sync for each requested configuration
foreach ($cfg in $Configurations) {
    Write-Host "Building configuration: $cfg"
    Push-Location -LiteralPath $repoRoot
    try {
        if ($cfg -eq 'Debug') {
            cargo build
        } else {
            cargo build --release
        }
    } finally {
        Pop-Location
    }

    Write-Host "Syncing plugins for: $cfg"
    & pwsh -ExecutionPolicy Bypass -File (Join-Path $repoRoot 'scripts\sync-plugins.ps1') -Configuration $cfg
}

Write-Host "Build and plugin sync complete for: $($Configurations -join ',')"
