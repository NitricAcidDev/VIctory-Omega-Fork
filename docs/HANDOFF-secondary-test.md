# Handoff: `secondary-test`

24 November 2026. AI-assisted (Claude Code). For whoever continues here, a
person or an agent. To merge into `primary`, read
[secondary-test-merge-notes.md](secondary-test-merge-notes.md).

## Where it stands

- The title boots, plays its menus and enters the world. The last run whose
  log is available (an earlier commit, `29124aa`) went on for almost 8
  minutes at 3,000 to 4,000 draws a frame with no device loss in its log.
- It ran at 0.4 to 4 frames a second. What limited it is in the table below.
- **Nothing after `29124aa` has been run with the title.** The loading and
  streaming work is proven by unit tests and benchmarks only.
- Whether the 3D picture is right has not been judged from a capture.

Machine: Windows 11 x64, RTX 5080 (driver 617.42), 24 logical processors.

## Commits since `primary` (`4800f0d`)

| Commit | What |
|--------|------|
| `7941c4b` | The low-address guard the fxceless merge dropped (the title asserted at startup here without it) |
| `29124aa` | Buffer views that do not start on 16 bytes; record counts of structured buffers |
| `808b29f` | Range lock on title memory, translations kept across rebuilds, more pipeline workers, streamed writes in place |
| this one | Texture staging buffers are reused; this handoff |

## What the last run said, and what was done

Mean of the last 60 frame lines: 813 ms a frame, of which 378 preparing (264
of it textures, 44 copies), 169 on the GPU and 64 copying back.

| Cost | Measured | Done | Expected, not yet run |
|------|----------|------|-----------------------|
| Textures waiting for title memory | 80 % of all texture time | range lock: 18.5 s to 44 us for 1,000 reads beside remapping | most of it gone |
| The pass over textures picked by index | median 256 ms a frame, its own limit is 250 | staging buffers reused: 341 to 2 us (64 KiB), 674 to 20 (512 KiB), 1,349 to 72 (2 MiB) | more textures a frame, a shorter pass once caught up |
| About 1,450 pipelines queued at start | 4 minutes with 8 workers and a cold cache | 16 workers; translations kept | the second start much shorter |
| Streamed writes | 332 us for 64 KiB, 24 threads | one view per section: 28 us | pages already resident only |
| Creating the host texture | about 300 us each | nothing: "not zeroed" heaps made no difference | place textures in shared heaps |
| GPU time | 139 to 169 ms a frame | nothing | unknown; needs `VICTORY_GPU_TIMING=1` |

## Run it

`Launch VIctory.bat` (in `runtime\build`, outside this repository) starts the
diagnostics build `target-v2\diag\victory.exe`. Logs are `launch-*.log` in
`%LOCALAPPDATA%\victory-nitric\logs`.

`VIctory RTX 5080 test.cmd` beside it starts **another, older build** and
writes `loop1000-*.log`. The device loss of 24 November, 22:21, was that one.

A white picture that slowly fades in is reported by others with this tree as
the title's auto-exposure settling over many frames; at 1 fps that takes
minutes. A device removal is a crash, and says so in the log.

Compare two starts with the same cache. Look for `host pipelines all made`
(times, translations kept and made), `texture refreshes per frame` (the wait
for title memory), `heap buffers ... passes` and the `ms/frame` lines.

| Switch | For |
|--------|-----|
| `VICTORY_PIPELINE_WORKERS=8` | the old number of workers |
| `VICTORY_STAGING_POOL_MB=0` | no reuse of staging buffers |
| `VICTORY_SHADER_LOOP_LIMIT=256` | if the driver resets |
| `VICTORY_RELAY=2` | the fork's own faster mode, off by default; others report less stability with such switches on 8 GB cards |

## If it breaks

| Seen | Look at first |
|------|---------------|
| Access violation reading title memory | `xmem.rs` `cut` (the range a remapping locks) |
| A texture shows another one's picture | `texture.rs` `SPARE`, `no_leftovers`; run with `VICTORY_STAGING_POOL_MB=0` |
| Wrong shaders after a switch | `emit::SWITCHES` must name every variable a translation reads |
| Stops at start, offset `+0x19f4f58` | the low-address guard |
| The whole PC is slow while it loads | `VICTORY_PIPELINE_WORKERS=8` |

## Next

1. One run of this build, then compare with the table above.
2. Device slots `0x1c8` and `0x218`, pipeline creation that is not written:
   over 16,000 calls of the first at start. The title goes without those
   pipelines. Their descriptions' first words are logged; the shader
   pointers in them are a few hundred bytes long, so not whole containers.
3. Host textures placed in shared heaps instead of one allocation each.
4. The shaders that still do not translate (listed in the merge notes).
5. Judge the picture from a capture: `VICTORY_CAPTURE`, with
   `VICTORY_DUMP_TARGETS=1` and `VICTORY_CAPTURE_FRAMES`.

## Checked on this commit

`cargo test --workspace` (116 GPU, 98 HLE and 212 shader tests), clippy with
diagnostics, the asset policy. Built with the pinned toolchain and
`--features blake3/pure` (this PC has no C compiler). GPU unit tests crash
now and then when run in parallel, with and without these changes; none do
with `--test-threads=1`.

## Rules here

Push to `secondary-test` only, never by force. Fetch first: the branch has
been replaced while work was under way. Nothing of the game goes into Git.
The player uses a controller: do not send input or take the focus.
