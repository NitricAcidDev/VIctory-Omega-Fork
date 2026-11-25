# Handoff

25 November 2026, late evening. AI-assisted (Claude Code). For whoever
continues here, a person or an agent. `primary` and `secondary-test` are the
same commit; **work goes on `primary` from here**. How the two were compared
is in [secondary-test-merge-notes.md](secondary-test-merge-notes.md).

## Where it stands

- The base is `primary` (`ccf29aa`) and the newest fxceless tree (`085ee7c`).
  On top of them are fixes for what a player reported, each in the table
  below.
- A player ran the build of `25004ee` twice on the machine below: 9 and 12
  minutes, the opening cutscene and gameplay, no crash. Models: good. What was
  left: "shaders/black shadow bug". The answer to that (rows marked **new**)
  has not been run with the title yet.
- The first start after this update translates every shader again (the
  translator changed). Compare loading times from the second start on.

Machine: Windows 11 x64, RTX 5080 (driver 617.42, 32-lane waves), 24 logical
processors, 64 GB.

## What was reported, and what was done

"Here" in the last column: a test on this RTX 5080. "Title": seen by someone
in the title, not by us, unless it says "player".

| Reported | Found | Done | Shown by |
|----------|-------|------|----------|
| **new** Black where the sun and lamps do not reach, blocks of other colours | The title's indirect light is traced, and nothing builds the structures its rays walk | A stand-in that follows the sky the title draws, less of it in creases (`ambient.rs`, after Belobug's fork; #1093). `VICTORY_AMBIENT=off`: not | Builds here. Title: #1093 on an RTX 4070 (constant form): black half of the room lit, blocks gone, 4 % of a frame; a second tester: faces lit, squares on characters gone |
| **new** Black tiles on hair and faces, lit squares in shadow, other ones at each launch | The title's rays walked whatever bytes lay where the structures would be | A node test answers no hit until the structures are built (#994). `VICTORY_BVH_NO_HIT=0`: as before | Its tests, here. Title: #994, builder off, four machines |
| **new** Glossy that should not be, indoors too | Without rays a glossy surface reflects the open sky | Reflections held to half of the light around (`VICTORY_AMBIENT_REFLECT`, after Belobug) | Title: Belobug |
| **new** Faces, clothes, walls without their textures | The card was full: 13.0 of 13.8 GB from the first world frame, 15,000 textures waited (player's log) | `VICTORY_LEVELS_ON_DEMAND=1` and 95 % of the card are the launcher's default | Project's notes: 2.2 to 3.1 GB less. To be read off the next log |
| **new** Faces that differ at each launch | Reads into the title's pages began before its Submit, with page lists it had not filled | They wait for it (fxceless `085ee7c`, after Marvisak) | Its tests |
| **new** Hair of another colour; 42 programs missing | A branch choosing between two samplers did not translate; a cube level 1.44 times too coarse | #1112 | Its tests, here. Title: #1112 on an RTX 3090 Ti |
| Squares | An indirect compute pass asks for 85,000 to 115,200 groups in x; 65,535 ran | The rest runs as a second dispatch | Here. Player's log: 94,512 to 96,400 asked, one record a frame cut in two |
| Squares, along the bottom | The tiling of the screen-space reflections has only a 64-lane form, which the driver refuses | Can be switched on: `(squares test)`. The passes it feeds no longer walk missing structures | Title: #823, 3 of 3 frames |
| Squares, glow that stays | One NaN or infinity stays in what the title blends into every frame | Pixel exports are guarded (#837) | Here |
| Glossy | A view that picks linear or sRGB of a texture made as the other read with the texture's curve | Such views keep their own, in seven formats | Here, fails without |
| Glossy, wrong material | Normalized vertex data divided by the host is off: 16-bit code 257 came back as 256 | The nearest float (Marvisak `da82e44`) | Here, fails without |
| Wrong vertex data | `tbuffer_load_format_d16` gave the first of two halves as 32 bits | Two 16-bit channels a register (`c430b1d`) | Here |
| Wrong pixels along edges | A helper lane in a waterfall loop took another lane's value | Its own (`0601955`) | Here: 9 of 388 pixels wrong without |
| Shadows | The tiled cascade shadow programs ran approximate on 32-lane cards, one not at all | Exact forms (#1002) | Its tests, here. Title: #1002. Player's log: not refused any more |
| Shadows | One sampler used with and without a comparison; a comparison with a minimum or maximum | #756 | Its tests |
| Missing, wrong, other ones in each launch | A texture uploaded before its data came; a look at 48 KiB of it; a list late | Due again when a read ends; every block looked at, in twelve windows; #606, #632 | A block that comes last is seen in all 64 places |
| Textures that never come | Retries newest first; views that could not be made counted as done | Turns, parts first (NitricAcidDev); tried again | Its tests |
| What is laid over the cutscenes, the intro | A texture that keeps changing was looked at every eighth frame | Looked at by every submission; `VICTORY_INTRO_UPLOADS` | Title: NitricAcidDev, the intro |
| Flickering and missing meshes | Records nothing wrote charged a list's budget; LDS barriers; stale arguments; corner numbers with the edge flag in them | Free; #714; #663, #742, #743; `c0228d9` | Their tests. Player: models good |
| Missing models | The host refuses a pipeline that blends an integer target | #501 | Its test; player's log |

Numbers with `#` are pull requests and issues of fxceless/victory-gta6, the
others commits of Marvisak/VIctory (the fork's parent). Each was read before
it was taken.

## Frame rate and loading

| What | Before | Now |
|------|--------|-----|
| **new** Pass over the buffer views of the heap | 27 to 117 ms a frame in the player's log: it put its own work back in its queue | Only what was written (#1043), under a read lock (#1068). Not timed here |
| **new** Start, second launch | | 2,581 pipelines in 3 s, none compiled (player's log) |
| Submissions by a thread of the layer, fetches by a copy queue | off | on in the launcher. Others: 10 to 21 % less time a frame (fxceless #873) |
| Compare of copies on the card | a thread a 64 KiB piece | 64 threads (#1014); passes over neighbours joined (primary). Primary's other form of it was not raced against this one |
| 3,000 host textures made | 512 ms, 1,062 MB | 42 ms, 976 MB |
| Telling whether a texture changed | 5.3 GB/s | 33 GB/s |

Player's log: 4 to 8 frames a second in the cutscene and gameplay, 2,400 to
11,000 draws; of a 220 ms frame 70 ms preparing, 120 ms on the card, 37 ms
copying back. `Launch VIctory (fast, experimental).bat` adds every other
switch that was measured to pay in the fxceless tree (its run-253 set: 97 to
104 ms a frame in the first rooms).

## Run it

`Launch VIctory.bat` (in `runtime\build`, outside this repository) starts the
diagnostics build `target-v2\diag\victory.exe`; logs are `launch-*.log` in
`%LOCALAPPDATA%\victory-nitric\logs`. Beside it:

- `(safe, to compare)`: the newest changes off, frame rate switches too.
- `(fast, experimental)`: every frame rate switch on.
- `(squares test)`: the reflection tiling runs.

The build before this update is kept as `target-v2\diag\victory-25004ee.exe`.

The stand-in light can be tuned in a running game: two numbers (gain, least)
in `%LOCALAPPDATA%\victory-nitric\light.txt`, `0.36 0.05` by default.

Lines to look for in a log:

| Line | What it says |
|------|--------------|
| `ambient light`, `ambient light follows the sky`, `reflections held` | the stand-in runs; `no stand-in light: other tables` or `another format`: it does not |
| `host copies of title textures take the card's memory level by level` | levels on demand is on; then `in use` and `waiting` beside it |
| `heap buffers` in the frame line | the pass above: passes, ms |
| `indirect compute dispatches` | the largest group count asked for, records cut in two. Over 131,070 is still cut |
| `host pipeline refused` | pipelines missing from the picture. Seen here: `2da4beb386509b77`, `2cc4c197ee29e291`, `02384a60fda6f2d0`, `6b82bc7d22016cb5`, `5dc3b25986622949`, `6f225b641c5b0708`, `926c46bab3b86bd6` |
| `dropped ... a stage of the ray-tracing structure builder` | expected: it is off |

## If it breaks

| Seen | Try | Look at |
|------|-----|---------|
| Anything new | `(safe, to compare)`, then its lines one by one | |
| Too bright, flat or grey in the dark | `light.txt` with smaller numbers, then `VICTORY_AMBIENT=off` | `ambient.rs` |
| Light where a wall should block it | `VICTORY_BVH_NO_HIT=0` | `pipeline.rs` `structures_walked` |
| Dull water or metal | `VICTORY_AMBIENT_REFLECT=0` | `ambient.rs` `hold` |
| A picture without some shadows or reflections | `VICTORY_PROBE_RAYS=1` | `pipeline.rs` `idle_without_rays` |
| The device is lost | `VICTORY_QUEUE2=0`, then `VICTORY_ASYNC_SUBMIT=0` | `queue2.rs`, `queue.rs` |
| A crash or a wrong texture while loading | `VICTORY_LEVELS_ON_DEMAND=0`, then `VICTORY_PLACED_TEXTURES=0` | `tiles.rs`, `placed.rs` |
| Geometry from stale buffers | revert #1043 | `descriptor/host.rs` `slot_flags`, `mirror_heap_buffer` |
| A low frame rate with much uploaded a frame | `VICTORY_PAGE_WRITES=guards`, `VICTORY_LIVELY_TEXTURES=0` | `descriptor/host.rs`, `texture.rs` |
| Colours off | `VICTORY_NAN_GUARD=0` | `descriptor/host.rs` `keeps_gamma_view`, `lower.rs` `nan_guard` |
| Shadows worse than before | | `ir.rs` (#1002) |
| Stops at start, offset `+0x19f4f58` | | the low-address guard |

## Ray tracing

Not run here. The real fix for the black: fxceless #994 (on #854 and #993)
has the title's structure builder running on 32-lane cards for 4 to 11
minutes (RTX 4070, 5060 Ti, 5070 Ti, 5090), at 88 ms more a frame on an RTX
5090 and 2 to 5 times the frame on others, and on this branch's base. A merge
into this tree has 37 conflicts and is under way in a local folder. Needs
#911's cap on constant buffer views, or the device goes.

## Next

1. A run of this build, then the lines above.
2. The ray tracing merge, as a build of its own.
3. Upstream `cd94d35` + `940d32f`: copy-backs compare only what shaders
   marked (365 MB compared a frame for 2 MB, fxceless #650).
4. Device slots `0x1c8` and `0x218`, pipeline creation that is not written:
   the player's log counts them still.
5. fxceless #827's descriptor cache, #1095's bounds on bindless heap reads,
   #1130's detiling of changed pages only.

## Rules here

Push to `primary` and `secondary-test`, never by force; to `main` when the
owner of the request says so. Fetch first: the branches have moved while work
was under way. Nothing of the game goes into Git, nor logs that hold it. The
player uses a controller: do not send input or take the focus.
