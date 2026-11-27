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

The follow-up Series S run with audio tracing active recorded 8.9–11.9 FPS across nine late gameplay intervals. Submission preparation took 41–51 ms per frame and buffer copies uploaded 67–82 MB per frame. The pipeline backlog was clear during those intervals. In the same stretch, the mesh dispatch diagnostics recorded zero records rejected by the per-list work budget. The camera moved during the follow-up, so these ranges describe the bottleneck and do not measure the effect of the renderer changes.

For a repeat, launch `scripts/play.ps1` with `-Console series-s -Build diag -PerformanceLog`, then save timings for the same fixed camera view and pan after the pipeline queue clears. Record the selected graphics profile, driver, save checkpoint, and cache state with the result.
