# NVIDIA Series S performance

Measured 27 November 2026 on a GeForce RTX 5060 Ti with 16 GB VRAM and driver 621.24. These are 1920×1080 output runs of the diagnostics build. The lighting trace saw the Series S diffuse resolve at 1280×720.

The table shows medians of logged intervals with at least 4,000 draws per frame. The runs include different camera positions and load states. Use the figures to identify bottlenecks and use a matched save and camera for comparisons between builds.

| Run | Intervals | Median FPS | Median frame | Median CPU submission prep | Median CPU → GPU buffer upload/frame | Median GPU → CPU copy back/frame |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Story, ray tracing on | 19 | 7.1 | 141.1 ms | 75.0 ms | 125.5 MB | 33.5 MB |
| Story, ray tracing off | 28 | 11.4 | 87.2 ms | 63.6 ms | 100.0 MB | 29.6 MB |
| House and porch trace, ray tracing off | 18 | 8.2 | 122.7 ms | 68.7 ms | 101.9 MB | 32.3 MB |

The house trace ranged from 1.5 to 12 FPS in the selected gameplay intervals. Menus were about 30 FPS. Gameplay remains above the 33.3 ms frame budget for stable 30 FPS.

The house trace had no pending pipeline builds in the sampled gameplay intervals, and it reported exhaustion of the shader-visible descriptor slots followed by dropped draws. It recorded the radiance, diffuse resolve, and volumetric lighting corrections. CPU buffer refresh and copy-back were recurring costs, and submission prep alone exceeded the 30 FPS budget. The timing counters cover overlapping work, so their values cannot be added to reconstruct frame time.

### NVIDIA log review, 27 November 2026

An earlier play log contained 9 light intervals (up to 15 draws per frame)
and 139 gameplay intervals (at least 4,000 draws per frame). These are
medians of the recorded intervals, without a matched before-and-after run.

| Interval | Base FPS | Frame time | Submission prep | Texture prep | Copy-back | Draws dropped | Pending pipelines |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Light | 30.0 | 33.3 ms | 0.7 ms | 0.3 ms | 0.0 ms | 0 | 437 |
| Gameplay | 13.7 | 72.9 ms | 43.1 ms | 1.2 ms | 17.7 ms | 188 | 0 |

The gameplay sample puts submission prep over the full 33.3 ms budget.
Pipeline warming can help the opening backlog. Profile submission prep and
buffer copy-back first in a warmed scene.

The later preload reported 25,760 cached shaders and 225 translation failures.
Of those errors, 198 reported containers with no supported shader stage,
22 reported missing wave64-to-wave32 lowering, and five reported descriptors
read as data. The supplied output ends while building the 83,583 recorded
pipelines. It gives no evidence that a specific failed shader caused the
shoreline or vehicle artifact.

A separate audio trace recorded 440,979 XMA voice shutdowns. In 440,903
records the decoder produced nonzero PCM, 440,703 recorded samples taken by
the game, and 200 recorded decoded samples with none taken. No shutdown
record reported a frame decode error. The trace lacks markers for the TV
and phone interactions. Correlate those interactions with voice creation,
PCM produced, and samples consumed before deciding whether the loss is in
XMA or later audio routing.

### VIctoryUnified comparison

VIctoryUnified reports 22–24 base FPS in its RTX 4060 / i7-14700F prison-cell
test at 960×540 internal resolution, and 18–21 FPS walking at two-thirds
scale. The scene and internal resolution differ from Omega's samples.
[VIctoryUnified benchmark notes](https://github.com/Keel62155/VIctoryUnified/blob/e423a308c711fd50d6fd70d2b0d33ae6a097b563/PROGRESS.md)

For a matched test on Omega, alternate runs with **Unified tuning** off
and on at the same checkpoint, using Series S, ray tracing off, a warmed
cache, and the same DLSS mode. Disable frame generation and select
**Unlimited** so the base rate is visible. The preset asks for a 1 GiB GPU
upload budget via Resizable BAR and six recording workers. The runner
keeps its existing allocation fallback and VRAM-scaled buffer limits.
Use `-More @{VICTORY_WINDOWS_BAR_MB='0'}` or
`-More @{VICTORY_THIN_THREADS='8'}` with `play.ps1 -Nvidia -UnifiedTuning`
to isolate either setting. Results on Omega are pending.

The follow-up Series S run with audio tracing active recorded 8.9–11.9 FPS across nine late gameplay intervals. Submission preparation took 41–51 ms per frame and buffer copies uploaded 67–82 MB per frame. The pipeline backlog was clear during those intervals. In the same stretch, the mesh dispatch diagnostics recorded zero records rejected by the per-list work budget. The camera moved during the follow-up, so these ranges describe the bottleneck and do not measure the effect of the renderer changes.

For a repeat, launch `scripts/play.ps1` with `-Console series-s -Build diag -PerformanceLog`, then save timings for the same fixed camera view and pan after the pipeline queue clears. Record the selected graphics profile, driver, save checkpoint, and cache state with the result.
