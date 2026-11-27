# Build the NVIDIA runner with Omega window branding. The checkout and binary stay outside the repo.
param(
    [string]$SourceDir = (Join-Path $env:LOCALAPPDATA 'victory/nvidia-source')
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path $PSScriptRoot -Parent
$sourceUrl = 'https://github.com/trustno1-coder/VIctory-NVIDIA.git'
$release = 'v2026.11.27.3'
$commit = '0cef07e9d766df48f0e95f13ce3bbbe2290fe9d3'

if (!(Test-Path -LiteralPath (Join-Path $SourceDir '.git'))) {
    git clone --depth 1 --branch $release -- $sourceUrl $SourceDir
    if ($LASTEXITCODE) { throw 'Could not clone the pinned NVIDIA source.' }
}
$revision = git -C $SourceDir rev-parse HEAD
if ($LASTEXITCODE -or $revision -ne $commit) { throw 'Expected the pinned VIctory-NVIDIA release checkout.' }
if (!(Test-Path -LiteralPath (Join-Path $SourceDir 'Cargo.toml'))) {
    git -C $SourceDir switch --detach $commit
    if ($LASTEXITCODE) { throw 'Could not check out the NVIDIA source.' }
}

function Update-NvidiaSource([string]$Path, [scriptblock]$Change) {
    $file = Join-Path $SourceDir $Path
    $original = ((git -C $SourceDir show "${commit}:$Path") -join [char]10).TrimEnd("`n")
    if ($LASTEXITCODE) { throw "Could not read $Path at $commit." }
    $updated = & $Change $original
    if ($updated -eq $original) { throw "The Omega branding change did not match $Path." }
    $current = ([IO.File]::ReadAllText($file) -replace '\r\n', "`n").TrimEnd("`n")
    if ($current -ne $original -and $current -ne $updated) { throw "Preserving local modifications in $file." }
    [IO.File]::WriteAllText($file, $updated + [char]10, [Text.UTF8Encoding]::new($false))
}

$branding = Get-Content -LiteralPath (Join-Path $repo 'crates/victory-gpu/src/window.rs') -Raw
$start = $branding.IndexOf("/// The game's desktop window name.")
$end = $branding.IndexOf('struct Windowed')
if ($start -lt 0 -or $end -le $start) { throw 'Could not find the Omega window branding code.' }
$header = 'use std::sync::OnceLock;', 'use windows_sys::Win32::Foundation::HWND;',
    'use windows_sys::Win32::UI::WindowsAndMessaging as wm;', ''
$module = ($header -join "`n") + "`n" + $branding.Substring($start, $end - $start).TrimEnd()
$gpu = Join-Path $SourceDir 'crates/victory-gpu'
$null = [IO.Directory]::CreateDirectory((Join-Path $gpu 'assets'))
Copy-Item -LiteralPath (Join-Path $repo 'crates/victory-gpu/assets/omega.png') -Destination (Join-Path $gpu 'assets/omega.png')
[IO.File]::WriteAllText((Join-Path $gpu 'src/omega_window.rs'), $module + "`n", [Text.UTF8Encoding]::new($false))

Update-NvidiaSource 'crates/victory-gpu/src/lib.rs' {
    param($text)
    $text.Replace('mod host;', "mod host;`n#[cfg(windows)]`npub mod omega_window;")
}
Update-NvidiaSource 'crates/victory-hle/src/modules/lifecycle.rs' {
    param($text)
    $text.Replace("    hook(hwnd);`n    hwnd", "    if !hwnd.is_null() {`n        victory_gpu::omega_window::brand(hwnd);`n    }`n    hook(hwnd);`n    hwnd")
}
Update-NvidiaSource 'crates/victory-gpu/src/host.rs' {
    param($text)
    $text = $text -replace '(?m)^    /// The caption the window had, once read\.\n    caption: Option<Vec<u16>>,\n', ''
    $text = $text -replace '(?m)^            caption: None,\n', ''
    $text = $text -replace '(?s)        if self\.caption\.is_none\(\) \{.*?        let Some\(caption\) = &self\.caption else \{ return \};\n', ''
    $text = $text.Replace(' | VIctory-NVIDIA ', ' | ')
    $text = $text.Replace('caption.iter().copied().chain(text.encode_utf16())', 'crate::omega_window::TITLE.encode_utf16().chain(text.encode_utf16())')
    $text = $text.Replace('w!("VIctory"),', 'w!("Victory | Omega"),')
    $text = $text.Replace("        CreateWindowExW(`n            WINDOW_EX_STYLE(0),`n            CLASS,", "        let hwnd = CreateWindowExW(`n            WINDOW_EX_STYLE(0),`n            CLASS,")
    $text.Replace("            Some(instance.into()),`n            None,`n        )", "            Some(instance.into()),`n            None,`n        )?;`n        crate::omega_window::brand(hwnd.0);`n        Ok(hwnd)")
}
Update-NvidiaSource 'crates/victory-gpu/src/host/menu.rs' {
    param($text)
    $text.Replace('SETTINGS                       F1 OR ESC CLOSES', 'OMEGA GRAPHICS                 F1 OR ESC CLOSES')
}

Push-Location $SourceDir
try {
    cargo build --release --locked -p victory
    if ($LASTEXITCODE) { throw 'The Omega NVIDIA runner build failed.' }
} finally { Pop-Location }

$target = Join-Path $env:LOCALAPPDATA 'victory/nvidia-runner'
$null = [IO.Directory]::CreateDirectory($target)
$active = Join-Path $target 'victory.exe'
$stage = Join-Path $target ('.omega-install-' + [Guid]::NewGuid())
try {
    [IO.File]::Copy((Join-Path $SourceDir 'target/release/victory.exe'), $stage)
    if ([IO.File]::Exists($active)) {
        [IO.File]::Replace($stage, $active, [NullString]::Value)
    } else {
        [IO.File]::Move($stage, $active)
    }
} finally { [IO.File]::Delete($stage) }
[IO.File]::WriteAllText((Join-Path $target 'omega-branding-v1'), $commit)
