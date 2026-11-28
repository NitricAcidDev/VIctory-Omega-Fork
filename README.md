# Victory Omega Fork

<img src="crates/victory-gpu/assets/omega.png" alt="Victory Omega logo" width="160">

Victory Omega is a fork of [VIctory](https://github.com/Marvisak/VIctory), a compatibility layer for running an Xbox Series title on Windows from files the user supplies.

This project is focused on NVIDIA hardware. Development and local testing currently target **NVIDIA GeForce RTX 50 Series GPUs**; the RTX 5060 Ti is the primary local test card. AMD hardware support is a future goal and is not a focus of current testing or compatibility claims.

> **This repository contains no game files, code, or assets from the title.** You need your own complete, legally obtained copy.

## Windows build

Build the game and Omega Launcher together:

```powershell
.\Build-Victory.ps1
```

The launcher build requires the .NET 10 SDK and runs on the .NET 10 Desktop Runtime.

Use `-ShortcutDirectory` to place the generated **Omega Launcher** shortcut elsewhere:

```powershell
.\Build-Victory.ps1 -ShortcutDirectory 'D:\Shortcuts'
```

Open the Omega Launcher, browse for the folder containing `game_scarlett_master.exe`, select a profile, and launch the game.

You can also build the game directly:

```powershell
cargo build --release --locked -p victory
```

The release executable is written to `target\release\victory.exe`.

## Requirements

- Windows 10 or 11 x64
- .NET 10 SDK and Desktop Runtime for the Omega Launcher
- Rust using the pinned toolchain
- MSVC C++ build tools and a Windows SDK
- A Direct3D 12 GPU with Shader Model 6.6 and mesh shader support
- A complete Xbox dump that you own

NVIDIA RTX 50 Series hardware is the supported and tested direction. Other Direct3D 12 GPUs may work, but no compatibility level is claimed until tested.

## Running

Set the game directory and start the selected residency profile:

```powershell
$env:VICTORY_GAME_DIR = 'D:\Games\MyDump'
.\Play-VIctory.cmd pareto-residency
```

The launcher builds and reuses a local shader cache under `%LOCALAPPDATA%\VIctory\cache-pareto`. Set `VICTORY_CACHE_DIR` to keep the cache outside the checkout:

```powershell
$env:VICTORY_CACHE_DIR = 'D:\VIctory\cache'
```

Both Series S and Series X profiles are available. Series S renders the 1080p frame and is the default for NVIDIA residency testing. Series X uses higher-resolution internal buffers.

Ray tracing is optional and off by default. Use `-RayTracing` only when testing ray-traced indirect light or geometry tracing.

## Development goals

- Stable 30 fps or more without ray tracing
- Higher frame rates and lower frame times on NVIDIA hardware
- Reliable texture residency and lower memory pressure
- Improved lighting, reflections, rendering coverage, and geometry stability
- Ray tracing remains optional and is not required for the primary experience
- AMD GPU support remains a future goal rather than a current testing priority

The implementation approach is porting, not emulation: patching the title's renderer so it behaves more like a PC renderer instead of copying the console's shared memory every frame.

## Diagnostics and shader cache

The first run can take longer while shaders and pipelines are compiled. Prepare commonly used shaders ahead of time when needed:

```powershell
target\diag\victory.exe shaders <shader archives>
target\diag\victory.exe warm --used
```

Every `VICTORY_...` debugging switch is described near where it is read in the source.

Implementation details and development findings are documented in:

- [Architecture](docs/architecture.md)
- [GPU replay and timing](docs/replay.md)
- [Ray tracing work](docs/fork-ray-tracing.md)
- [Findings](docs/findings.md)

The upstream and fork-specific notes remain available in [docs/README-fxceless.md](docs/README-fxceless.md).

## Credits

This repository combines other people's work with additional NVIDIA-focused development. Commit history records contributions taken from elsewhere.

- [VIctory](https://github.com/Marvisak/VIctory) and its contributors: loader, runtime services, Direct3D 12 back end, and shader recompiler
- [fxceless/victory-fork](https://github.com/fxceless/victory-fork): the fork from which this work started
- [DexNom/VIctory](https://github.com/DexNom/VIctory): ray tracing fixes and related rendering work
- [FQY55kai/ZJOU-VICTORY](https://github.com/FQY55kai/ZJOU-VICTORY)
- Geckomacabre and cxy0072019-sketch: texture read-back work
- [XCL](https://github.com/ivall/xcl): GPL-2.0-only reference implementation used for interoperability research
- This fork's changes were made with Claude Code and GitHub Copilot

## License

GPL-3.0-or-later, as VIctory. See [LICENSE](LICENSE). The one file not covered by that license is identified in [docs/asset-policy.md](docs/asset-policy.md).
