param(
    [string]$LegacyRoot,
    [string]$DataDirectory = (Join-Path $env:LOCALAPPDATA 'victory/data'),
    [string]$CacheDirectory = (Join-Path $env:LOCALAPPDATA 'victory/cache')
)

$ErrorActionPreference = 'Stop'
if (!$LegacyRoot) { return }
$legacy = (Resolve-Path -LiteralPath $LegacyRoot).Path

function Copy-LegacyTree([string]$Source, [string]$Destination, [switch]$MergePipelineHistory) {
    if (!(Test-Path -LiteralPath $Source -PathType Container)) { return }
    $sourcePath = [IO.Path]::GetFullPath($Source).TrimEnd('\', '/')
    $targetPath = [IO.Path]::GetFullPath($Destination).TrimEnd('\', '/')
    if ($sourcePath -eq $targetPath) { return }
    if ($targetPath.StartsWith($sourcePath + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Victory storage must be outside the legacy source directory.'
    }

    $hash = [Security.Cryptography.SHA256]::Create()
    try {
        $key = [BitConverter]::ToString($hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($sourcePath.ToUpperInvariant()))).Replace('-', '')
    } finally {
        $hash.Dispose()
    }
    $marker = Join-Path $targetPath ".omega-migrated-$key"
    if ($MergePipelineHistory) {
        Merge-EarlyPipelines $sourcePath $targetPath $key
    }
    if (Test-Path -LiteralPath $marker) { return }

    $null = [IO.Directory]::CreateDirectory($targetPath)
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($sourcePath)
    while ($pending.Count) {
        $directory = $pending.Pop()
        foreach ($entry in Get-ChildItem -LiteralPath $directory -Force) {
            if (($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $entry.Name -like '.omega-migrated-*') { continue }
            $target = Join-Path $targetPath $entry.FullName.Substring($sourcePath.Length + 1)
            if ($entry.PSIsContainer) {
                $null = [IO.Directory]::CreateDirectory($target)
                $pending.Push($entry.FullName)
                continue
            }
            if ([IO.File]::Exists($target)) { continue }
            $temporary = Join-Path ([IO.Path]::GetDirectoryName($target)) ('.omega-copy-' + [Guid]::NewGuid())
            try {
                [IO.File]::Copy($entry.FullName, $temporary)
                try { [IO.File]::Move($temporary, $target) }
                catch [IO.IOException] { if (![IO.File]::Exists($target)) { throw } }
            } finally {
                [IO.File]::Delete($temporary)
            }
        }
    }
    [IO.File]::WriteAllText($marker, '')
}

function Merge-EarlyPipelines([string]$Source, [string]$Destination, [string]$Key) {
    # Older migrations preserved the AppData list when both caches already existed.
    $relative = 'pipelines/early-pipelines.txt'
    $legacyList = Join-Path $Source $relative
    $activeList = Join-Path $Destination $relative
    $marker = Join-Path $Destination ".omega-migrated-pipelines-$Key"
    if ((Test-Path -LiteralPath $marker) -or
        ![IO.File]::Exists($activeList) -or ![IO.File]::Exists($legacyList)) { return }

    $keys = [Collections.Generic.SortedSet[string]]::new([StringComparer]::Ordinal)
    foreach ($list in $activeList, $legacyList) {
        $folder = Get-Item -LiteralPath (Split-Path -Parent $list) -Force
        $file = Get-Item -LiteralPath $list -Force
        if (($folder.Attributes -bor $file.Attributes) -band [IO.FileAttributes]::ReparsePoint) { return }
        foreach ($line in [IO.File]::ReadLines($list)) {
            if ($line.Length) { $null = $keys.Add($line) }
        }
    }
    $temporary = Join-Path (Split-Path -Parent $activeList) ('.omega-copy-' + [Guid]::NewGuid())
    try {
        [IO.File]::WriteAllLines($temporary, [string[]]$keys, [Text.UTF8Encoding]::new($false))
        [IO.File]::Replace($temporary, $activeList, [NullString]::Value)
    } finally {
        [IO.File]::Delete($temporary)
    }
    [IO.File]::WriteAllText($marker, '')
}

if ($DataDirectory) { Copy-LegacyTree (Join-Path $legacy 'data') $DataDirectory }
if ($CacheDirectory) { Copy-LegacyTree (Join-Path $legacy 'cache/cache-coolfox') $CacheDirectory -MergePipelineHistory }
