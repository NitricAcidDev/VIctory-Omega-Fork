# Starts the title with the settings used by Omega Launcher (see README.md).
#   scripts\play.ps1 -Exe <path of your own copy's executable>
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -RayTracing   the title's own ray-traced indirect light: lit interiors, about a third slower
# -Build        target\release (default) or target\diag
# -More         other settings: -More @{VICTORY_PANEL='1'}
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [ValidateSet('series-s', 'series-x')][string]$Console = 'series-s',
    [switch]$Safe,
    [switch]$RayTracing,
    [ValidateSet('diag', 'release')][string]$Build = 'release',
    [hashtable]$More = @{}
)
$settings = @{
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_OUTPUT_1080P = '1'; VICTORY_PANEL = '1'; VICTORY_NAN_SCRUB = '1'; VICTORY_SYNC_FINE = '1'
    VICTORY_AMBIENT_FLOOR = '0.05'; VICTORY_AMBIENT_PROBE_LIMIT = '2'; VICTORY_AMBIENT_AIR = '1.5'
    VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'; VICTORY_SKIP_PIPELINES = 'none'
}
if ($Safe) {
    $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
} else {
    $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'; $settings.VICTORY_UPLOAD_AHEAD = '1'
    $settings.VICTORY_LEVELS_ON_DEMAND = '1'; $settings.VICTORY_CARD_BUFFERS_MIN_KB = '64'
}
if ($RayTracing) {
    $settings.VICTORY_BVH_BUILDER = '1'
} elseif ($Console -eq 'series-x') {
    $settings.VICTORY_AMBIENT_FLOOR = '0.09'
}
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = Join-Path $PSScriptRoot "..\target\$Build\victory.exe"
if (-not (Test-Path $victory)) { throw "No $victory. Build it first (README.md)." }
if (-not (Test-Path $Exe)) { throw "No $Exe." }
# (Other settings left over in this window go; where your files are stays.)
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR'
Get-ChildItem Env:VICTORY_* | Where-Object { $_.Name -notin $paths } | ForEach-Object { Remove-Item "Env:$($_.Name)" }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
$Exe = (Resolve-Path $Exe).Path
Push-Location (Join-Path $PSScriptRoot '..')   # victory.toml is looked for here
try { & $victory run $Exe --console $Console --execute } finally { Pop-Location }
exit $LASTEXITCODE
