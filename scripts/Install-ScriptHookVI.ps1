param(
    [Parameter(Mandatory)]
    [string]$GameDirectory
)

$ErrorActionPreference = 'Stop'
$game = (Resolve-Path -LiteralPath $GameDirectory).Path
if (!(Test-Path -LiteralPath (Join-Path $game 'game_scarlett_master.exe') -PathType Leaf)) {
    throw 'Select the Xbox game folder containing game_scarlett_master.exe.'
}

$revision = '57b1946d1a72ef055766ac76f2145151ee53feb6'
$source = Join-Path $env:LOCALAPPDATA 'victory/scripthook-vi-source'
$build = Join-Path $source 'omega-build'
if (!(Test-Path -LiteralPath (Join-Path $source '.git'))) {
    git init $source
    if ($LASTEXITCODE) { throw 'Could not create the ScriptHook VI source checkout.' }
    git -C $source remote add origin https://github.com/modpapi/scripthook-vi.git
    if ($LASTEXITCODE) { throw 'Could not configure the ScriptHook VI source checkout.' }
    git -C $source fetch --depth 1 origin $revision
    if ($LASTEXITCODE) { throw 'Could not fetch the pinned ScriptHook VI source.' }
    git -C $source switch --detach $revision
    if ($LASTEXITCODE) { throw 'Could not check out the pinned ScriptHook VI source.' }
}
$actual = git -C $source rev-parse HEAD
if ($LASTEXITCODE -or $actual -ne $revision) { throw 'Expected the pinned ScriptHook VI source revision.' }
$changes = git -C $source status --porcelain --untracked-files=no
if ($LASTEXITCODE -or $changes) { throw 'ScriptHook VI source has local changes.' }

cmake -S $source -B $build -A x64
if ($LASTEXITCODE) { throw 'ScriptHook VI configuration failed.' }
cmake --build $build --config Release --target shvi_core trainer --parallel
if ($LASTEXITCODE) { throw 'ScriptHook VI build failed.' }

$files = [ordered]@{
    'shvi/plugins/trainer.asi' = (Join-Path $build 'shvi/plugins/trainer.asi')
    'shvi/natives/1794606036.json' = (Join-Path $source 'nativedb/db/1794606036/native_table_full.json')
    'shvi/shvi_core.dll' = (Join-Path $build 'Release/shvi_core.dll')
}
foreach ($relative in $files.Keys) {
    $target = Join-Path $game $relative
    $sourceFile = $files[$relative]
    if (!(Test-Path -LiteralPath $sourceFile -PathType Leaf)) { throw "Missing build output $sourceFile" }
    if ((Test-Path -LiteralPath $target) -and (Get-FileHash -LiteralPath $target).Hash -ne (Get-FileHash -LiteralPath $sourceFile).Hash) {
        throw "Preserving the existing ScriptHook VI file at $target."
    }
}
foreach ($relative in $files.Keys) {
    $target = Join-Path $game $relative
    $null = [IO.Directory]::CreateDirectory((Split-Path $target))
    if (!(Test-Path -LiteralPath $target)) {
        [IO.File]::Copy($files[$relative], $target)
    }
}
Write-Output 'ScriptHook VI trainer installed. Press F4 in offline story gameplay.'
