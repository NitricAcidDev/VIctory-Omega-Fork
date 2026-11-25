$ErrorActionPreference = 'Stop'
$containerRoot = Split-Path -Parent $PSScriptRoot
$run = Get-ChildItem -LiteralPath (Join-Path $containerRoot 'logs\primary') -Directory | Sort-Object Name -Descending | Select-Object -First 1
$game = Get-Process -Name victory -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (!$game -or !$run) { throw 'Start the game with the normal launcher first, then run this monitor.' }
$ffmpeg = Get-Command ffmpeg.exe -ErrorAction SilentlyContinue
$encoder = if ($ffmpeg) { $ffmpeg.Source } else { 'C:\Program Files\Softdeluxe\Free Download Manager\ffmpeg.exe' }
if (!(Test-Path -LiteralPath $encoder -PathType Leaf)) { throw 'FFmpeg was not found. Install FFmpeg or add it to PATH.' }
$folder = Join-Path $run.FullName ('monitor-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $folder | Out-Null
Write-Host "Recording the desktop to $folder. Press Q to stop."
Write-Host 'The run folder contains the game log. Screenshots are saved every 30 seconds.'
$args = @('-hide_banner','-loglevel','warning','-y','-f','gdigrab','-framerate','10','-draw_mouse','0','-i','desktop','-filter_complex','[0:v]split=2[vid][still];[still]fps=1/30[shots]','-map','[vid]','-c:v','h264_mf','-pix_fmt','yuv420p', (Join-Path $folder 'recording.mkv'), '-map','[shots]','-vsync','vfr', (Join-Path $folder 'screenshot-%05d.jpg'))
& $encoder @args
if ($LASTEXITCODE -ne 0) { throw "Recording stopped with error $LASTEXITCODE." }