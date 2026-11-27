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
$build = Join-Path $source 'omega-build-status-off'
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

$trainerBinary = Join-Path $build 'shvi/plugins/trainer.asi'
$trainerTarget = Join-Path $game 'shvi/plugins/trainer.asi'
$trainerMarker = Join-Path $game 'shvi/omega-status-line-off'
$deferred = $false
$unmodifiedTrainer = Join-Path $source 'omega-build/shvi/plugins/trainer.asi'
$originalTrainerHash = if (Test-Path -LiteralPath $unmodifiedTrainer -PathType Leaf) {
    (Get-FileHash -LiteralPath $unmodifiedTrainer).Hash
}
$managedTrainerHash = if (Test-Path -LiteralPath $trainerMarker -PathType Leaf) {
    [IO.File]::ReadAllText($trainerMarker).Trim()
} elseif (Test-Path -LiteralPath "$trainerTarget.next-hashes" -PathType Leaf) {
    $pendingHashes = [IO.File]::ReadAllLines("$trainerTarget.next-hashes")
    if ($pendingHashes.Length -eq 2) { $pendingHashes[0] }
}

$header = Join-Path $source 'samples/trainer/trainer.hpp'
$original = [IO.File]::ReadAllBytes($header)
$upstream = [Text.Encoding]::UTF8.GetString($original)
$default = 'bool showStatus = true;'
if ([regex]::Matches($upstream, [regex]::Escape($default)).Count -ne 1) {
    throw 'Could not find the pinned trainer status-line default.'
}
$patched = $upstream.Replace($default, 'bool showStatus = false;')
try {
    [IO.File]::WriteAllText($header, $patched, [Text.UTF8Encoding]::new($false))
    cmake -S $source -B $build -A x64
    if ($LASTEXITCODE) { throw 'ScriptHook VI configuration failed.' }
    cmake --build $build --config Release --target shvi_core trainer --parallel
    if ($LASTEXITCODE) { throw 'ScriptHook VI build failed.' }
} finally {
    if ([IO.File]::ReadAllText($header) -eq $patched) {
        [IO.File]::WriteAllBytes($header, $original)
    }
}
if ($originalTrainerHash -and (Get-FileHash -LiteralPath $trainerBinary).Hash -eq $originalTrainerHash) {
    throw 'The rebuilt trainer still matches the unmodified trainer.'
}

$files = [ordered]@{
    'shvi/plugins/trainer.asi' = $trainerBinary
    'shvi/natives/1794606036.json' = (Join-Path $source 'nativedb/db/1794606036/native_table_full.json')
    'shvi/shvi_core.dll' = (Join-Path $build 'Release/shvi_core.dll')
}
$previousCore = Join-Path $source 'omega-build/Release/shvi_core.dll'
$installedCore = Join-Path $game 'shvi/shvi_core.dll'
if ((Test-Path -LiteralPath $previousCore -PathType Leaf) -and
    (Test-Path -LiteralPath $installedCore -PathType Leaf) -and
    (Get-FileHash -LiteralPath $previousCore).Hash -eq (Get-FileHash -LiteralPath $installedCore).Hash) {
    $files['shvi/shvi_core.dll'] = $previousCore
}
foreach ($relative in $files.Keys) {
    $target = Join-Path $game $relative
    $sourceFile = $files[$relative]
    if (!(Test-Path -LiteralPath $sourceFile -PathType Leaf)) { throw "Missing build output $sourceFile" }
    if (Test-Path -LiteralPath $target -PathType Leaf) {
        $current = (Get-FileHash -LiteralPath $target).Hash
        if ($current -eq (Get-FileHash -LiteralPath $sourceFile).Hash) { continue }
        if ($target -ne $trainerTarget -or $current -notin @($originalTrainerHash, $managedTrainerHash)) {
            throw "Preserving the existing ScriptHook VI file at $target."
        }
    }
}
foreach ($relative in $files.Keys) {
    $target = Join-Path $game $relative
    $null = [IO.Directory]::CreateDirectory((Split-Path $target))
    if (!(Test-Path -LiteralPath $target)) {
        [IO.File]::Copy($files[$relative], $target)
    } elseif ($target -eq $trainerTarget -and
        (Get-FileHash -LiteralPath $target).Hash -ne (Get-FileHash -LiteralPath $trainerBinary).Hash) {
        if (Get-Process victory -ErrorAction SilentlyContinue) {
            $oldHash = (Get-FileHash -LiteralPath $target).Hash
            [IO.File]::Copy($trainerBinary, "$target.next", $true)
            $newHash = (Get-FileHash -LiteralPath $trainerBinary).Hash
            [IO.File]::WriteAllLines("$target.next-hashes", @($oldHash, $newHash))
            $deferred = $true
            continue
        }
        $staged = $target + '.' + [Guid]::NewGuid().ToString('N') + '.tmp'
        $backup = $target + '.' + [Guid]::NewGuid().ToString('N') + '.backup'
        try {
            [IO.File]::Copy($trainerBinary, $staged)
            [IO.File]::Replace($staged, $target, $backup)
        } finally {
            [IO.File]::Delete($staged)
        }
    }
}
if ($deferred) {
    Write-Output 'ScriptHook VI update staged. Close Victory and launch with the trainer enabled to install it.'
    return
}
[IO.File]::WriteAllText($trainerMarker, (Get-FileHash -LiteralPath $trainerTarget).Hash)
Write-Output 'ScriptHook VI trainer installed. Press F4 in offline story gameplay.'
