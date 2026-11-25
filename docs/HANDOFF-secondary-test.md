# Handoff: `secondary-test`

25 November 2026, evening. AI-assisted (Claude Code). For whoever continues
here, a person or an agent. To merge into `main` or `primary`, read
[secondary-test-merge-notes.md](secondary-test-merge-notes.md).

## Where it stands

- The base is `primary` (`e0be79c`) with the newest fxceless tree (`d510ee4`)
  merged in. On top of it are fixes for what a player reported: surfaces that
  are glossy and should not be, squares (mainly in cutscenes), shadows,
  flickering meshes, missing textures and models that vary with each launch,
  what is laid over the cutscenes, loading, and the frame rate.
- **Nothing since `29124aa` has been run with the title on this machine.**
  What is said below rests on unit tests, GPU tests on an RTX 5080,
  benchmarks, and on what others measured in the title (said where).
- The first start translates every shader again (the translator changed).
  Compare loading times from the second start on.

Machine: Windows 11 x64, RTX 5080 (driver 617.42, 32-lane waves), 24 logical
processors, 64 GB.

## What was reported, and what was done

"Here" in the last column: a test on this RTX 5080. "Title": seen by someone
in the title, not by us.

| Reported | Found | Done | Shown by |
|----------|-------|------|----------|
| Squares | An indirect compute pass over classified tiles asks for 85,000 to 115,200 groups in x; 65,535 ran, the tiles past them kept an earlier shot | The rest runs as a second dispatch | Here: 100,000 of 100,000 groups run. Title: fxceless #834 (the same fix, found apart): blocks in 2 of 4 runs without, 0 of 3 with |
| Squares, along the bottom | The tiling of the screen-space reflections has only a 64-lane form, which the driver refuses | Can be switched on: `Launch VIctory (squares test).bat` (#823; off by default: two passes it feeds darken the room) | Title: #823, 3 of 3 frames; issue #512 for the darkening |
| Squares, glow that stays | One NaN or infinity stays in what the title blends into every frame | Pixel exports are guarded (#837) | Here |
| Glossy | A view that picks linear or sRGB of a texture made as the other read with the texture's curve: 0.5 of a roughness map read 0.21 | Such views keep their own, in seven formats | Here, fails without |
| Glossy, wrong material | Normalized vertex data divided by the host is off: a material index truncates to the one below (16-bit code 257 came back as 256) | The nearest float, worked out in integers (Marvisak/VIctory `da82e44`) | Here, fails without |
| Wrong vertex data | `tbuffer_load_format_d16` gave the first of two halves as 32 bits | Two 16-bit channels a register (`c430b1d`) | Here |
| Wrong pixels along edges | A helper lane in a waterfall loop took another lane's value | Its own (`0601955`) | Here: 9 of 388 pixels wrong without |
| Shadows | The tiled cascade shadow programs ran in an approximate form on 32-lane cards, one not at all (it is in this machine's log) | Exact forms (#1002) | Its tests, here. Title: #1002, an RTX 4080 |
| Shadows | One sampler used with and without a comparison; a comparison with a minimum or maximum became another kind | #756 | Its tests |
| Missing, wrong, other ones in each launch | A texture uploaded before its data came; a look at 48 KiB of it; a list late | Due again when a read ends; every block looked at; #606, #632 | A block that comes last is seen in all 64 places |
| Textures that never come, away from where the player started | Retries newest first: the older ones had no turn. Memory that changed under a texture a table showed lately was left alone for 240 frames. A view or a staging buffer that could not be made counted as done | Turns; 8 frames; tried again (after NitricAcidDev, whose changes are not pushed) | Its tests |
| What is laid over the cutscenes, the intro | A texture that keeps changing was looked at every eighth frame and shown a list late | Looked at by every submission, copied by the list that samples it; `VICTORY_INTRO_UPLOADS` on by default | Title: NitricAcidDev, the intro only |
| Flickering and missing meshes | Records nothing wrote charged a list's budget; LDS barriers; stale arguments; corner numbers with the edge flag in them | Free; #714; #663, #742, #743; `c0228d9` | Their tests |
| Missing models | The host refuses a pipeline that blends an integer target | #501 | Its test; in this machine's log |

Numbers with `#` are pull requests and issues of fxceless/victory-gta6, the
others commits of Marvisak/VIctory (the fork's parent). Each was read before
it was taken; what it changed is said in the commit that took it.

## Frame rate and loading

| What | Before | Now |
|------|--------|-----|
| Submissions by a thread of the layer, fetches by a copy queue | off | on in the launcher. Others: 10 to 21 % less time a frame, the same picture, an RTX 5080 among them (fxceless #873) |
| Compare of copies on the card | a thread a 64 KiB piece | 64 threads (#1014: about 35 to 10 ms a frame on an RTX 3060) |
| 3,000 host textures made | 512 ms, 1,062 MB | 42 ms, 976 MB |
| Telling whether a texture changed | 5.3 GB/s | 33 GB/s, side by side |
| Passes before the title's bake draws | up to 1.5 s in every frame that bakes | 6 s in all, then 20 ms a second |

`Launch VIctory (fast, experimental).bat` adds every other switch that was
measured to pay in the fxceless tree (its README: 120 to 97 ms a frame in the
first rooms). The fork's fps counter is in the corner of the picture
(`VICTORY_FPS_COUNTER=0`: not).

## Run it

`Launch VIctory.bat` (in `runtime\build`, outside this repository) starts the
diagnostics build `target-v2\diag\victory.exe`. Logs are `launch-*.log` in
`%LOCALAPPDATA%\victory-nitric\logs`. Beside it:

- `(safe, to compare)`: the newest changes off, frame rate switches too.
- `(fast, experimental)`: every frame rate switch on.
- `(squares test)`: the reflection tiling runs, two ray passes held back.

`VIctory RTX 5080 test.cmd` starts **another, older build**.

Lines to look for in a log:

| Line | What it says |
|------|--------------|
| `indirect compute dispatches` | the largest group count asked for, and how many records a frame are cut in two. Over 131,070 is still cut |
| `textures in shared heaps` | heaps, textures in them, textures made on their own for the heaps' waste |
| `writes behind the guards` | how many texture slots were uploaded before the bytes came |
| `a view reads as its texture's own format` | views whose format is still not kept (first 32) |
| `host pipeline refused` | pipelines that are missing from the picture. Expected here: `2da4beb386509b77`, `02384a60fda6f2d0`, `6b82bc7d22016cb5`, and `2cc4c197ee29e291` without the squares test |
| `host pipelines all made` | times, translations kept and made |

## If it breaks

| Seen | Try | Look at |
|------|-----|---------|
| Anything new | `(safe, to compare)`, then its lines one by one | |
| The device is lost | `VICTORY_QUEUE2=0`, then `VICTORY_ASYNC_SUBMIT=0` | `queue2.rs`, `queue.rs` |
| A crash or a wrong texture while loading | `VICTORY_PLACED_TEXTURES=0` | `placed.rs` |
| A low frame rate with much uploaded a frame | `VICTORY_PAGE_WRITES=guards`, `VICTORY_LIVELY_TEXTURES=0` | `descriptor/host.rs`, `texture.rs` |
| Too dark or too bright | `VICTORY_NAN_GUARD=0` | `lower.rs` `nan_guard` |
| Colours off | | `descriptor/host.rs` `keeps_gamma_view` |
| Shadows worse than before | | `ir.rs` (#1002) |
| A compute pass wrong on one side of the screen | | `lower.rs` `group_id`, `draw.rs` `group_base` |
| Stops at start, offset `+0x19f4f58` | | the low-address guard |
| The whole PC is slow while it loads | `VICTORY_PIPELINE_WORKERS=8` | |

## Known, and not ours to fix quickly

Places the sun and lamps do not reach directly are dark: the title's
ray-traced light needs structures whose builder does not run on 32-lane
cards yet (fxceless #16, #512). What looks like black or missing textures in
the prison is mostly that.

## Next

1. One run, then the lines above. Confirm or drop each cause on a picture.
2. fxceless #854 (paired form of wave64 programs, mesh programs over 32 KiB)
   and #799's approximate form would let the refused programs run. Large, or
   approximate: not taken.
3. Upstream `cd94d35` + `940d32f`: copy-backs compare only what shaders
   marked (365 MB compared a frame for 2 MB, fxceless #650). Needs
   `27c3dc9` and `c75fdba`: a merge of its own.
4. Device slots `0x1c8` and `0x218`, pipeline creation that is not written.
5. fxceless #827's descriptor cache (40,000 to 113,000 views made a frame).
6. `codex/shader-coverage` (`a71d96a`): see the merge notes.

## Rules here

Push to `secondary-test`, never by force; to `main` and `primary` only when
the owner of the request says so. Fetch first: the branch has been replaced
while work was under way. Nothing of the game goes into Git, nor logs that
hold it. The player uses a controller: do not send input or take the focus.
