$ErrorActionPreference = 'Stop'
$repo = $PSScriptRoot
$containerRoot = Split-Path -Parent $repo
$cargo = (Get-Command cargo.exe -ErrorAction Stop).Source
$env:VICTORY_INTRO_UPLOADS = '1'
if (-not $env:VICTORY_CACHE_DIR) { $env:VICTORY_CACHE_DIR = Join-Path $containerRoot 'cache\fxceless-rendering' }
if (-not $env:VICTORY_LOG) { $env:VICTORY_LOG = 'warn,victory_gpu::host=info,victory_gpu::texture=info' }
Push-Location $repo
try {
    & $cargo build --locked --release -p victory
    if ($LASTEXITCODE -ne 0) { throw 'VIctory build failed.' }
} finally { Pop-Location }
& (Join-Path $PSScriptRoot 'launch.ps1') -CheckoutPath $repo -LauncherTitle 'VIctory Primary' -RunLogDirectory (Join-Path $containerRoot 'logs\primary')
