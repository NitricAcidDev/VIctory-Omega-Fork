# Working on Victory Omega

- Fetch the destination branch before starting and again before publishing. Check `git status` and `git worktree list` first. Preserve other contributors' changes and use a separate branch or worktree for concurrent edits.
- Commit each coherent change separately. Stage explicit files or hunks. Push each verified commit to `primary` with a normal push unless the owner directs another destination. If the remote advances, review the new commits and integrate them before retrying.
- For rendering work on NVIDIA, start with Series S at 1080p and ray tracing off unless the task asks for another profile. Test the same save, camera view, resolution, and cache state before attributing a change in performance to a patch.
- Report gameplay separately from menu FPS. Record frame time, draw drops, pipeline backlog, memory and transfer costs when available. Compare a stationary interior view and a camera pan when checking lighting, reflections, or streaming.
- Keep the original graphics option available when adding visual corrections. Prefer fixing shader translation, bindings, or resource lifetime when those explain the artifact.
- If Victory still uses the old `data` folder under its install root, tell the user that it needs to migrate. Close Victory normally before copying saves, then run `.\scripts\Migrate-VictoryStorage.ps1 -LegacyRoot '<old install root>'`. The script copies data and cache into `%LOCALAPPDATA%\victory` and preserves existing files. Verify the new location before launching again and keep the old folder as a backup.
- Follow [the asset policy](docs/asset-policy.md). Keep game files, extracted shaders, saves, captures, and raw logs outside the repository. Summary metrics and synthetic tests belong in the repository.
