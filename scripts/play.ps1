# Starts the title with the settings this fork is played with (README.md, "How to run it").
#   scripts\play.ps1 -Exe <path of your own copy's executable>
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -Console      series-s for 1080p (default) or series-x for the 4K profile
# -RayTracing   the title's experimental ray-traced indirect light
# -ShadowBoost  lift the darkest areas with ray tracing off
# -LowMemory    use smaller memory windows for buffer copies
# -Build        target\diag (the default, the build all measurements are of) or target\release
# -Nvidia       use the installed VIctory-NVIDIA runner for DLSS and its F1 menu
# -More         other settings: -More @{VICTORY_PANEL='1'}
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Safe,
    [switch]$RayTracing,
    [switch]$ShadowBoost,
    [switch]$LowMemory,
    [switch]$OriginalGraphics,
    [switch]$PerformanceLog,
    [switch]$AudioTrace,
    [switch]$Nvidia,
    [ValidateSet('15', '30', '60', 'unlimited')][string]$FrameLimit,
    [string]$LegacyRoot,
    [ValidateSet('series-s', 'series-x')][string]$Console = 'series-s',
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [hashtable]$More = @{}
)
$settings = @{
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_PANEL = '1'; VICTORY_NAN_SCRUB = '1'; VICTORY_SYNC_FINE = '1'
    VICTORY_SHOW_AFTER = '1'; VICTORY_D3D12_DRED = 'light'; VICTORY_SKIP_PIPELINES = 'none'
}
if ($Safe) {
    $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
} else {
    $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'; $settings.VICTORY_UPLOAD_AHEAD = '1'
}
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1' }
if (!$OriginalGraphics -and !$RayTracing) {
    $settings.VICTORY_AMBIENT = '0.8'
    if ($ShadowBoost) { $settings.VICTORY_AMBIENT_FLOOR = '0.09' }
    $settings.VICTORY_AMBIENT_PROBE_LIMIT = '2'
    $settings.VICTORY_AMBIENT_AIR = '1.5'
}
if ($LowMemory) {
    $settings.VICTORY_WINDOW_KB = '2048'
}
if ($PerformanceLog) { $settings.VICTORY_LOG = 'warn,victory_gpu::host=info,victory_gpu::pipeline=info,victory_gpu::ambient=info' }
if ($AudioTrace) {
    $log = if ($settings.VICTORY_LOG) { $settings.VICTORY_LOG } else { 'warn' }
    $settings.VICTORY_LOG = "$log,victory_hle::modules::audio=debug,victory_hle::modules::audio::capture=warn,victory_hle::modules::apu=debug"
}
if ($FrameLimit) { $settings.VICTORY_FPS_LIMIT = $FrameLimit }
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = if ($Nvidia) { Join-Path $env:LOCALAPPDATA 'victory/nvidia-runner/victory.exe' }
    else { Join-Path $PSScriptRoot "..\target\$Build\victory.exe" }
if (-not (Test-Path -LiteralPath $victory -PathType Leaf)) { throw "No $victory. Build it first (README.md)." }
if (-not (Test-Path -LiteralPath $Exe -PathType Leaf)) { throw "No $Exe." }
if ($LegacyRoot) {
    $data = if (!$env:VICTORY_DATA_DIR) { Join-Path $env:LOCALAPPDATA 'victory/data' }
    $cache = if (!$env:VICTORY_CACHE_DIR) { Join-Path $env:LOCALAPPDATA 'victory/cache' }
    & (Join-Path $PSScriptRoot 'Migrate-VictoryStorage.ps1') -LegacyRoot $LegacyRoot -DataDirectory $data -CacheDirectory $cache
    if ($data) { $env:VICTORY_DATA_DIR = $data }
    if ($cache) { $env:VICTORY_CACHE_DIR = $cache }
}
# (Other settings left over in this window go; where your files are stays.)
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR'
if ($Nvidia) { $paths += 'VICTORY_STREAMLINE_DIR' }
Get-ChildItem Env:VICTORY_* | Where-Object { $_.Name -notin $paths } | ForEach-Object { Remove-Item "Env:$($_.Name)" }
$Exe = (Resolve-Path -LiteralPath $Exe).Path
if ($Nvidia) {
    # The two renderers use separate compiled caches and the same title saves.
    $env:VICTORY_CACHE_DIR = Join-Path $env:LOCALAPPDATA 'victory/cache-nvidia'
    $sdk = Join-Path $env:LOCALAPPDATA 'victory/nvidia-sdk/Streamline/bin/x64'
    if (!$env:VICTORY_STREAMLINE_DIR -and (Test-Path -LiteralPath (Join-Path $sdk 'sl.interposer.dll') -PathType Leaf)) {
        $env:VICTORY_STREAMLINE_DIR = $sdk
    }
    if ($settings.VICTORY_LOG) { $env:VICTORY_LOG = $settings.VICTORY_LOG }
    $launch = @('play', $Exe, '--dlss', 'quality', '--fg-type', 'dlss')
    if ($Console -eq 'series-s') { $launch += '--series-s' }
    if ($FrameLimit) { $launch += '--fps', $(if ($FrameLimit -eq 'unlimited') { '1000' } else { $FrameLimit }) }
    & $victory @launch
    exit $LASTEXITCODE
}
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
Push-Location (Join-Path $PSScriptRoot '..')   # victory.toml is looked for here
try { & $victory run $Exe --console $Console --execute } finally { Pop-Location }
exit $LASTEXITCODE
