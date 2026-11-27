param(
    [string]$ShortcutDirectory = $PSScriptRoot,
    [switch]$Diagnostics
)

$ErrorActionPreference = 'Stop'
$shortcutRoot = (Resolve-Path -LiteralPath $ShortcutDirectory).Path
Push-Location $PSScriptRoot
try {
    if ($Diagnostics) {
        cargo build --locked --profile diag --features diagnostics -p victory
    } else {
        cargo build --locked --release -p victory
    }
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
    & powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'tools/omega-launcher/build.ps1')
    if ($LASTEXITCODE) { exit $LASTEXITCODE }

    $launcher = Join-Path $PSScriptRoot 'target/omega-launcher/Omega Launcher.exe'
    $shortcutPath = Join-Path $shortcutRoot 'Omega Launcher.lnk'
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $launcher
    $shortcut.WorkingDirectory = $shortcutRoot
    $shortcut.IconLocation = "$launcher,0"
    $shortcut.Description = 'Set up and launch Victory Omega'
    $shortcut.Save()
    Write-Output $shortcutPath
} finally {
    Pop-Location
}
