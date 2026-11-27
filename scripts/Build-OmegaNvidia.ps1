# Build the NVIDIA runner with Omega window branding. The checkout and binary stay outside the repo.
param(
    [string]$SourceDir = (Join-Path $env:LOCALAPPDATA 'victory/nvidia-source'),
    [string]$TrainerGameDirectory
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

function Update-NvidiaSource([string]$Path, [scriptblock]$Change, [scriptblock]$Previous, [string[]]$PreviousHash) {
    $file = Join-Path $SourceDir $Path
    $original = ((git -C $SourceDir show "${commit}:$Path") -join [char]10).TrimEnd("`n")
    if ($LASTEXITCODE) { throw "Could not read $Path at $commit." }
    $updated = & $Change $original
    if ($updated -eq $original) { throw "The Omega branding change did not match $Path." }
    $current = ([IO.File]::ReadAllText($file) -replace '\r\n', "`n").TrimEnd("`n")
    if ($current -ne $original -and $current -ne $updated -and (!$Previous -or $current -ne (& $Previous $original))) {
        if ((Get-NvidiaSourceHash $current) -notin $PreviousHash) {
            throw "Preserving local modifications in $file."
        }
    }
    [IO.File]::WriteAllText($file, $updated + [char]10, [Text.UTF8Encoding]::new($false))
}

function Get-NvidiaSourceHash([string]$Text) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))).Replace('-', '') }
    finally { $sha.Dispose() }
}

$gpu = Join-Path $SourceDir 'crates/victory-gpu'
$null = [IO.Directory]::CreateDirectory((Join-Path $gpu 'assets'))
Copy-Item -LiteralPath (Join-Path $repo 'crates/victory-gpu/assets/omega.png') -Destination (Join-Path $gpu 'assets/omega.png')
Copy-Item -LiteralPath (Join-Path $repo 'crates/victory-gpu/src/window.rs') -Destination (Join-Path $gpu 'src/omega_window.rs')

Update-NvidiaSource 'crates/victory-gpu/Cargo.toml' {
    param($text)
    $text.Replace('features = ["Win32_Foundation", "Win32_Security"', 'features = ["Win32_Foundation", "Win32_Graphics_Gdi", "Win32_Security"')
}

Update-NvidiaSource 'crates/victory-gpu/src/lib.rs' {
    param($text)
    $text.Replace('mod host;', "mod host;`n#[cfg(windows)]`npub mod omega_window;")
}
$brandTitleWindow = {
    param($text)
    $text.Replace("    hook(hwnd);`n    hwnd", "    if !hwnd.is_null() {`n        victory_gpu::omega_window::brand(hwnd);`n    }`n    hook(hwnd);`n    hwnd")
}
Update-NvidiaSource 'crates/victory-hle/src/modules/lifecycle.rs' {
    param($text)
    $text = & $brandTitleWindow $text
    $text = $text -replace '(?s)fn fullscreen\(hwnd: HWND\) \{.*?\n\}\n', @'
fn fullscreen(hwnd: HWND) {
    victory_gpu::omega_window::set_fullscreen(hwnd, enabled("VICTORY_FULLSCREEN"));
}

'@
    $text.Replace('    if message == wm::WM_SETFOCUS {', @'
    if victory_gpu::omega_window::handle_fullscreen_message(hwnd, message, wparam) {
        return 0;
    }
    if message == wm::WM_SETFOCUS {
'@)
} -Previous $brandTitleWindow
$brandHostWindow = {
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
Update-NvidiaSource 'crates/victory-gpu/src/host.rs' {
    param($text)
    $text = & $brandHostWindow $text
    $text = $text.Replace('            p.menu.frame();', '            p.menu.frame(p.hwnd.0);')
    $text = $text.Replace('    // SAFETY: the default handling for our window.', @'
    if crate::omega_window::handle_fullscreen_message(hwnd.0, message, wparam.0) {
        return LRESULT(0);
    }
    // SAFETY: the default handling for our window.
'@)
    $text.Replace('        crate::omega_window::brand(hwnd.0);', @'
        crate::omega_window::brand(hwnd.0);
        let fullscreen = std::env::var_os("VICTORY_FULLSCREEN").is_some_and(|value| value == "1");
        crate::omega_window::set_fullscreen(hwnd.0, fullscreen);
'@)
} -Previous $brandHostWindow
$graphicsMenuBranding = {
    param($text)
    $text.Replace('SETTINGS                       F1 OR ESC CLOSES', 'OMEGA GRAPHICS                 F1 OR ESC CLOSES')
}
$frameRateMenu = {
    param($text)
    $text = & $graphicsMenuBranding $text
    $cap = '("30", "30 (THE GAME)")], live: false },'
    if (!$text.Contains($cap)) { throw 'Could not find the NVIDIA frame-rate cap choices.' }
    $text.Replace($cap, '("30", "30 (THE GAME)"), ("15", "15 FPS")], live: false },')
}
$fullscreenMenu = {
    param($text)
    $text = & $frameRateMenu $text
    $text = $text.Replace('VK_ESCAPE, VK_F1, VK_LEFT', 'VK_ESCAPE, VK_F1, VK_F11, VK_LEFT')
    $text = $text.Replace('[u16; 7] = [VK_F1, VK_UP', '[u16; 8] = [VK_F1, VK_F11, VK_UP')
    $text = $text.Replace('var: "VICTORY_FULLSCREEN", label: "FULLSCREEN (NO BORDER)", values: UNSET_ON, live: false',
        'var: "VICTORY_FULLSCREEN", label: "FULLSCREEN (F11)", values: UNSET_ON, live: true')
    $text = $text.Replace('pub(super) fn frame(&mut self) {',
        'pub(super) fn frame(&mut self, hwnd: windows_sys::Win32::Foundation::HWND) {')
    $text = $text.Replace("        for vk in pressed {`n            let open = Self::open();", @'
        let fullscreen = ITEMS.iter().position(|item| item.var == "VICTORY_FULLSCREEN").unwrap();
        let previous_fullscreen = self.values[fullscreen].clone();
        for vk in pressed {
            if vk == VK_F11 {
                let step = if self.values[fullscreen] == "1" { 1 } else { -1 };
                self.change(fullscreen, step);
                self.changed = true;
                continue;
            }
            let open = Self::open();
'@)
    $text.Replace(@'
            self.changed = true;
        }
    }

    /// Records the copy of the box onto
'@, @'
            self.changed = true;
        }
        let enabled = self.values[fullscreen] == "1";
        if enabled != (previous_fullscreen == "1")
            && !crate::omega_window::request_fullscreen(hwnd, enabled)
        {
            self.values[fullscreen] = previous_fullscreen;
            self.apply_live(fullscreen);
            self.note = "COULD NOT CHANGE FULLSCREEN".to_owned();
        }
    }

    /// Records the copy of the box onto
'@)
}
Update-NvidiaSource 'crates/victory-gpu/src/host/menu.rs' {
    param($text)
    $text = & $fullscreenMenu $text
    $text.Replace('    // The layer''s switches.', @'
    Item { var: "VICTORY_GAME_KEYBOARD", label: "GAME KEYBOARD", values: &[("", "ON"), ("0", "OFF")], live: true },
    // The layer's switches.
'@)
} -Previous $fullscreenMenu -PreviousHash @(
    '91A2717E47DD1C717E6FF134181A7087BAE6A32F0FCA3035E58DE17D79A544DB',
    'DE032347BC0ADD20E740B2C86D4C2553305A5ACD1751024678BA38F3312A441C'
)
Update-NvidiaSource 'crates/victory-hle/src/modules/gameinput.rs' {
    param($text)
    $check = '            if victory_gpu::SETTINGS_MENU_OPEN.load(std::sync::atomic::Ordering::Relaxed) {'
    if (!$text.Contains($check)) { throw 'Could not find the NVIDIA game keyboard input gate.' }
    $text.Replace($check, @'
            if victory_gpu::SETTINGS_MENU_OPEN.load(std::sync::atomic::Ordering::Relaxed)
                || std::env::var_os("VICTORY_GAME_KEYBOARD").is_some_and(|value| value == "0")
            {
'@)
}
Update-NvidiaSource 'crates/victory-gpu/src/texture.rs' {
    param($text)
    $original = @'
        let read = with_read(size, |read| {
            if !crate::host::read_into(self.memory, read) {
                return false;
            }
'@
    $start = $text.IndexOf('    fn refresh_unlocked(')
    $at = $text.IndexOf($original, $start)
    if ($at -lt 0) { throw 'Could not find the NVIDIA unlocked texture refresh.' }
    $retry = @'
        let read = with_read(size, |read| {
            // Retry a cold, small upload while title memory is being remapped.
            // Keep the usual single read for warm or large textures.
            let mut complete = crate::host::read_into(self.memory, read);
            if !complete && last.hash.is_none() && size <= 256 * 1024 {
                for _ in 0..2 {
                    std::thread::yield_now();
                    complete = crate::host::read_into(self.memory, read);
                    if complete { break; }
                }
            }
            if !complete {
                return false;
            }
'@
    $text.Remove($at, $original.Length).Insert($at, $retry)
}
Update-NvidiaSource 'crates/victory-gpu/src/frame.rs' {
    param($text)
    $text = $text.Replace(
        'const SIXTIETH: Duration = Duration::from_micros(16_667);',
        @'
const SIXTIETH: Duration = Duration::from_micros(16_667);
const THIRTIETH: Duration = Duration::from_micros(33_333);
// The existing capture trigger uses 500 draws to recognize a 3D scene.
#[cfg(windows)]
const WORLD_DRAWS: u64 = 500;

#[cfg(windows)]
pub(crate) fn note_present(drawn: u64) {
    if !menu_floor_enabled() {
        return;
    }
    let mut clock = clock();
    let Some(c) = clock.as_mut() else { return };
    let draws = drawn.saturating_sub(c.last_drawn);
    c.last_drawn = drawn;
    // Two light frames avoid switching pace on a single fade or dropped frame.
    c.light_frames = if draws >= WORLD_DRAWS { 0 } else { (c.light_frames + 1).min(2) };
}

fn menu_floor_enabled() -> bool {
    asked().is_none() && interval_override().is_some_and(|interval| interval > THIRTIETH)
}
'@)
    $text = $text.Replace(
        '    interval: Duration,',
        "    interval: Duration,`n    requested: Duration,`n    last_drawn: u64,`n    light_frames: u8,")
    $text = $text.Replace(
        "        interval,`n        first,",
        "        interval,`n        requested: interval,`n        last_drawn: 0,`n        light_frames: 2,`n        first,")
    $before = @'
    let c = clock.get_or_insert_with(|| Clock {
        start: Instant::now(),
        interval: asked().or_else(interval_override).unwrap_or(SIXTIETH),
        first: 0,
    });
'@
    if (!$text.Contains($before)) { throw 'Could not find the NVIDIA frame clock.' }
    $text.Replace($before, @'
    let c = clock.get_or_insert_with(|| {
        let requested = asked().or_else(interval_override).unwrap_or(SIXTIETH);
        Clock {
            start: Instant::now(),
            interval: requested,
            requested,
            last_drawn: 0,
            light_frames: 2,
            first: 0,
        }
    });
    if menu_floor_enabled() {
        let interval = if c.light_frames == 2 {
            c.requested.min(THIRTIETH)
        } else {
            c.requested
        };
        if interval != c.interval {
            let last_tick = LAST.iter().map(|tick| tick.load(Ordering::Relaxed)).max().unwrap_or(0);
            // Keep tick IDs monotonic and measure the next wait from the last tick.
            let now = Instant::now();
            let elapsed = now.duration_since(c.start);
            let since_tick = if last_tick > c.first {
                let tick_ns = u128::from(last_tick - c.first) * c.interval.as_nanos();
                Duration::from_nanos(elapsed.as_nanos().saturating_sub(tick_ns).min(u128::from(u64::MAX)) as u64)
            } else {
                elapsed
            };
            c.start = now - since_tick.min(interval);
            c.first = last_tick;
            c.interval = interval;
        }
    }
'@)
}
Update-NvidiaSource 'crates/victory-gpu/src/queue.rs' {
    param($text)
    $present = '    let frame = FRAMES.fetch_add(1, Ordering::Relaxed) + 1;'
    if (!$text.Contains($present)) { throw 'Could not find the NVIDIA present hook.' }
    $text.Replace($present, "$present`n    #[cfg(windows)]`n    crate::frame::note_present(crate::draw::drawn());")
}
Update-NvidiaSource 'crates/victory-gpu/src/ambient.rs' {
    param($text)
    $luma = 'static const float3 LUMA = float3(0.2126, 0.7152, 0.0722);'
    $text = $text.Replace($luma, $luma + "`n" + @'

// Give reused probe color a small allowance in every channel and more
// where nearby scene radiance supports it. Neutral light keeps its value.
float3 bound_probe_color(float3 sample, float3 local_light, float gain) {
    float neutral = min(sample.r, min(sample.g, sample.b));
    float local_neutral = min(local_light.r, min(local_light.g, local_light.b));
    float3 allowance = 0.25 * dot(local_light, LUMA) + gain * (local_light - local_neutral);
    return min(sample, neutral + allowance);
}
'@)
    $text = $text.Replace('    float cap = air * lerp(dot(cells[CELLS].rgb, LUMA), dot(seen_there, LUMA), far);', @'
    float3 local_light = air * max(lerp(cells[CELLS].rgb, seen_there, far), 0.0);
    float cap = dot(local_light, LUMA);
'@)
    $text = $text.Replace('    if (has > cap) scattered[at] = c * (cap / has);', @'
    float3 held = c * min(1.0, cap / max(has, 1e-6));
    held = bound_probe_color(held, local_light, 1.0);
    if (any(held < c)) scattered[at] = held;
'@)
    $text = $text.Replace('    reflected[at.xy] = lerp(found, was, constant);', '    reflected[at.xy] = lerp(found, bound_probe_color(was, found, 1.0), constant);')
    $text = $text -replace '(?s)    // Magenta/violet garbage the probes.*?(?=    // What the program left counts)', ''
    $text.Replace('    float3 now = max(was, found);', '    float3 now = max(bound_probe_color(was, found, most), found);')
}
$omegaAudio = (Get-Content -LiteralPath (Join-Path $repo 'crates/victory-hle/src/modules/audio.rs') -Raw) -replace '\r\n', "`n"
Update-NvidiaSource 'crates/victory-hle/src/modules/audio.rs' {
    param($text)
    # The two runners share this XMA context layout and streaming protocol.
    $context = '(?s)impl Context \{.*?(?=\nimpl Xma)'
    $shared = [regex]::Match($omegaAudio, $context)
    if (!$shared.Success -or ![regex]::IsMatch($text, $context)) { throw 'Could not find the shared XMA context.' }
    $text = [regex]::Replace($text, $context, [Text.RegularExpressions.MatchEvaluator]{ param($match) $shared.Value })
    $text = $text.Replace('use std::ffi::c_void;', "use std::ffi::c_void;`nuse std::sync::atomic::{AtomicU32, Ordering};")
    $text = $text.Replace("            context.set_field(2, 26, 5, INPUT_INVALID);`n            context.set_field(2, 31, 1, 1);", '            context.set_field(2, 26, 6, INPUT_INVALID | 32);')
    $text = $text.Replace('    if next == 0 {', '    if next == 0 && !looping {')
    $endInput = "        context.set_field(0, 20 + current, 1, 0);`n    }`n    context.set_field(2, 0, 26, next as u32);"
    if (!$text.Contains($endInput)) { throw 'Could not find the shared XMA input handoff.' }
    $text = $text.Replace($endInput, "        context.finish_input(current);`n    } else {`n        context.set_field(2, 0, 26, next as u32);`n    }")
    $text = $text.Replace("        context.set_field(0, 20 + current, 1, 0);`n        return true;", "        context.finish_input(current);`n        return true;")
    $text = $text.Replace('        context.set(3, first);', '        context.set(3, 0);')
    $text.Replace('assert_eq!(step(first), 2 * subframe, "the loop', 'assert_eq!(step(0), 2 * subframe, "the loop')
} -PreviousHash 'E884E10E0D509FC43F385EE74DFE7D6A48BE0F7568469C9DFBED50E00BF35EDE'
Update-NvidiaSource 'crates/victory-core/src/logging.rs' {
    param($text)
    $text.Replace('.with(CAP)', @'
.with(if std::env::var_os("VICTORY_OMEGA_AUDIO_TRACE").is_some_and(|value| value == "1") {
            LevelFilter::DEBUG
        } else {
            CAP
        })
'@)
}
$launcherOptions = {
    param($text)
    $text.Replace(
        '    let menu = parse_menu(&std::fs::read_to_string(&menu_file).unwrap_or_default());',
        @'
    let mut menu = parse_menu(&std::fs::read_to_string(&menu_file).unwrap_or_default());
    // Launcher choices apply once. An F1 restart reads its own saved choices.
    if let Ok(options) = std::env::var("VICTORY_OMEGA_LAUNCH") {
        menu.extend(parse_menu(&options));
    }
'@)
}
Update-NvidiaSource 'apps/victory/src/play.rs' {
    param($text)
    $text = & $launcherOptions $text
    $text.Replace('    if !s.contains_key("VICTORY_WARMUP_RESTART")', '    if s.get("VICTORY_WARMUP_RESTART").is_some_and(|value| value != "0")')
} $launcherOptions

Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'omega-scripthook-host.rs') -Destination (Join-Path $SourceDir 'apps/victory/src/omega_scripthook_host.rs')
Update-NvidiaSource 'apps/victory/src/run.rs' {
    param($text)
    $entry = '    let main = unsafe { Arc::new(image).spawn_main(dlls)? };'
    if (!$text.Contains($entry)) { throw 'Could not find the NVIDIA title entry point.' }
    $text = "#[cfg(windows)]`n#[path = `"omega_scripthook_host.rs`"]`nmod omega_scripthook_host;`n`n" + $text
    $text.Replace($entry, "    omega_scripthook_host::start(image.base(), image.info().size_of_image as u64, exe);`n$entry")
}

Push-Location $SourceDir
try {
    cargo build --release --locked -p victory
    if ($LASTEXITCODE) { throw 'The Omega NVIDIA runner build failed.' }
} finally { Pop-Location }

# Install beside older builds so an active game can finish its session.
$target = Join-Path $env:LOCALAPPDATA 'victory/nvidia-runner/omega-options-v1'
$null = [IO.Directory]::CreateDirectory($target)
$active = Join-Path $target 'victory.exe'
$pending = Join-Path $target 'victory.next'
$running = Get-Process victory -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $active }
$install = if ($running) { $pending } else { $active }
$stage = Join-Path $target ('.omega-install-' + [Guid]::NewGuid())
try {
    [IO.File]::Copy((Join-Path $SourceDir 'target/release/victory.exe'), $stage)
    if ([IO.File]::Exists($install)) {
        [IO.File]::Replace($stage, $install, [NullString]::Value)
    } else {
        [IO.File]::Move($stage, $install)
    }
    if ($running) { Write-Output 'The update will be installed at the next launch.' }
    else { [IO.File]::Delete($pending) }
} finally { [IO.File]::Delete($stage) }
[IO.File]::WriteAllText((Join-Path $target 'omega-branding-v1'), $commit)
if ($TrainerGameDirectory) {
    $trainerRoot = Join-Path $TrainerGameDirectory 'shvi'
    if (!(Test-Path -LiteralPath (Join-Path $trainerRoot 'shvi_core.dll') -PathType Leaf) -or
        !(Test-Path -LiteralPath (Join-Path $trainerRoot 'plugins/trainer.asi') -PathType Leaf) -or
        !(Test-Path -LiteralPath (Join-Path $trainerRoot 'natives/1794606036.json') -PathType Leaf)) {
        & (Join-Path $PSScriptRoot 'Install-ScriptHookVI.ps1') -GameDirectory $TrainerGameDirectory
    }
}
