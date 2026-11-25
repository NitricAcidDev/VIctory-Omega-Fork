# Handoff: `secondary-test`

25 November 2026. AI-assisted (Claude Code). For whoever continues here, a
person or an agent. To merge into `primary`, read
[secondary-test-merge-notes.md](secondary-test-merge-notes.md).

## Where it stands

- The base is `primary` (`fb89116`) with the newest fxceless tree (`9376fd6`)
  merged in. On top of it are fixes for what a player reported on this
  branch: surfaces that are glossy and should not be, squares (mainly in
  cutscenes), shadows, flickering meshes, missing textures and models,
  others in each launch, and slow loading.
- **Nothing since `29124aa` has been run with the title on this machine.**
  What is said below rests on unit tests, GPU tests on an RTX 5080 and
  benchmarks. No cause below has been confirmed on a picture: they are the
  faults found in the code that fit what was reported.
- The first start translates every shader again (the translator changed).
  Compare loading times from the second start on.

Machine: Windows 11 x64, RTX 5080 (driver 617.42), 24 logical processors,
64 GB.

## What was reported, and what was done

| Reported | Found | Done | Shown by |
|----------|-------|------|----------|
| Glossy where it should not be | A view that picks linear or sRGB of a texture made as the other read with the texture's curve: 0.5 of a roughness map read 0.21 | Such views keep their own, in RGBA8 (#644) and here in BC1, BC2, BC3, BC7, BGRA8, BGRX8 | Sampled on the GPU in seven formats, both ways; fails without |
| Squares, mainly in cutscenes | Compute records of over 65,535 groups along x were cut: a program with a group for each screen tile leaves the tiles past it as they were | The rest runs as a second dispatch that counts on | 100,000 of 100,000 groups run |
| Squares, missing textures, others in each launch | A texture is uploaded when its memory is mapped; the read into it may finish after. Nothing told of it, and later looks read 48 KiB | The end of a read makes its textures due again (one look for a run of writes); a look reads every mip and every 64 KiB block | A block that comes last is seen in all 64 places; before, more than half were missed |
| The same | A texture over remapped memory was uploaded a list late; a slot found streamed out cleared a view written since; filler words of a description told one texture from itself | #606, #632, part of #676 | Their tests |
| Flickering and missing meshes | Records nothing wrote (all ones) charged a list's budget, after which its indirect draws ran empty; mesh programs lost LDS barriers; arguments and constants from before were kept | Such records are empty and free; #714; #663; #742, #743 | Their tests |
| Missing models | The host refuses a pipeline that blends an integer target | #501 | Its test; seen in this machine's log (one mesh pipeline) |
| Shadows | One sampler used with and without a comparison was one host sampler; a comparison with a minimum or maximum became another kind | #756 | Its tests |
| Loading | See below | | |

Numbers with `#` are pull requests of fxceless/victory-gta6, each reviewed
before it was taken.

## Loading, measured here

| What | Before | Now |
|------|--------|-----|
| 3,000 host textures made | 512 ms, 1,062 MB | 42 ms, 976 MB (heaps they share) |
| Telling whether a texture changed | 5.3 GB/s (BLAKE3), four to five times the detiling | 33 GB/s, and none for an upload that is made once |
| A submission's texture checks | one after another | on seven threads |
| Staging buffers kept | 512 MiB | a 32nd of the memory (2 GiB here) |
| Passes before the title's bake draws | up to 1.5 s in every frame that bakes | 6 s in all, then 20 ms a second |

The last is others' measurement (issue #594 there: half the frame rate, 50 to
110 s of loading), not repeated here.

## Commits since `primary` (`fb89116`)

| Commit | What |
|--------|------|
| `447b915`, `89e5c7b` | The newest fxceless tree |
| `e03d8d0` | Views keep their sRGB conversion; stale constants, arguments and views |
| `459c2e3` | Textures over remapped memory |
| `245a717` | Blocks that arrive after the upload |
| `5ff689e` | Samplers, mesh barriers, `DispatchX` thread counts, and more |
| `b1ac314` | Loading |
| `6dc6e42` | Dispatches over 65,535 groups; unfilled records |
| `bbee73d` | `lower.rs`' tests in their own file (the asset policy's size limit) |
| this one | README rows, this handoff |

## Run it

`Launch VIctory.bat` (in `runtime\build`, outside this repository) starts the
diagnostics build `target-v2\diag\victory.exe`. Logs are `launch-*.log` in
`%LOCALAPPDATA%\victory-nitric\logs`. Beside it:

- `Launch VIctory (safe, to compare).bat`: the newest loading and texture
  changes off. If something is wrong with the first and right with this one,
  it is one of the switches below.
- `Launch VIctory (fast, experimental).bat`: buffer copies refreshed on
  several threads too.

`VIctory RTX 5080 test.cmd` starts **another, older build**.

Lines to look for in a log:

| Line | What it says |
|------|--------------|
| `textures in shared heaps` | heaps, textures in them, textures made on their own for the heaps' waste |
| `writes behind the guards` | how many texture slots were uploaded before the bytes came |
| `indirect compute dispatches` | the largest group count asked for, and how many records a frame are cut in two. **0 means the squares have another cause** |
| `a view reads as its texture's own format` | views whose format is still not kept (first 32) |
| `a texture picked by index changed after its upload` | textures found changed with nothing to tell of it |
| `host pipeline refused` | pipelines that are missing from the picture |
| `host pipelines all made` | times, translations kept and made |

## If it breaks

| Seen | Try | Look at |
|------|-----|---------|
| A crash or a wrong texture while loading | `VICTORY_PLACED_TEXTURES=0` | `placed.rs` |
| The same | `VICTORY_WIDE_TEXTURES=0` | `host.rs` `Batch` |
| A low frame rate with much uploaded a frame | `VICTORY_PAGE_WRITES=guards` | `descriptor/host.rs` `TextureQueue::filled` |
| Rechecks take long | `VICTORY_HEAP_RECHECK_MIPS=0` | `texture.rs` `look_ranges` |
| Wrong constants, flicker | `VICTORY_FILL_ON_CARD=0` | `draw.rs`, #522 |
| Colours too dark or too bright | | `descriptor/host.rs` `keeps_gamma_view` |
| A compute pass wrong on one side of the screen | | `lower.rs` `group_id`, `draw.rs` `group_base` |
| A texture shows another one's picture | `VICTORY_STAGING_POOL_MB=0` | `texture.rs` `SPARE`, `no_leftovers` |
| Access violation reading title memory | | `xmem.rs` `cut` |
| Stops at start, offset `+0x19f4f58` | | the low-address guard |
| The whole PC is slow while it loads | `VICTORY_PIPELINE_WORKERS=8` | |

## Next

1. One run, then the lines above. Confirm or drop each cause on a picture.
2. Nine compute programs of an older cache ask for waves of 64 lanes, which
   this card has not (the driver refuses them; `cached_compute_translations_
   make_host_pipelines` lists them). The base splits such programs now:
   check that none is refused any more.
3. Two mesh programs of that cache write more than 32,768 bytes a group and
   two pack rows of different interpolation (`dxv.exe` says so; the driver
   took them).
4. Device slots `0x1c8` and `0x218`, pipeline creation that is not written.
5. Textures of shared heaps let go leave their range unused until the heap
   goes. Ranges could be given out again once cleared on the card.
6. Not taken, to look at again: #583 (buffer states; many barriers), #604
   (view instancing), the handle change of #576 (1,507 of 1,516 programs
   make pipelines without it), the masks of #646.

## Checked on this commit

`cargo test --workspace` (175 GPU, 102 HLE, 228 shader tests), clippy with
diagnostics (two warnings, both from the fxceless tree), the asset policy.
Built with the pinned toolchain and `--features blake3/pure` (this PC has no
C compiler). Run in parallel the GPU tests crash in 2 to 4 of 100 runs: all
four stacks taken are inside `d3d10warp.dll`, the software renderer some of
them use and the game does not. None crash with `--test-threads=1`.

## Rules here

Push to `secondary-test` only, never by force. Fetch first: the branch has
been replaced while work was under way. Nothing of the game goes into Git.
The player uses a controller: do not send input or take the focus.
