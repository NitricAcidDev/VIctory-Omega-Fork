# AGENT.md: what is wanted, where things are, how to carry on

For the next agent (ChatGPT/Codex, Claude or a person). Written 25 November
2026 by Claude Code. Read this, then [the handoff](HANDOFF-secondary-test.md)
(what was found and changed, with evidence), then
[the merge notes](secondary-test-merge-notes.md). `AGENTS.md` in the root
holds the repository's own rules and wins over this file.

## 1. What the owner of the request wants

In their words, sorted. Work through it without asking questions; they have
said "just do it, you have permission".

1. The picture right in the Xbox Series title that VIctory (Rust, Windows,
   Direct3D 12) runs: no black shadows ("the stupid shaders/black shadow
   bug"), no squares in cutscenes, nothing glossy that should not be, no
   missing or late textures and models, no flickering meshes, shadows right,
   credits and prologue overlays shown right. Faults vary from launch to
   launch.
2. Faster: loading of assets and shaders, frames per second.
3. Take anything better from other repositories and their issues and pull
   requests (list in section 7), compare it with what is here feature by
   feature, keep the better, say why.
4. Ray tracing: they want to try it. A build and a `.bat` for it.
5. A test `.bat` that loads everything without being limited by the card's
   memory (section 5, item 1).
6. Work goes on **`primary`** now, not on `secondary-test`. Push there when
   tested. (`main` only when asked.)
7. After each round: full tests, rebuild the diagnostics exe, update the
   handoff and merge notes, push, give them a launcher to test and say in
   plain words what changed, what to look for, and what was not tested.

## 2. Rules that must hold

- Never push by force. Fetch first; fast-forward or merge. Do not overwrite
  another person's work (NitricAcidDev also works on `primary`).
- Nothing of the game goes into Git, issues or pull requests: no game files,
  shaders, caches, screenshots, captures, disassembly, or logs that hold them.
- The title stays offline: no Xbox Live, Rockstar or other services. Nothing
  that works around encryption or protection.
- Do not launch the game, send input or take the focus; the owner runs it and
  plays with a controller. One game at a time (`tasklist | findstr victory`).
- While `victory.exe` runs: no GPU tests, builds at low priority, and do not
  edit `Launch VIctory.bat` (cmd reads a running batch file by offset).
- Do not hide failures with skip lists; a black frame is not a success.
- Say what is AI-assisted, what was tested, and "Not run with the title" when
  it was not.
- `cargo run -p xtask -- policy` must pass (files of at most 512 KiB, no game
  data). Tests of `emit/lower.rs` live in `emit/lower_tests.rs` for that.

## 3. Where things are

| What | Where |
|------|-------|
| Repository | `https://github.com/NitricAcidDev/VIctory` (renamed VIctory-Omega-Fork; pushes work) |
| Checkout | `runtime\build\victory-render-test`, branch `primary` |
| Build environment | `runtime\build\victory-env.sh` (Git Bash: `. victory-env.sh`, gives `$CARGO`) |
| The exe the launchers start | `target-v2\diag\victory.exe`; the one before: `victory-25004ee.exe` |
| Launchers | `runtime\build\Launch VIctory*.bat`, outside Git |
| Logs | `%LOCALAPPDATA%\victory-nitric\logs\launch-*.log` (strip the colour codes) |
| Settings and caches | `%LOCALAPPDATA%\victory-nitric` (`light.txt` tunes the stand-in light live) |
| Ray tracing merge, half done | `runtime\build\victory-rt-test` (a git worktree, section 6) |

Machine: RTX 5080 16 GB (32-lane waves), 24 logical processors, 64 GB,
Windows 11.

## 4. Build and test (Git Bash, in the checkout)

```bash
. "../victory-env.sh"
F="--locked --offline -j 14"
# GPU tests: one at a time (in parallel they crash now and then inside WARP)
"$CARGO" test --profile diag -p victory-gpu --features diagnostics,blake3/pure $F --lib -- --test-threads=1
"$CARGO" test --profile diag -p victory-shader $F
"$CARGO" test --profile diag --workspace --exclude victory-gpu --exclude victory-shader --exclude victory $F
"$CARGO" clippy --profile diag --workspace --all-targets --features diagnostics,blake3/pure $F
"$CARGO" run --profile diag -p xtask --locked --offline -- policy
# the exe (the game must be closed)
"$CARGO" build --profile diag -p victory --features diagnostics,blake3/pure $F
```

Last full run: GPU 255, shader 256, others 194 passed, policy 204 files ok
(`primary` after `4237574`). A change in `victory-shader` (not in `*_tests.rs`) makes
the first start translate every shader again.

After any merge, look at what it **removes**, not only at conflicts: the last
one dropped a line silently.

```bash
git diff -U0 OLD NEW -- crates | grep '^-[^-]'
```

## 5. Do next, in this order

1. **The launchers that load everything** are made; what they show is to be
   read off the owner's run. `Launch VIctory (load everything).bat` sets
   `VICTORY_TEXTURE_SPILL=1` (textures go to the PC's memory, which the card
   reads over the bus, once the card is full), `(GPU shared memory, no
   VRAM).bat` sets `all` (always there). `VICTORY_TEXTURE_SPILL_PERCENT` (60)
   is how much of the shared memory they may take. Code: `resource.rs`
   (`spill`, `shared_room`, `spills_now`, `card_room`), `placed.rs`
   (`shared_heap`). On the RTX 5080: texels right, nothing of the card's
   memory taken, 46.9 GB/s read against 218.8. Why: the player's log has the
   card at 13.0 of 13.8 GB with 15,000 textures waiting. Log lines: `textures
   are made in the PC's memory`, `textures made in the PC's memory since`. If
   the picture is whole and the frame rate holds with `1`, make it the
   launcher's default.
2. **Read the next log** of the owner's run. Lines: `ambient light`,
   `ambient light follows the sky`, `reflections held` (the stand-in runs);
   `level by level`, `in use`, `waiting` (card memory); `heap buffers` in the
   frame line (was 27 to 117 ms); `host pipeline refused`; `fps`. Compare with
   the numbers in the handoff.
3. **Ray tracing build** (section 6).
4. **Frame rate.** A 220 ms frame is about 70 ms preparing, 120 ms on the
   card, 37 ms copying back. Leads: `Launch VIctory (fast, experimental).bat`
   (fxceless' run-253 switches, 97 to 104 ms elsewhere); upstream `cd94d35` +
   `940d32f` (copy-backs compare only what shaders marked); fxceless #827
   (descriptor cache), #1130 (detile only changed pages), #1126, #1128, #1129.
   `VICTORY_WINDOW_UPLOAD=1` and `VICTORY_CARD_WINDOWS_MB` (already in) with
   resizable BAR.
5. **Missing programs.** Device slots `0x1c8` and `0x218` (pipeline creation
   from a deserialized stream, 16,000 calls a start) are not written; #601
   says they are `Deserialize{Graphics,Compute}PipelineStateX`. Seven compute
   programs are refused on 32-lane cards (hashes in the handoff); #854's
   paired form, #1124 and #799 make them run.
6. **#1095** (bounds on bindless heap reads, null views of the right kind):
   a correctness fix that may explain wrong textures.
7. Race the three forms of the card's hash (`CARD_COMPARE`,
   `CARD_COMPARE_WIDE`, primary's 64-thread tree fold from `ccf29aa`
   `host.rs`) with timestamps, keep the fastest.

## 6. The ray tracing merge

Goal: an exe of its own in which the title's structure builder runs, with a
launcher that sets `VICTORY_BVH_BUILDER=1` (the stand-in light turns itself
off then). fxceless #994 (`fx/pr-994`, stacked on #854 and #993) is the one:
it ran 4 to 11 minutes on an RTX 4070, 5060 Ti, 5070 Ti and 5090. Cost: from
88 ms a frame (5090) to several times the frame.

`victory-rt-test` is a worktree at `04cf7b6` with `git merge fx/pr-994` begun.
Resolved and not to be done again: `README.md` (ours), `descriptor/host.rs`,
`draw.rs`, `host.rs`, `list.rs`, `pipeline.rs`, `pipeline/host.rs`,
`shader_cache.rs`, `bitcode/opt.rs`, `emit/host.rs`, `emit/mod.rs`,
`emit/shader.rs`, `ir.rs`, `wave64_tests.rs`, and `indirect.rs` (ours, whole:
our split dispatch stays, theirs with `SPLIT_DISPATCH` and `split_signature`
is out).

Left: `crates/victory-shader/src/emit/lower.rs`, six conflicts:

1. `group_id`: one function, adding `USER_GROUP_BASE`, left out when
   `c != 0 || stage != Cs || self.user_first() != 0 || named > 40`.
2. `descriptor_data`: theirs moved it to `emit/descriptor_data.rs`. Take their
   file and put our five changes into it: `fixed_prefix`; buffers always raw
   (`(11 | u, 0)`); scalar loads of SRV **or UAV**; `view_bias` taken off the
   record count; `slot_stride` dividing it.
3. `scalar_dynamic(value)` (ours) with their `helper_own` for helper lanes.
4. Exports: both our `nan_guard` line and their `sig` line.
5. `edge(b, taken, false)` / `edge(b, not_taken, taken == not_taken)` (ours:
   an edge has a third part) with their loop notes; pass it in `note_exit`.
6. Tests: keep `#[path = "lower_tests.rs"] mod tests;`, put their three new
   tests there (`lower_cs`, the group base test, the waterfall test).

Codes: `USER_GROUP_BASE` 0x105, `USER_STRUCTURES` 0x106, `USER_THREAD_LIMIT`
0x107. Then `git add -A`, compile, all tests, commit, merge `primary` in. Check
that a constant buffer view is cut to 64 KiB (#911): without it the device goes
35 s after the builder starts. First test run with `VICTORY_BVH_QUARANTINE=1`.

## 7. Where the good fixes come from

`fxceless/victory-gta6` (upstream of `primary`; pull requests as
`refs/pull/N/head`), `Marvisak/VIctory` (the parent), `Belobug/VIctory-Belobug-AMD`
(the stand-in light, at `4f62c2a`; it also holds clean-up passes over
translated shaders), `DexNom/VIctory`. All GPL-3.0. Read a change before taking
it, name its author in the commit, run its tests here.

## 8. What the owner should see if it works

Parts of rooms the sun does not reach are dim, not black; no blocks of colours;
wood and counters without a white glaze; faces and clothes with their textures;
the second start in a few seconds. Too bright or too dark: two numbers in
`light.txt` (gain, least; `0.36 0.05` is the default, `0.5 0.1` brighter).
Anything worse than before: `Launch VIctory (safe, to compare).bat`.
