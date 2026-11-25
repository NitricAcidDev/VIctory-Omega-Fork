# Guide for coding agents

Read [README.md](README.md) and [docs/architecture.md](docs/architecture.md) before changing this repository. Follow the repository's asset policy and any more specific instructions in the affected directory. These instructions apply to all coding agents, including Claude.

## Priority

Focus on producing correct, visible 3D world frames and improving shader translation and compilation. Menus, intro artwork, credits and subtitles have rendered, but the opening 3D scene was still dark in the latest handoff. Successful boot, shader compilation, pipeline creation or passing synthetic tests alone does not establish that the world renders correctly. Preserve working behavior while fixing the underlying causes; do not hide failures with skip lists or report a black frame as a success.

Consult the 24 November 2026 handoff when it is available for previous results, tests, hypotheses and source references. Verify the current hardware, branch, build settings and behavior rather than assuming the handoff still applies. If it is unavailable, work from the repository and current evidence.

## Investigation

1. Obtain a complete world-frame capture, including intermediate render targets and depth where possible. Determine whether geometry reaches the G-buffer and where lighting or compositing loses it. Compare the same scene and record observations; distinguish evidence from hypotheses.
2. Correlate dropped draws and dispatches with shader translation errors and frequency. Prioritize fixes that unblock the most relevant work in that frame. Implement correct translation and runtime behavior, with small synthetic regression tests and validation of generated shaders where available.
3. Inspect actual remaining failures before integrating external changes. The handoff identifies indirect rectangles with vertex inputs and lane-control/derivative handling as possible leads, not proven fixes. Review dependencies, compatibility and licensing before adopting code.
4. Measure performance after pipeline compilation settles, using comparable scenes and settings. Record frame times, CPU/GPU bottlenecks, hardware and driver. Keep diagnostic settings such as shader loop limits explicit and compare against a baseline before drawing correctness conclusions.

Use the pinned Rust toolchain and the repository's documented commands. Run focused tests during development and relevant shader and GPU suites before reporting a change; run broader checks when appropriate. Prefer synthetic inputs and readback tests. Report what was tested, what was observed, and any remaining uncertainty or unavailable hardware. A full retail shader corpus census is useful when available, but do not claim one was performed without evidence.

## Synchronization and parallel work

- At the start of work, inspect the current branch, worktree, remotes and uncommitted changes. Fetch the relevant remote branches when network access is available, then compare their commit ancestry and diffs. A branch name or a reported merge alone does not prove that a change is present.
- Check again before integrating, building a shared release, committing or pushing, and whenever another contributor reports new work. Fetching updates remote references without changing the working tree. Integrate the requested changes deliberately and verify the result; preserve uncommitted work and avoid reset, force-push, or overwriting another contributor's changes.
- People and agents may work in parallel. Check active claims, handoffs, open changes and worktrees before taking overlapping work. Coordinate ownership of shared files and builds, communicate dependencies and findings promptly, and review changes that affect another person's work.
- Use existing suitable worktrees where possible. Keep independent edits isolated when they would collide, and integrate them into the requested branch after review and testing. Preserve the user's requested primary build and confirm its commit and actual launcher executable before reporting it ready.
- If a fetch or coordination source is unavailable, continue useful independent work from the known local state and report what could not be checked. Never assume that unsynced changes are incorporated.

## Local and collaboration boundaries

- Keep game files, title-derived shaders and caches, screenshots, captures, disassembly, audio and raw game-bearing logs local and outside Git. Never add them to commits, issues or pull requests. Follow the repository asset policy and use hand-written synthetic fixtures.
- Preserve the offline network policy; do not connect the title to Xbox Live, Rockstar services or other online services. Do not work around encryption or protection.
- The handoff says the user plays with a controller. During live observation, do not inject keyboard or mouse input or change focus; let the user advance the game. Run one game instance at a time. Use local paths and settings appropriate to this machine.
- Branch ownership can change. Verify the current branches, remote state and handoff before integrating work. Preserve existing changes and do not overwrite another person's work.
- Live GPU testing can reset the display driver; follow the README's diagnostics and WARP guidance, and state which adapter was used.
