# Victory Omega Fork

<img src="crates/victory-gpu/assets/omega.png" alt="Victory Omega logo" width="160">

## Run Victory Omega on Windows

Victory Omega targets NVIDIA GeForce RTX 50 Series. The RTX 5060 Ti 16 GB is the local Windows test card. To build, install Git, Rust via rustup, Visual Studio Build Tools with **Desktop development with C++** and a Windows SDK, and the .NET 10 SDK. Run these commands in PowerShell:

```powershell
git clone --branch Main https://github.com/NitricAcidDev/VIctory-Omega-Fork.git
Set-Location VIctory-Omega-Fork
.\Build-Victory.ps1 -Diagnostics
Start-Process '.\Omega Launcher.lnk'
```

In Omega Launcher, browse to your own complete Xbox game folder and select **Launch game**. The default profile is Series S at 1080p with ray tracing off. F3 toggles the performance overlay in a diagnostics build. Alt+Enter toggles borderless fullscreen. Windows x64, an AVX2 CPU, and a Direct3D 12 GPU with shader model 6.6 and mesh shader support are required.

The launcher saves your Home and Mods settings when you launch or close it. The in-game F1 menu has its own **Save for Next Start** action.

`Build-Victory.ps1 -Diagnostics` builds the game in `target\diag` and the launcher in `target\omega-launcher`. Omit `-Diagnostics` for a release build. To put the shortcut in an existing Victory folder, add `-ShortcutDirectory 'E:\Victory'`.

**Shadow boost** is off by default. Turn it on to lift the darkest areas of the diffuse lighting with ray tracing off. **Original graphics** bypasses the lighting and reflection corrections. **Low-memory mode** uses smaller buffer-copy windows.

### Migrate an existing installation

Close Victory normally and run the migrator from this checkout. Use the old Victory folder that contains `data`.

```powershell
.\scripts\Migrate-VictoryStorage.ps1 -LegacyRoot 'E:\Victory'
```

The migrator copies missing files into `%LOCALAPPDATA%\victory\data` and `%LOCALAPPDATA%\victory\cache`, merges the early pipeline list, and keeps the old folders as a backup. It uses the old `cache\cache-coolfox` directory when present and the old `cache` directory otherwise. The launcher also runs it when the shortcut opens from a recognized legacy root. PowerShell users can pass `-LegacyRoot` to `play.ps1`. Explicit storage directories take precedence for the default Omega runner. The NVIDIA runner uses its own compiled cache and the same saves.

### NVIDIA fork features and Omega support

Omega Launcher provides the frame limit and audio tracing on its main screen. Select the NVIDIA runner to enable the DLSS and Frame Generation controls. **F1** opens its graphics menu in game. Press **F11** to toggle borderless fullscreen immediately, or change **Fullscreen (F11)** in the F1 menu. Use **Save** in the menu to keep the setting for the next launch.

With the NVIDIA runner set to **15 FPS**, light frames such as menus and intro videos target **30 FPS**. The runner uses draw count to distinguish those frames from 3D gameplay, so menus that keep drawing the world may still follow the gameplay cap. Video playback keeps its source cadence.

Build the optional NVIDIA runner and install Streamline once:

```powershell
.\scripts\Build-OmegaNvidia.ps1
& "$env:LOCALAPPDATA\victory\nvidia-runner\omega-options-v1\victory.exe" get-dlss
```

Reopen Omega Launcher, check **Use NVIDIA runner**, and select **Launch game**. You can also launch it directly with the command below. The build script uses the pinned release source. [VIctory-NVIDIA releases](https://github.com/trustno1-coder/VIctory-NVIDIA/releases)

```powershell
.\scripts\play.ps1 -Exe 'D:\Games\MyDump\game_scarlett_master.exe' -Console series-s -Nvidia -Dlss quality -FrameGeneration 0
```

For a fresh NVIDIA launch, Omega Launcher applies the console, ray tracing, graphics, low-memory and diagnostics choices. Game default uses 30 FPS. Original graphics uses the game's upscaler and lighting. F1 still saves its settings and can restart the game when requested. The launcher skips the automatic first-run restart and gives streamed textures more time to catch up when the queue is large. Series X defaults to 1440p in the NVIDIA runner. [RTX 5060 Ti performance report](docs/performance-nvidia.md)

For NVIDIA shader precompilation with the selected launch profile, add `-PreloadShaders` to the command above. It uses the NVIDIA runner and its cache, then starts the game. The **Preload Shaders** action in F1 uses the same NVIDIA preloader.

**Unified tuning** on Home lets the NVIDIA runner use up to 1 GiB of Resizable BAR upload memory and six draw-recording workers. It is off by default and unavailable with Low-memory mode. PowerShell users can pass `-UnifiedTuning` to `play.ps1`. To try whole-program optimization, add `-OptimizedBuild` to `Build-OmegaNvidia.ps1`. This build takes longer. See the [comparison notes](docs/performance-nvidia.md).

**Water Light** starts off in Omega Launcher because the experimental tiled passes can cause dark or green blocks on NVIDIA. The F1 menu also offers **Transmission Only** to compare the base water shading with tiled local lighting disabled. Save and restart to apply it. [NVIDIA settings reference](docs/nvidia-settings.md)

### ScriptHook VI trainer

Install CMake 3.24 or newer and the Visual Studio C++ build tools with MASM. Build the NVIDIA runner and install the trainer beside your Xbox game executable:

```powershell
.\scripts\Build-OmegaNvidia.ps1 -TrainerGameDirectory '<Xbox game folder>'
```

The trainer starts with **Status line** off. To show the position, heading, and speed again, enable it in the trainer's **Settings** menu.

The build installs the pinned ScriptHook VI revision when the trainer files are missing. In Omega Launcher, select **Use NVIDIA runner** on **Home**, then **Enable trainer** on **Mods**. In offline story gameplay, press **F4** to open the trainer. Use the arrow keys to navigate, Enter to select and Backspace to go back. In **F1**, set **Game Keyboard** to Off to stop Victory from sending keyboard input to the game and its emulated controller. This applies live. ScriptHook VI has its own keyboard handler. The trainer targets Xbox build 1.0.2.1. [ScriptHook VI](https://github.com/modpapi/scripthook-vi)

A fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer
that runs an Xbox Series title natively on Windows from files the user supplies.

## The goal

**RTX 5060 Ti 16 GB:** Reported story gameplay ranges from **15–25 FPS** depending on the scene. [Performance report](docs/performance-nvidia.md)

**The first milestone is a stable 15 FPS at 1080p in story on NVIDIA GeForce RTX 50 Series, with console-like rendering.** The longer-term target is 30 FPS. Ray tracing is experimental. A warmed Series X ray-tracing run on the RTX 5060 Ti measured 2.3 FPS median indoors.

The way there is **porting, not emulating**: patching the title's renderer so it
behaves the way a PC renderer does, rather than having the layer watch and copy
the console's shared memory every frame. Shaders are translated ahead of time
into a versioned store that is kept between builds. Ray tracing stays optional
and off by default. The picture comes after the frame rate: texture pop-in and
lighting that is too bright or too dark when walking indoors are next in line.

**This repository contains no game files, code or assets.** You need your own
copy of the game. It is not affiliated with or endorsed by the game's developer
or publisher.

## Upstream AMD benchmark (2026-11-26)

Tested on one machine only: Windows 11, Intel i9-12900K, 64 GB RAM, Radeon
RX 7900 XTX, the Series X profile, ray tracing off.

| | |
|---|---|
| City street, renderer patches 1+2 on | about 53 ms a frame (about 19 fps) |
| Same scene before the patches | 57-75 ms (13-17 fps) |
| Shaders the title uses that translate | all of them (25,790 of 25,985; the rest are unused geometry shaders) |
| Memory | about 35 GB of private memory: **64 GB of RAM recommended** for now |

It started at 280 ms a frame in the same scenes three days earlier.

### Renderer patches (off by default while they are tested)

| Switch | What it does |
|---|---|
| `VICTORY_PORT_CONSTANTS=1` | The title's 174 MB constants heap is read by the card where the title writes it: no uploads, no fills |
| `VICTORY_PORT_DESCRIPTORS=1` | The title's descriptor-table stores are patched (byte-checked) to report the slots they write, instead of a scan of ~600,000 slots a frame |

What is known to be wrong:

- Trees, signs and far objects flicker in the distance in some places.
- Lighting is too bright or too dark after walking indoors, until the title has
  run for a while; rooms with little direct light are too dark without `-RayTracing`.
- Textures of what just came into view appear late while the title streams.
- The first run of a new build stutters while the driver makes pipelines
  (`victory warm` makes them ahead).
- Recording a frame in the world makes the title's own watchdog end it.

## Launch from PowerShell

```powershell
.\scripts\play.ps1 -Exe 'D:\Games\MyDump\game_scarlett_master.exe' -Console series-s -Build diag -PerformanceLog
```

Use `-Build release` if you built without `-Diagnostics`. Use `-Safe` to try the more conservative submission settings, or `-RayTracing` to test experimental ray-traced indirect light. `-Console series-x` selects the higher-resolution profile.

Before the first run, ahead of time (both can be stopped and go on later):

```powershell
.\target\diag\victory.exe shaders 'D:\Games\MyDump\shaders.rpf' 'D:\Games\MyDump\game_scarlett_master.exe'
.\target\diag\victory.exe warm --used
```

Commands, debugging switches and upstream's own description:
[docs/README-fxceless.md](docs/README-fxceless.md). Every `VICTORY_...` switch is
described in a comment where it is read.

## What is different here

In short; the reasons and the measurements are in [docs/findings.md](docs/findings.md).

- **List calls are written down, not carried out, on the title's threads**
  (`VICTORY_THIN`): eight threads of ours translate them.
- **Submissions are prepared by a thread of ours and joined**
  (`VICTORY_ASYNC_SUBMIT`, `VICTORY_JOIN_PARTS`), with what can be spread over
  threads spread: refreshes, compares (AVX2), descriptor records, landings.
- **What shaders wrote comes back beside the write guards**, by the dwords that
  changed, on a thread of its own (`VICTORY_RELAY`, `VICTORY_LAND`).
- **Buffer fills that need bytes still on the card are done on the card**
  (`VICTORY_CARD_STEPS`) instead of waiting for it.
- **The title's writes are caught more cheaply**: one view opened at a fault,
  blocks that rest, places in the title's code that look before they write.
- **Textures go up on a copy queue of their own** and a slot shows one only after
  its upload ran, which also ended a device loss that came every 15 minutes or so.
- **Boats float**: textures that shaders write and the title's CPU reads go back
  to title memory (from fxceless and Geckomacabre, on by default here).
- **Frame record and replay** with per-pass times and shader variants compared
  turn by turn ([docs/replay.md](docs/replay.md)).

## Credits

This is a tree of other people's work with more on top. The history is kept, and
commits taken from elsewhere name the commit and its author.

- **[VIctory](https://github.com/Marvisak/VIctory)** by Marvisak and its
  contributors: the loader, the runtime services, the Direct3D 12 back end and
  the shader recompiler.
- **[fxceless/victory-fork](https://github.com/fxceless/victory-fork)**: the fork
  this one started from, and the people whose pull requests it carries.
- **[DexNom/VIctory](https://github.com/DexNom/VIctory)**: ray tracing fixes,
  cube gradients, sparse textures.
- **[Belobug/VIctory-Belobug-AMD](https://github.com/Belobug/VIctory-Belobug-AMD)**:
  the ambient light idea, pipeline workers, and work by romainhedouin.
- **[FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)**.
- **[myokex/victory-amd](https://github.com/myokex/victory-amd)**: the guard over
  every view of a page (with Spexivus, joroyt and tightfall), and Joshua Odom's
  reads into title memory.
- **Geckomacabre** and **cxy0072019-sketch**: the texture read-back.
- **[trustno1-coder/VIctory-NVIDIA](https://github.com/trustno1-coder/VIctory-NVIDIA)**
  and its contributors supplied the optional NVIDIA runner, F1 graphics menu,
  and NVIDIA shader and pipeline preloader.
- **[Keel62155/VIctoryUnified](https://github.com/Keel62155/VIctoryUnified)**
  published the Resizable BAR and worker-count tuning results and tested the
  whole-program build. Its notes also identified fxceless's pipeline panic
  recovery from pull request #1353.
- **[modpapi/ScriptHook VI](https://github.com/modpapi/scripthook-vi)** supplied
  the hosted startup interface and optional offline story trainer.
- **[NVIDIA Streamline](https://github.com/NVIDIA-RTX/Streamline)** provides the
  NVIDIA runner's DLSS and Frame Generation integration.
- **[AMD FidelityFX SDK](https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK)**
  supplies the optional FSR 3 Frame Generation path in the NVIDIA runner.
- **[dashdogy/MFG Unlocker](https://github.com/dashdogy/MFG-Unlocker)** is the
  third-party mod offered by the NVIDIA runner's optional MFG Unlocker setting.
- **[XCL](https://github.com/ivall/xcl)** (GPL-2.0-only) was read for how the same
  title behaves under another layer; no code of it is here.
- This fork's changes were made with [Claude Code](https://claude.com/claude-code).

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE); the one file that is not
covered is named in [docs/asset-policy.md](docs/asset-policy.md).
