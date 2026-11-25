[CmdletBinding()]
param(
    [ValidateSet('series-x', 'series-s')]
    [string]$Console = 'series-x',
    [switch]$Diagnostics,
    [string]$CheckoutPath,
    [string]$LauncherTitle = 'VIctory Launcher',
    [string]$RunLogDirectory,
    [string]$CaptureFrames,
    [switch]$CaptureWorld,
    [switch]$DumpTargets
)

$ErrorActionPreference = 'Stop'
$env:VICTORY_BVH_BUILDER = '0'
$containerRoot = Split-Path -Parent $PSScriptRoot
$repoRoot = if ($CheckoutPath) { $CheckoutPath } else { $PSScriptRoot }
$lastUsedFile = Join-Path $containerRoot 'data\last-game-executable.txt'
$releaseExe = Join-Path $repoRoot 'target\release\victory.exe'
$diagnosticsExe = Join-Path $repoRoot 'target\diag\victory.exe'
$victoryExe = if ($Diagnostics) { $diagnosticsExe } else { $releaseExe }

function Read-LastExecutable {
    if (Test-Path -LiteralPath $lastUsedFile -PathType Leaf) {
        $saved = (Get-Content -LiteralPath $lastUsedFile -Raw).Trim()
        if ($saved -and (Test-Path -LiteralPath $saved -PathType Leaf)) { return $saved }
    }
    return $null
}

function Select-GameExecutable {
    Add-Type -AssemblyName System.Windows.Forms
    $dialog = New-Object System.Windows.Forms.OpenFileDialog
    $dialog.Title = 'Select game_scarlett_master.exe'
    $dialog.Filter = 'GTA VI executable (game_scarlett_master.exe)|game_scarlett_master.exe'
    $dialog.CheckFileExists = $true
    $dialog.Multiselect = $false
    $dialog.FileName = 'game_scarlett_master.exe'
    $last = Read-LastExecutable
    if ($last) { $dialog.InitialDirectory = Split-Path -Parent $last }
    elseif ($env:VICTORY_GAME_DIR -and (Test-Path -LiteralPath $env:VICTORY_GAME_DIR -PathType Container)) { $dialog.InitialDirectory = $env:VICTORY_GAME_DIR }
    $result = $dialog.ShowDialog()
    if ($result -eq [System.Windows.Forms.DialogResult]::OK) { return $dialog.FileName }
    return $null
}

try {
    if (-not (Test-Path -LiteralPath $repoRoot -PathType Container)) { throw "VIctory checkout not found: $repoRoot" }
    if (-not (Test-Path -LiteralPath $victoryExe -PathType Leaf)) {
        if ($Diagnostics) { throw 'Diagnostics build not found. Build it with: cargo build --profile diag -p victory --features diagnostics' }
        throw 'Release build not found. Build it with: cargo build --release -p victory'
    }

    if (-not $env:VICTORY_CONFIG -and -not (Test-Path -LiteralPath (Join-Path $repoRoot 'victory.toml'))) {
        if (-not $env:VICTORY_CACHE_DIR) { $env:VICTORY_CACHE_DIR = Join-Path $containerRoot 'cache' }
        if (-not $env:VICTORY_DATA_DIR) { $env:VICTORY_DATA_DIR = Join-Path $containerRoot 'data' }
    }

    while ($true) {
        Clear-Host
        $last = Read-LastExecutable
        Write-Host $LauncherTitle
        Write-Host ''
        if ($last) { Write-Host "Last executable: $last" }
        else { Write-Host 'No last-used executable saved yet.' }
        Write-Host ''
        Write-Host '1  Use last-used executable'
        Write-Host '2  Select executable...'
        Write-Host '0  Exit'
        $choice = Read-Host 'Choose an option'

        if ($choice -eq '0') { break }
        if ($choice -eq '1') {
            if (-not $last) {
                Write-Host 'No valid last-used executable. Choose option 2 first.' -ForegroundColor Yellow
                [void](Read-Host 'Press Enter to continue')
                continue
            }
            $selectedExe = $last
        } elseif ($choice -eq '2') {
            $selectedExe = Select-GameExecutable
            if (-not $selectedExe) { continue }
        } else {
            Write-Host 'Choose 1, 2, or 0.' -ForegroundColor Yellow
            [void](Read-Host 'Press Enter to continue')
            continue
        }

        if ([IO.Path]::GetFileName($selectedExe) -ine 'game_scarlett_master.exe') {
            Write-Host 'Select game_scarlett_master.exe.' -ForegroundColor Yellow
            [void](Read-Host 'Press Enter to continue')
            continue
        }
        $selectedExe = (Resolve-Path -LiteralPath $selectedExe).Path
        $gameDir = Split-Path -Parent $selectedExe
        $repoPrefix = [IO.Path]::GetFullPath($repoRoot).TrimEnd('\') + '\'
        if ($gameDir.StartsWith($repoPrefix, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'The game files must remain outside the VIctory repository.'
        }

        $saveDir = Split-Path -Parent $lastUsedFile
        New-Item -ItemType Directory -Path $saveDir -Force | Out-Null
        Set-Content -LiteralPath $lastUsedFile -Value $selectedExe -Encoding UTF8
        $env:VICTORY_GAME_DIR = $gameDir

        Write-Host ''
        Write-Host "Starting VIctory from $selectedExe"
        Write-Host 'Close the game window to return here; choose 0 to close this launcher window.'
        Push-Location $repoRoot
        try {
            if ($RunLogDirectory) {
                $runName = (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + "-$PID"
                $runDirectory = Join-Path $RunLogDirectory $runName
                New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null
                Write-Host "Run logs: $runDirectory"
                @(
                    $LauncherTitle
                    "Executable: $victoryExe"
                    "SHA256: $((Get-FileHash -LiteralPath $victoryExe -Algorithm SHA256).Hash)"
                ) | Set-Content -LiteralPath (Join-Path $runDirectory 'build.txt') -Encoding UTF8
                if ($CaptureFrames -or $CaptureWorld) {
                    $env:VICTORY_CAPTURE = $runDirectory
                    if ($CaptureFrames) { $env:VICTORY_CAPTURE_FRAMES = $CaptureFrames }
                    if ($CaptureWorld) { $env:VICTORY_CAPTURE_WORLD = '1' }
                    if ($DumpTargets) { $env:VICTORY_DUMP_TARGETS = '1' }
                }
                $gameArgs = @('run', ('"' + $selectedExe + '"'), '--execute', '--console', $Console)
                $run = Start-Process -FilePath $victoryExe -ArgumentList $gameArgs -WorkingDirectory $repoRoot -NoNewWindow -PassThru -Wait `
                    -RedirectStandardOutput (Join-Path $runDirectory 'stdout.log') `
                    -RedirectStandardError (Join-Path $runDirectory 'stderr.log')
                $runExitCode = $run.ExitCode
                Get-Content -LiteralPath (Join-Path $runDirectory 'stderr.log') -Tail 8
            } else {
                & $victoryExe run $selectedExe --execute --console $Console
                $runExitCode = $LASTEXITCODE
            }
        } finally {
            Pop-Location
        }
        Write-Host "VIctory exited with code $runExitCode."
        [void](Read-Host 'Press Enter to return to the launcher')
    }
} catch {
    Write-Host "Launcher error: $($_.Exception.Message)" -ForegroundColor Red
    [void](Read-Host 'Press Enter to close this window')
}

