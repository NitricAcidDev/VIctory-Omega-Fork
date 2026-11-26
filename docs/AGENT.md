# AGENT.md: what is wanted, where things are, how to carry on

For the next agent (ChatGPT/Codex, Claude or a person). Written 25 November
2026 by Claude Code. Read this, then [the handoff](HANDOFF-secondary-test.md)
(what was found and changed, with evidence), then
[the merge notes](secondary-test-merge-notes.md). `AGENTS.md` in the root
holds the repository's own rules and wins over this file.

## Latest Codex verification, 25 November

`primary` integrates fxceless `9c184b3` while preserving the fork's buffer
offsets, UAV reads, mapping leases, readback layouts and streaming changes.
Its diagnostics exe is rebuilt. The full run passed 257 GPU, 257 shader and
195 other tests. Wide-load offset/UAV tests, Clippy, policy, and the GPU
crate without diagnostics also passed (Clippy has warnings).

The timed warm-up now stops scanning additional files after its deadline.
Its zero-second CLI check returned in 0.07 s. A completed warm-up found
3,184 matching shaders from the earlier normal run, all cached, with zero
translation failures. This is coverage of recorded requests, not the full
retail shader corpus or proof of a complete picture. Matching driver
pipelines can reuse the retained pipeline library.

The separate `ray-tracing-test` branch has the same upstream update and
an RT fallback handoff fix: builder startup alone no longer ends the light
while ray passes or world pipelines are unready. Its full run passed 262
GPU, 310 shader and 195 other tests. The owner's preceding RT run was black
around 1 FPS while compilation and texture allocations were pending. That
failure is not declared fixed by synthetic tests. Updated executables have
not been run with the title. Use the existing extra-memory launchers for
the next controlled comparison, with the owner using the controller.

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
4. Ray tracing: they want to try it. A build and a `.bat` for it (made,
   section 6).
5. A test `.bat` that loads everything without being limited by the card's
   memory (section 5, item 1).
6. Work goes on **`primary`** now, not on `secondary-test`. Push there when
   tested. (`main` only when asked.)
7. After each round: full tests, rebuild the diagnostics exe, update the
   handoff and merge notes, push, give them a launcher to test and say in
   plain words what changed, what to look for, and what was not tested.
8. Latest (26 November): base everything fully on **cool_fox** (also called
   CGBJ, `codec-d/cool_fox`, taken down that day; section 5, "cool_fox")
   and make only a few small changes where something is found wrong.

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
| Launchers | `runtime\build`, outside Git: `Launch VIctory.bat`, `(ray tracing test)`, `(safe, to compare)`, `(7900 XTX fork)`, `(cool_fox)`. Every other one is in `runtime\build\Other launchers`, with a `README.txt` |
| Logs | `%LOCALAPPDATA%\victory-nitric\logs\launch-*.log` (strip the colour codes) |
| Settings and caches | `%LOCALAPPDATA%\victory-nitric` (`light.txt` tunes the stand-in light live) |
| Ray tracing build | `runtime\build\victory-rt-test` (a git worktree, section 6) |
| The RX 7900 XTX fork as it is | `runtime\build\victory-xtx`, branch `xtx-base` (local) at `avatarmanz/VIctory-7900XTX` `39031cb`; exe `target-xtx\diag\victory.exe`; `Launch VIctory (7900 XTX fork).bat` starts it with its `play.ps1` settings and both renderer patches, cache `cache-7900xtx`, saves copied to `data-7900xtx`, logs `xtx-*.log`. Its 386 GPU tests pass on the RTX 5080. Replaced as the base by cool_fox (next row) |
| cool_fox (CGBJ), the base now | `runtime\build\victory-coolfox`, branch `coolfox-speed` (local; `coolfox-base` is `f4b48ad` plus three small changes) with the 7900 XTX fork's later speed work merged in and three switches (section 5, "cool_fox"); exe `target-cf\release\victory.exe` (`victory-unchanged.exe` beside it: `f4b48ad` alone); `Launch VIctory (cool_fox).bat` starts it with the settings of its `Play-VIctory.cmd` (Series S profile), `VICTORY_NAN_SCRUB=1`, `VICTORY_FPS_LIMIT=0`, `VICTORY_AMBIENT_FLOOR=0.05`, `VICTORY_TEXTURE_DROP_MIPS=1`, `VICTORY_LANE_CORE=1`; cache `cache-coolfox`, saves copied to `data-coolfox`, logs `coolfox-*.log`. Remote `coolfox` = `medicthewanted/cool_fox` (a mirror) |

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

Last full run of `primary`, after merging fxceless `9c184b3`: GPU 257,
shader 257, others 195 passed, policy ok; Clippy completed with warnings.
The diagnostics exe was rebuilt. Optional wide-load scalar, UAV, and
unaligned-view tests passed too. Not run with the title. Details are in
the merge notes. A change in `victory-shader` (not in `*_tests.rs`) makes
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
3. **Ray tracing build** (section 6): made, waits for the owner's run.
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

### cool_fox (CGBJ): the base now

`codec-d/cool_fox` was taken down on 26 November. Its last commit,
`f4b48ad` (codec_d, 26 Nov 06:33 UTC, "Publish the pareto-residency
Windows build and portable launcher"), is the head of `main` in at least
eight copies found by searching GitHub (`DoubleGhosty`, `medicthewanted`,
`carlm125`, `kitotake`, `AP1028`, `cputroll/cgbj-reupload`,
`VoidOwlSoul/cgbj-backup`, `brenofrancoa/fork-cgbj`); no copy has a later
commit by its author. It is the 7900 XTX fork at `0a43566` plus six commits
of codec_d for NVIDIA RTX cards (an RTX 5090): the Series S two-plane
compose, fallback heaps, a residency guard, 256 MiB DirectStorage, fence
wakes by `WaitOnAddress`, a 32-lane form for wave64 compute programs that
can take it, bindless refreshes scoped to their heap, and backing off when
the card or the PC is short of memory. It lacks the 7900 XTX fork's later
renderer patches (`VICTORY_PORT_*`). No `Cargo.toml`, `Cargo.lock` or
`build.rs` differs from that fork. It builds offline with 1.98.1 in 39 s
(release).

Changes on `coolfox-base`, on top of `f4b48ad`:

| Commit | What | From |
|--------|------|------|
| `c3a7d88` | Decode operandless VOP1 (`v_nop`, `v_pipeflush`, `v_clrexcp`) without a phantom literal, and VOP3 `v_cmpx` writing EXEC | Yoshi-233, `AP1028/cool_fox` branch `fix/gfx10-operand-decode` (after Belobug `4d291d4`) |
| `7af23c9` | `VICTORY_NAN_SCRUB=1`: a NaN or infinity a pixel shader writes as a colour, or an f32 image store, becomes 0, so it cannot spread through bloom, blur and the upscaler into black blocks (seen on NVIDIA on faces and in the air). Part of the shader cache key. The launcher turns it on | `miaohaixiang1-netizen/cool-fox-12gb` `db556df` |
| `e48274c` | Two GPU tests of the indirect clamp still expected `VICTORY_INDIRECT_REST` on; cool_fox turned it off by default (`f4b48ad`) and they failed. They now test both forms (the batch) or the run's form (the host list) | here |

Tests on the RTX 5080 (`"$CARGO" test --release --locked --offline -p
victory-shader -p victory-gpu -p victory-formats`, from the worktree with
`CARGO_TARGET_DIR` set to its `target-cf` after sourcing `victory-env.sh`,
which changes directory): GPU 335, shader 283, formats 41 pass. The shader
cache in `cache-coolfox` was filled ahead with `victory shaders <game
folder>` and the launcher's settings, so the first start translates
little (25,756 shaders; the same 26 compute programs refused on the card).

What the offline translation says (`victory translate <game> --list` with
`VICTORY_HOST_WAVE=32 VICTORY_HOST_MAX_WAVE=32 VICTORY_NAN_SCRUB=1`, 1 min
42 s): everything translates except 195 geometry shaders (unused), 5 ray
traced shadow programs (`a descriptor read only as data`; ray tracing is
off) and **26 wave64 compute programs** with `no wave32 lowering`: seven
`GpuParticleEmit`, nine `vfx_proj_decal_trail`, three `vfx_triangleClip`
(`6b82bc7d22016cb5`, `02384a60fda6f2d0`), five `water_shading`
(`5dc3b25986622949`, `6f225b641c5b0708`, `926c46bab3b86bd6`) and two
`shadowmask:CS_GenerateShadowMasks*_SampleHairScreenSpace`. The reflection
tiling (`2cc4c197ee29e291`, primary's squares fix), particle visibility
(`369cbe906ea72764`) and alpha resolve (`2da4beb386509b77`), all refused on
primary, translate here. Next small change if the owner's log shows these
dropped: primary's per-hash wave-wide list (`shader_cache.rs`
`translate_listed`, `emit::with_wave_wide`) ported for those 26; cool_fox's
`VICTORY_WAVE_WIDE=1` turns it on for every program, which lost the card
before.

**Run 1 with the title (26 Nov, by Claude at the owner's request, keyboard
stand-in pad, prologue cell to common room):** 23-24 fps median in the game
(was 5-6 on `primary`), 30 on the start screen (the title asks for 33.3 ms
frames). Face skin missing, far heads missing, rainbow tattoo texture,
corridor walls near-black. Frame: 32-42 ms our preparing (copies ~100 MB,
copying back 40-65 MB), ~0 ms waiting for the card; 16,000-17,000 textures
waiting with the 16 GB card full.

Branch `coolfox-speed` then adds (local, not pushed):

| Commit | What |
|--------|------|
| `971cfb6` | Merge of `xtx/main` `39031cb`: the fork's ~45 later commits (renderer patches 1 and 2 off by default, `UNDER_FUSED`, `SYNC_IN_PLACE`, `SPANS_QUICK`, `SEND_OWN`, `SHOW_NEW_LATER`, view addresses, shader fixes). 4 conflicts resolved keeping both sides (see its message); no added line of either side lost |
| `88316ee` | `VICTORY_FPS_LIMIT` (`frame.rs`: 0 = unlimited, n = cap; replaces the title's interval) and `VICTORY_AMBIENT_FLOOR` (`ambient.rs`: least light of the stand-in's estimate) |
| `ee8a237` | `VICTORY_TEXTURE_DROP_MIPS` (miaohaixiang1-netizen `fcb4f71`, cool-fox-12gb): host copies leave out the N largest levels of big 2D textures |

Tests on the RTX 5080: GPU 354, shader 291, HLE 151, loader 42, formats 41,
core 13 pass. Shader cache refilled (25,764 translate; 26 wave64 compute
programs still refused).

**Run 2 (same route):** 25 fps median (26 in the corridor and common room,
up to 30; the start screen 530-725 fps uncapped). **Faces and far heads
drawn, the corridor lit, hair whole**; 6,700-8,500 textures waiting. Frame:
30 ms preparing, copies 13 ms, copying back 12 ms. Dropped draws unchanged
(~340 a frame), so they were not the faces: the textures were.

A/B on NVIDIA, both renderer patches are **off** in the launcher:
- Patch 1 (`VICTORY_PORT_CONSTANTS`): 8-9 fps, the card 94% busy and the
  submitting thread waiting 85 ms a frame: the card reads the title's 185 MB
  of constants from the PC's memory over the bus.
- Patch 2 alone (`VICTORY_PORT_DESCRIPTORS`): pictures posterized, view
  records "found by the fallback" late, ~2,500 draws dropped a frame.

**Round 3 (26 Nov afternoon), on `coolfox-speed`:**

| Commit | What | With the title |
|--------|------|----------------|
| `329834b` | `VICTORY_TEXTURE_SPILL` (1/all): textures in the PC's memory when the card is full | **Off.** With 1 the game ended about a minute in, three times out of three, "a host command list did not close (parameter incorrect)". The debug layer is not installed here (Graphics Tools), so the command is not known |
| `3c83330`, `5b73321` | A per-program wave-wide list (`emit::with_wave_wide`, `shader_cache.rs` `WAVE_WIDE_DEFAULT`, `VICTORY_WAVE_WIDE_ONLY`): 7 GPU particle emitters, 6 projected decal trails | On; walls smooth; refused on the card 26 -> 13. The two hair shadow masks (`WAVE_WIDE_SHADOWS`) took the blocks around hair away but banded every wall, two runs of two: off. Water's light off (fxceless #799) |
| `3615e6f` | Refreshes of host copies cut in pieces for the threads (`VICTORY_REFRESH_PIECE_KB`), `VICTORY_WIDE_THREADS` | **Off / default.** Cutting: the title wrote at 0 on the loading screen, twice. 8 wide threads: banded walls |

Thread settings in the launcher: `VICTORY_LAND_THREADS=6`,
`VICTORY_THIN_THREADS=8`, wide threads at their default 6. More threads did
not raise fps (25-26 in the cell and corridor either way).

The title's unimplemented `Deserialize{Compute,Graphics}PipelineStateX`
calls are its FSR 4 upscaler passes ("title said: FFX FSR4 ACO: ...
falling back to NewBE"): it falls back by itself; not the faces.

Pause menu (Esc) works; the only save is "(Autosave) Prologue (1%)", so no
later part of the map was reachable. An AVG AntiTrack pop-up can take the
foreground mid-run; the key script then stops (keys go only to the game's
window); only its window was closed (WM_CLOSE), at the owner's word.

Why not 60 fps (told to the owner): the preparing thread is ~30 ms of a
~38 ms frame (noticing and moving ~100 MB a frame up and ~40 MB back,
translating list calls); the card is about half busy. "Mostly in RAM" makes
it slower (renderer patch 1: 8-9 fps). The way up is the fork's own next
step: split what one submission does over two threads in a row, ask for the
next submission while this one is sent (`docs/findings.md` of the fork,
"What is next"), and copy only what really changed.

**Round 4 (26 Nov, 13:40-14:10):**

- Dropped draws, from a diagnostics run (`target-cfdiag`): of ~340 a frame,
  250-296 are the ray-tracing structure builder (off on purpose) and 1 the
  refused `02384a60fda6f2d0`; a constant "592/frame being made" is a stale
  count from loading. Not a visual fault.
- The preparing thread, diagnostics split (ms a frame): 14.2 refreshes in 40
  flushes of ~10,300 spans (~9,300 copy nothing), 7.8 copies in the card's
  memory, 4.2 copies and fills, 3.5 the write watch, 3.7 planning card steps,
  2.7 bytes under flights; threads together 16.5 ms waiting for the guards.
- **Banding of the whole picture** (contours on walls and ceilings): in 5 of
  8 runs after 13:00, none of 3 before, with the same build and settings
  between a smooth and a banded run. Ruled out one by one: the hair shadow
  masks, the stand-in light (`VICTORY_AMBIENT=0`), 8 wide threads, the
  launcher's landing/list threads, `VICTORY_TEXTURE_NO_WAIT=0`. Log message
  kinds do not differ between smooth and banded runs. Not found; the D3D12
  debug layer (Windows Graphics Tools, not installed) or a frame capture is
  the next tool.
- The hair shadow masks are back in the wave-wide list (`b128682`): the
  blocks around hair are gone.

Next: that split; free roam (needs a later save or playing the prologue
through); the 13 refused programs; the glossy, wet-looking clothes seen in
the common room (not the decal trails: seen without them too).

**Round 5 (26 Nov, 15:30-17:40), `coolfox-speed` `0ae378e`..`34c53f3`:**

| Commit | What | With the title |
|--------|------|----------------|
| `0ae378e` | `VICTORY_NAN_SCRUB`: an infinity becomes 64,000 of its sign, not 0; NaN still 0. `TRANSLATOR_VERSION` 1 -> 2 (the cache key does not follow the translator's code: old translations would have been served) | **The banding is gone**: the "banding" was the whole picture posterized (faces, walls, the yard, from the opening cutscene on), and the owner saw the sun's reflection black. One run (diagnostics build): smooth where the five runs before were all posterized |
| `601194a` | `VICTORY_TITLE_AFFINITY` (`all`, `spread`; unset: the title's own masks). The title pins its threads as on the console's seven cores, processors 0-13: eight workers on four cores of the 9900X, 14-23 nothing of the title's | Off: `all` measured median 25 fps, as without |
| `34c53f3` | `VICTORY_AB=<s>,VICTORY_WIDE_SPIN` measures the refresh threads' spin in one run | See below |

- Ruled out for the banding before the scrub: the 13:01 wave-wide list
  (`VICTORY_WAVE_WIDE_ONLY=none`, own cache: still posterized), drop-mips
  (never a one-level table), the refresh cutting (does nothing at 0), the
  cache (two fills compared file by file: all 25,779 identical, translation
  is deterministic). Why only some runs: an infinity had to come first.
- In-run A/B (`VICTORY_AB=8,VICTORY_LANE_CORE,VICTORY_WIDE_SPIN`, the
  owner playing, diagnostics build), median frame ms of five stretches:
  neither 58.8/55.8/44.5/47.3/44.2, own core 52.2/48.1/39.6/43.8/39.9, spin
  59.6/46.4/40.4/40.9/39.4, both 48.9/49.4/49.8/41.5/50.4. **The launcher
  sets `VICTORY_LANE_CORE=1`** (about 10% shorter frames); spin stays off.
- Timeline (`VICTORY_TIMELINE`, three recordings of 3 frames): the chain a
  present comes out of is mostly the title's own busiest thread (8-9 ms a
  frame), threads woken and not yet running (1.4-4.6 ms: ~13 cores' worth
  of work on 12), and our faults: title threads spent 23-39 ms a frame
  together in the guard fault handler, one fault 16.9 ms. The card ran 12-14
  ms a frame and stood idle 13-15.
- The frame replay (`victory replay`) of a 6.4 GB capture of the cell lost
  the device (hung) while loading, at the first barriers, before any of the
  title's work: not usable here yet.
- The title can be driven without focus: `VICTORY_KEYS=<s>:Enter,...`
  (scripted pad), and the diagnostics build saves presented frames itself
  (`VICTORY_CAPTURE`): no keys or focus taken while the owner uses the PC.

**Round 6 (26 Nov, 17:40-18:20), `coolfox-speed` `c9c31c7`, `824b114`:**

- **The first crash outdoors** (owner's session, 17:39, ~10 min in, first
  time anyone got outside): the title read `[rooms + 80 * index + 0x2c]`
  with the "current room" global at -1 (image `+0x9326b4`; the global at
  `0x14aab8594`, set with the current interior pointer `0x14aab84f8` by the
  function around `+0x7c0a2a`). The reader takes the interior, calls
  something, then reads the room: a race in the title's own code with the
  thread that left the interior. Not seen in any other log.
- Why such races show here: our guard faults pause the title's threads. In
  diagnostics builds every place that takes the guards' lock is now timed
  (`watch.rs` `held_line`, once a second): held about 6 ms of a 42 ms frame
  in all, but the title's write faults (~420 a frame) waited 30-33 ms a
  frame for it together, up to 7-13 ms once: sleep and wake-up on a busy
  processor, not long holds.
- `c9c31c7`: **`VICTORY_GUARD_SPIN` (200 us by default)**: a waiter asks for
  the lock awake before sleeping. Turn by turn (156 s each way): faults'
  wait 30.3 -> 10.6 ms a frame, frames ~1 ms shorter in 11 of 12 pairs.
- `824b114`: `VICTORY_GUARD_BOOST=1` (off): our holders time-critical while
  they hold it. Faults' wait 8.4 -> 7.2 ms, stalls over 2 ms in 115 seconds
  of 157 instead of 130; frames shorter in only 8 of 14 pairs.
- "Never over 30 fps" is the cost, not a cap: the start screen runs at 550+
  fps; in game the last session ranged 2-31 (mostly 10-28 outdoors).
- The owner asked for the game muted while tests run: the scratchpad's
  `mute.ps1` mutes victory.exe's audio session (the mixer's per-app mute);
  `-Unmute` gives it back. Their own sessions are not muted.

### From the RX 7900 XTX fork

`avatarmanz/VIctory-7900XTX` (remote `xtx`; the owner's copy is
`fornorharry/VIctory-7900XTX`, a shallow checkout in `Leonedia\work`) runs a
city street at about 53 ms a frame on an RX 7900 XTX, from 57-75 ms before its
two renderer patches and 280 ms three days earlier. Its base is fxceless
`a26859f` with 328 commits of its own, so it is taken commit by commit. Its
`docs/findings.md` says where a frame goes there.

Taken: `5f86fa3` as `VICTORY_CARD_AWAKE=1` (the card's clocks kept up between
submissions; 37.8 -> 28.6 ms for a captured frame there; not measured here).
`Launch VIctory (7900 XTX settings).bat` sets it and the fork's other switches
that exist here.

Next, by the size of what they measured, each behind its switch, off:

| Commit | Switch | What | Needs |
|--------|--------|------|-------|
| `86dc109`, `4b82f6c`, `9c23e8d` | `VICTORY_PORT_CONSTANTS` | The title's 174 MB constants ring used in place as a host heap: no uploads, compares or ~6,000 fills a frame | `port.rs` (new), `inplace.rs` `stretch`, `xmem.rs` gives a 32-bit graphics reservation's pages its section's slices in order. `inplace.rs` here has `cover_in`, `Cover::Run`, `views()` already |
| `9997d25`, `4cb90be` | `VICTORY_PORT_DESCRIPTORS` | Six 32-byte stores in the title's code (byte-checked) mark the descriptor slots they write; `sync_heaps` takes those instead of scanning ~600,000 slots a frame | `victory-loader/src/descriptor_sites.rs`, a hook in `run.rs`, `descriptor.rs` |
| `3a99ee6`, `e000e02` | `VICTORY_THIN` | List calls written down on the title's threads, translated by eight of ours (75 -> 68 ms there) | `list/thin.rs` |
| `b620284`, `6f3ede5` | `VICTORY_UNDER_FUSED` | Title bytes under shader writes compared with AVX2 on threads (14 -> 4-7 ms) | |
| `3bab23c` | `VICTORY_SHOW_AFTER` | A slot shows a new texture only after its upload ran (ended a device loss every ~16 min there) | |
| `350c686` | `VICTORY_UPLOAD_AHEAD` | First uploads on a copy queue of their own | |
| `cc03ecb` | `VICTORY_CARD_STEPS` | Reads of title memory under flights done on the card | |

Before the first one: a log of a run with `(7900 XTX settings)` against
`Launch VIctory.bat` in the same scene, to see what the card and the
preparing thread take here.


## 6. The ray tracing build

Done as a build of its own, not on `primary`: branch `ray-tracing-test`
(on GitHub; the merge is `ff82795`, on top of `primary` `4d52524`), worktree
`runtime\build\victory-rt-test`, exe `target-rt\diag\victory.exe`. It is
fxceless #994 (`fx/pr-994`, stacked on #854 and #993) merged into this tree;
what was kept of each side is in the commit's message. Tests on the RTX 5080:
translator 309, host 257, others pass. **Not run with the title yet.**

Launchers: `(ray tracing test)` sets `VICTORY_BVH_BUILDER=1`;
`(ray tracing build, rays off)` is the same exe without it, in which the seven
compute programs the card's driver refuses should run in the paired form (as
#994 reports; not seen here). Both use
their own cache (`cache-ray-tracing`) and a copy of the saves
(`data-ray-tracing`), made once.

To build it: `. victory-env.sh` **changes into the main checkout**, so go back
and set the target folder after it, or the wrong tree is compiled and tested:

```bash
. "../victory-env.sh"
cd "../victory-rt-test"
export CARGO_TARGET_DIR="$(cygpath -m "$PWD")/target-rt"
```

Lines to read in its log: `the ray-tracing structure builder runs: all of its
programs have host pipelines` (or `stays off: not all of ...` with how many
are missing), `ray-tracing structure builder is turned off` and why, `no host
pipeline for dispatches by thread counts`, device lost. Others had it run 4 to
11 minutes on an RTX 4070, 5060 Ti, 5070 Ti and 5090, at 88 ms more a frame
(5090) to several times the frame; it wants much shared memory (a PC with
16 GB ran out). If the title stops: once with `VICTORY_BVH_QUARANTINE=1`
(#1013 makes it hold), then `VICTORY_BVH_TRACE=<file>`.

If the pictures are better than the stand-in light at a frame rate the owner
accepts, or the paired form alone (rays off) is worth its card time: merge the
branch into `primary`.

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
