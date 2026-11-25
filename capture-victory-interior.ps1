$ErrorActionPreference = 'Stop'
$expected = Join-Path $PSScriptRoot 'target\diag\victory.exe'
$containerRoot = Split-Path -Parent $PSScriptRoot
if (!(Get-Process -Name victory -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $expected })) {
    throw 'Open the game using Launch VIctory.bat or Launch VIctory Interior Capture.bat first.'
}
$run = Get-ChildItem -LiteralPath (Join-Path $containerRoot 'logs\primary') -Directory |
    Sort-Object Name -Descending | Select-Object -First 1
if (!$run -or (Get-Content -LiteralPath (Join-Path $run.FullName 'build.txt') -TotalCount 1) -notin @('VIctory Interior Capture', 'VIctory Primary')) {
    throw 'The latest run is not an interior capture session.'
}
New-Item -Path (Join-Path $run.FullName 'capture-next-frame') -ItemType File -Force | Out-Null
Write-Host "Capture requested in $($run.FullName)"
