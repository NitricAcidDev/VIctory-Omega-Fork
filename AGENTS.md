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

## Local and collaboration boundaries

- Keep game files, title-derived shaders and caches, screenshots, captures, disassembly, audio and raw game-bearing logs local and outside Git. Never add them to commits, issues or pull requests. Follow the repository asset policy and use hand-written synthetic fixtures.
- Preserve the offline network policy; do not connect the title to Xbox Live, Rockstar services or other online services. Do not work around encryption or protection.
- The handoff says the user plays with a controller. During live observation, do not inject keyboard or mouse input or change focus; let the user advance the game. Run one game instance at a time. Use local paths and settings appropriate to this machine.
- The handoff says the friend's work is on `main` and this work continues on `secondary-test`. Verify the current branch and working tree before changes, preserve existing work, and fetch current remote changes before updating that branch when network access and authorization allow. Do not overwrite another person's work.
- Live GPU testing can reset the display driver; follow the README's diagnostics and WARP guidance, and state which adapter was used.
