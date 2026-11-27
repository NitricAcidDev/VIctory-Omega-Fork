# Working on Victory Omega

- Fetch the destination branch before starting and again before publishing. Check `git status` and `git worktree list` first. Preserve other contributors' changes and use a separate branch or worktree for concurrent edits.
- Commit each coherent change separately. Stage explicit files or hunks. Push each verified commit to `primary` with a normal push unless the owner directs another destination. If the remote advances, review the new commits and integrate them before retrying.
- For rendering work on NVIDIA, start with Series S at 1080p and ray tracing off unless the task asks for another profile. Test the same save, camera view, resolution, and cache state before attributing a change in performance to a patch.
- Report gameplay separately from menu FPS. Record frame time, draw drops, pipeline backlog, memory and transfer costs when available. Compare a stationary interior view and a camera pan when checking lighting, reflections, or streaming.
- Keep the original graphics option available when adding visual corrections. Prefer fixing shader translation, bindings, or resource lifetime when those explain the artifact.
- If Victory still uses the old `data` folder under its install root, tell the user that it needs to migrate. Close Victory normally before copying saves, then run `.\scripts\Migrate-VictoryStorage.ps1 -LegacyRoot '<old install root>'`. The script copies data and cache into `%LOCALAPPDATA%\victory` and preserves existing files. Verify the new location before launching again and keep the old folder as a backup.
- Follow [the asset policy](docs/asset-policy.md). Keep game files, extracted shaders, saves, captures, and raw logs outside the repository. Summary metrics and synthetic tests belong in the repository.

## Build and run

- Install the pinned Rust toolchain via rustup, Visual Studio C++ Build Tools with a Windows SDK, and the .NET 10 SDK. From the checkout root, run `.\Build-Victory.ps1 -Diagnostics` to build the runner and launcher and create `Omega Launcher.lnk`. Omit `-Diagnostics` for the release profile. Use `-ShortcutDirectory` for an existing install root.
- From PowerShell, run `.\scripts\play.ps1 -Exe '<game executable>' -Console series-s -Build diag -PerformanceLog` for diagnostics.
- For the optional NVIDIA runner, run `.\scripts\Build-OmegaNvidia.ps1`, then run `& "$env:LOCALAPPDATA\victory\nvidia-runner\omega-options-v1\victory.exe" get-dlss` to install Streamline. Check **Use NVIDIA runner** in Omega Launcher or add `-Nvidia` to `play.ps1`. Verify the console and effective settings in the NVIDIA play log before comparing runs.
- Pass `-TrainerGameDirectory '<Xbox game folder>'` to the NVIDIA build to install the pinned ScriptHook VI trainer if it is missing. Use `play.ps1 -Nvidia -PreloadShaders` with the selected game executable and launch options to precompile shaders into the NVIDIA cache before launch.
