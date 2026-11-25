# Merging `secondary-test`: notes for the agent that does it

Written 25 November 2026 for whoever works on `primary`. Everything here is
AI-assisted. Numbers come from synthetic programs, unit benchmarks and the
timing lines of local runs; nothing of the game is quoted.

## What to do

`secondary-test` contains `primary` (`e0be79c`) and `main` (`2577c6e`), so
either takes it as a fast-forward:

```powershell
git fetch origin
git switch primary
git merge --ff-only origin/secondary-test
```

If `primary` has moved, merge; do not take one side whole. The fxceless tree
was taken whole once and dropped the low-address guard without a conflict.
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

A new `VICTORY_*` switch the translator reads joins the shader cache's key by
itself. Files named `*_tests.rs` in `victory-shader` do not change it.

## New here against `primary`

See the tables of [the handoff](HANDOFF-secondary-test.md). In one line each:

| Change | Here | `primary` (`e0be79c`) |
|--------|------|-----------------------|
| fxceless tree | `d510ee4` | `c96b4f2` |
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
