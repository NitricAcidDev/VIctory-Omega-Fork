param([switch]$Performance)

$ErrorActionPreference = 'Stop'
$repo = $PSScriptRoot
$containerRoot = Split-Path -Parent $repo
$cargoCommand = Get-Command cargo.exe -ErrorAction SilentlyContinue
$cargo = if ($cargoCommand) { $cargoCommand.Source } else { Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }
$builtExe = Join-Path $repo 'target\diag\victory.exe'
$env:VICTORY_INTRO_UPLOADS = '1'
$env:VICTORY_CAPTURE_ON_REQUEST = '1'
$env:VICTORY_CAPTURE_MANUAL_ONLY = '1'
if ($Performance) {
    # Let GPU work and CPU readback processing overlap when the title permits it.
    $env:VICTORY_RELAY = '1'
}
if (-not $env:VICTORY_CACHE_DIR) { $env:VICTORY_CACHE_DIR = Join-Path $containerRoot 'cache\fxceless-rendering' }
if (-not $env:VICTORY_LOG) { $env:VICTORY_LOG = 'warn,victory_gpu::host=info,victory_gpu::texture=info,victory_gpu::capture=info' }
if (Test-Path -LiteralPath $cargo -PathType Leaf) {
    Push-Location $repo
    try {
        & $cargo build --locked --profile diag -p victory --features diagnostics
        if ($LASTEXITCODE -ne 0) { throw 'VIctory build failed.' }
    } finally { Pop-Location }
} elseif (Test-Path -LiteralPath $builtExe -PathType Leaf) {
    Write-Host 'Cargo was not found. Starting the existing build.'
} else {
    throw 'Cargo and the existing VIctory build were not found. Install Rust or restore the built executable.'
}
& (Join-Path $PSScriptRoot 'launch.ps1') -CheckoutPath $repo -LauncherTitle 'VIctory Primary' -RunLogDirectory (Join-Path $containerRoot 'logs\primary') -Diagnostics -CaptureFrames 0 -DumpTargets
