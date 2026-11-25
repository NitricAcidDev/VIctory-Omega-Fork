# `secondary-test` and `primary`: how they were merged

Written 25 November 2026 for whoever works on `primary`. Everything here is
AI-assisted. Numbers come from synthetic programs, unit benchmarks and the
timing lines of local runs; nothing of the game is quoted.

## What was done

Later on 25 November, Codex merged fxceless `9c184b3` into `primary`
(AI-assisted). This adds optional texture readback, small-buffer placement
controls, changed-run masks, and optional four-dword shader loads. Existing
readback layouts, ranged mapping leases, buffer read locks, root SRV/UAV
offsets, and streaming fixes are preserved. New readback diagnostics use
ranged leases. The wide-load path retains the selected view's byte bias.
Its helpers live in `emit/raw_loads.rs` to keep `lower.rs` under 512 KiB.

Validation on RTX 5080, driver 617.42: 257 GPU tests, 257 shader tests, and
195 remaining workspace tests passed. The optional wide-load scalar, UAV,
and unaligned-view cases also passed. Clippy completed with warnings, asset
policy passed, and the diagnostics executable was rebuilt. Not run with
the title. Shader source changes invalidate old translations; caches are
retained, and matching host pipelines can still load from their library.
`VICTORY_TEXTURE_BACK`, small-buffer card placement, and shader option bit
64 retain upstream's opt-in defaults. No frame-rate gain is claimed here.

`secondary-test` took `primary` (`ccf29aa`) and was then fast-forwarded onto
it: the two are one commit. Work goes on `primary`.

What both sides had, and what was kept:

| Feature | `primary` (`ccf29aa`) | Here before (`25004ee`) | Kept |
|---------|-----------------------|-------------------------|------|
| Queue of textures picked by index | those uploaded in part first, half of each batch | the others longest due and newest in turns | both |
| Memory under a table's texture | tracked always | tracked 8 frames after a table read | tracked always, looked at after 8 frames, two slots a texture |
| A view that could not be made | tried at every read | tried after 8, 16, ... 256 frames | here |
| What a look at a texture reads | 12 windows of 4 KiB over it | 16 KiB at its start, middle and end, and each level and block | 12 windows, levels and blocks: of 86 blocks of a 2048 texture 12, 86, **86** seen; of its 1,376 pages 12, 103, **105** |
| Pipelines kept by a list | once each (after Belobug) | at every use | primary |
| Readbacks of pieces that touch | joined, direct queue | one by one | primary, on the copy queue too |
| Hash passes over neighbours | joined | joined on request | primary |
| Hash of a piece on the card | 64 threads, tree | 64 threads, one adds up (#1014) | here; not raced |
| Integer targets that blend | own function | own function, the same 21 formats | here, both tests |
| A full card | textures that exist are refreshed (after DexNom) | refused at the limit | primary |
| Descriptor data from a load of another width | yes (after DexNom) | no | primary |
| Wave masks restored from scalar copies | yes (after DexNom) | no | primary |
| Captures on request | without automatic ones | with | primary |

The automatic merge dropped one line of ours without a conflict
(`made[index] = key`, in `mirror`): every slot would have been made again at
each read. It was found and put back before any build had it. Look at what a
merge removes, not only at its conflicts.

If the branches move apart again, merge; do not take one side whole. The
fxceless tree was taken whole once and dropped the low-address guard without a conflict.
Check after any merge that these are still there:

| Look for | Where |
|----------|-------|
| `low_guards::install()` | `apps/victory/src/run.rs` |
| `fn cut(` and both calls of it | `crates/victory-hle/src/modules/xmem.rs` |
| `title_memory_reading(at, len)` (two arguments) | `crates/victory-gpu/src/watch.rs` |
| `SLOT_BIASES` bound and dropped on when unset | `crates/victory-gpu/src/draw.rs`, `pipeline/host.rs` |
| `translation_fingerprint` with the switches found by `build.rs` | `crates/victory-shader/src/emit/mod.rs` |
| `fn filled(` and its call in `slots_written_under` | `crates/victory-gpu/src/descriptor/host.rs` |
| `fn gamma_pair(` | the same |
| `crate::placed::create` | `crates/victory-gpu/src/resource.rs` |
| `fn group_id(` and `fn group_base(` | `lower.rs`, `draw.rs` |
| `rest` in the clamp's source | `crates/victory-gpu/src/indirect.rs` |
| `fn norm_quotient(`, `fn helper_reads_own(`, `fn nan_guard(` | `lower.rs` |
| `fn keeps_changing(` and its two uses | `crates/victory-gpu/src/texture.rs` |
| `TABLED_GRACE` in `texture_slots_remapped` and `slots_written_under` | `crates/victory-gpu/src/descriptor/host.rs` |
| `made[index as usize] = key;` before `make(&view` | the same |
| `stand_in_light` after both kinds of dispatch, `sky_drawn` in `prepare_for` | `crates/victory-gpu/src/draw.rs` |
| `USER_STRUCTURES` asked by `intersect.rs`, answered in `draw.rs` | `crates/victory-shader/src/emit`, `victory-gpu` |

A new `VICTORY_*` switch the translator reads joins the shader cache's key by
itself. Files named `*_tests.rs` in `victory-shader` do not change it.

## New here against `primary`

See the tables of [the handoff](HANDOFF-secondary-test.md). In one line each:

| Change | Here | `primary` before (`ccf29aa`) |
|--------|------|-----------------------|
| fxceless tree | `085ee7c` | `c96b4f2` |
| Light where the sun and lamps do not reach | a stand-in that follows the sky | black |
| Rays while nothing builds their structures | hit nothing | walk what lies there |
| Reflections without rays | held to half of the light around | the open sky's, indoors too |
| Pass over buffer views | what was written | 11,000 to 26,000 slots a frame |
| Reads into the title's pages | wait for its Submit | begin before |
| Normalized vertex data | the nearest float | the host's division (16-bit code 257 reads as 256 on an RTX 5080) |
| Helper lanes in waterfall loops | their own value | another lane's |
| Tiled cascade shadows on 32-lane cards | exact | approximate, one program refused |
| Textures that keep changing | looked at by every submission | every eighth frame, a list late |
| Texture retries | in turns, never given up | newest first; failures counted as done |
| Compare on the card | 64 threads a piece | one |
| Queues | `VICTORY_ASYNC_SUBMIT`, `VICTORY_QUEUE2` there to switch on | not there |
| sRGB and linear views of one texture | kept, seven formats | read in the texture's own |
| Data that arrives after an upload | uploaded | kept as it was, but for 48 KiB looked at (`f42cea6`) |
| Compute records over 65,535 groups | run whole | cut |
| Unfilled indirect records | free | spend the list's budget |
| Host textures | in shared heaps | one by one |
| Texture fingerprint | 33 GB/s | BLAKE3, 5.3 GB/s |
| Bake passes | bounded for the run | 1.5 s a frame |

## Earlier changes of this branch, all still in

| Change | Before | After | Evidence |
|--------|--------|-------|----------|
| Title-memory lock is by range | one lock for all memory | per range | Benchmark `reads_beside_a_remapping`: 1,000 reads waited 18.5 s with the lock on all memory, 44 us on their own. In the last run before this, 2,028 of 2,093 ms a frame of "textures" was this wait |
| Streamed writes use one view per section | map, commit and unmap per request | mapped once | Benchmark `streamed_writes_into_pages`, 64 KiB into resident pages: 18.4 to 2.0 us (1 thread), 332 to 28 us (24 threads) |
| Texture staging buffers are reused | one made, zeroed and freed per upload (83,000 in one run) | kept by size, 512 MB at most; zeroed if the texture brings nothing | Benchmark `upload_costs` on an RTX 5080: 341 to 2 us (64 KiB), 674 to 20 us (512 KiB), 1,349 to 72 us (2 MiB). Test fails without the zeroing |
| Streaming buffers kept per worker | a fresh zeroed buffer per request | reused | Unit tests only; not timed on its own |
| Shader cache keyed on the translator | BLAKE3 of the executable: every rebuild translated everything again | a hash of `victory-shader`'s sources, its switches and the host limits | Unit tests. Also closes a hole: three switches, the view-heap limit and the wave size were not in the old key |
| Shaders that do not translate are kept | refused again at every launch | read back | Unit test |
| Pipeline workers | half the processors, at most 8; 2 MiB stacks | two thirds, at most 16; 16 MiB reserved | Unit test of the count. The launch backlog of about 1,450 pipelines took about 4 minutes with 8. Not yet timed with 16 |
| Pipeline library saved every 60 s while pipelines wait | 20 s | stores wait less often | Not timed |
| Low-address guard restored | dropped by the fxceless merge | back | Its two tests. They failed in 10 of 200 runs when run beside other tests; now 0 of 300 |
| Buffer views off 16 bytes | raw reads through a structured view, undefined | view starts early, shader adds the difference | Read back on an RTX 5080 and WARP |
| Record counts of structured buffers | 24 for 8 elements of 12 bytes on an RTX 5080 | 8 | Read back on an RTX 5080 and WARP |

## Against the codex branches

| Branch | Here? | Comparison |
|--------|-------|------------|
| `codex/fxceless-rendering` (`1684989`) | all of it | The base of the rendering. Better here: every row of the table above. Nothing of it is removed |
| `codex/nvidia-rendering`, `main` (`2577c6e`) | all of it | Same |
| `codex/shader-coverage` (`a71d96a`) | no | Written for the tree before the fxceless merge (`StrideEntry` and raw aliases). Conflicts in `apps/victory/src/main.rs`, `translate.rs` and `emit/lower.rs`. Do not merge it as it is |

What `codex/shader-coverage` has that is worth redoing here: a run-time
stride for a buffer chosen *per lane*. `slot_word` already takes such a
source (`lane_heap_index`), but `picked` accepts only a plain load, so a
per-lane pick of unknown stride is still refused. No program of the last run
failed for that reason. Its wave-size option for offline translation is
already in `primary` (`90228a7`).

## What this does not show

- None of this has been run with the title on the machine it was made on.
  The unit benchmarks say the waits are gone; frame times need a run.
- The write benchmark uses pages that are already resident. The first write
  of a page still faults it in.
- Sixteen workers use more memory and processor time while the title loads.
  `VICTORY_PIPELINE_WORKERS=8` restores the old number.
- Several GPU unit tests crash now and then when run in parallel: 5 of 200
  runs here, 4 of 200 without these changes, none single-threaded.

## Compare a run

The diagnostics build logs `host pipelines all made` with the time spent
compiling, loading and translating and how many translations were kept.
Start twice with the same cache: the second start should keep nearly all.
`texture refreshes per frame` gives the wait for title memory. For the old
behaviour of one part:

| Switch | Effect |
|--------|--------|
| `VICTORY_PIPELINE_WORKERS=8` | the old number of workers |
| `VICTORY_TEXTURE_NO_WAIT=1` | textures do not wait for title memory at all |

## Leads found on the way, not followed

- The title calls two pipeline creation methods that are not implemented
  (device slots `0x1c8` and `0x218`): over 16,000 calls of the first at
  start. Those pipelines' draws are missing.
- Shaders that still do not translate in the last run: 4 "descriptor read
  only as data", 3 `v_cvt_f32_f64`, 1 `v_permlanex16_b32`, 1 half
  interpolation, 1 lane mask exchange, 1 descriptor address read as data.
- `XMemMapPhysicalPages` cannot map a run over placeholders that were split
  before. It was not met in two runs (its warning never appeared).
