# NVIDIA runner settings

The F1 menu marks settings that apply immediately with **Live**. Other changes
take effect on the next start after you save. Omega Launcher supplies its
selected options again when you start from the launcher.

| Setting | What it does |
| --- | --- |
| DLSS | Chooses the NVIDIA upscaling mode. DLAA uses native rendering resolution with NVIDIA anti-aliasing. |
| Frame Generation | Chooses how many frames to show per game-rendered frame. CPU simulation and input still run at the base rate. |
| Frame Gen Type | Chooses DLSS or FSR 3 as the interpolation backend. Auto selects a supported backend. |
| MFG Unlocker | Enables the optional third-party frame generation mod for older RTX cards. |
| Frame Rate Cap | Targets a base rendering rate. The 15 FPS profile allows 30 FPS for light frames such as menus and intro video. |
| FPS at the Top | Shows the small base FPS and frame generation counter. |
| Stats Panel (F3) | Shows frame time, draw drops, transfers and memory counters. |
| Fullscreen (F11) | Toggles borderless monitor-sized output while the game runs. |
| Mouse Capture | Captures mouse movement for the game while its window has focus. |
| English Keyboard | Selects an installed US English keyboard layout at startup. |
| Console | Selects the emulated Series S or Series X graphics profile. |
| Series X Picture | Chooses the Series X render and output resolution path. |
| Game Keyboard | Enables Victory keyboard input and keyboard-to-controller emulation. Physical controllers and ScriptHook VI have separate input paths. |
| DLSS Model | Chooses the DLSS preset requested from Streamline. |
| Light Estimate | Sets the strength of Omega's estimate for indirect lighting. |
| Reflection Estimate | Enables the reflection fallback. For opaque surfaces, it checks the visible scene depth along the reflection ray so nearby walls or ceilings can block the sky estimate. Geometry outside the view and transparent glass are outside this check. |
| Fog Cap | Limits the lighting fed into the fog estimate. |
| Ray Tracing | Enables the experimental acceleration-structure and ray tracing path. |
| Water Light | On 32-lane hardware, **Off** skips both experimental tiled water passes, **Transmission Only** runs the base transmission pass, and **Both Passes** also runs tiled local lighting. Save and restart to compare the tiles. The launcher's checkbox selects Off or Both Passes. |
| Water Wave64 | Tries paired-wave lowering for three non-tiled water transmission shaders on 32-lane GPUs. Off by default. Restart to apply. |
| Heat Haze and Decal Defrag | **Left Out** skips both passes. Black road-sign markings have been reported in this mode. **Decals Only** runs decal atlas defragmentation while keeping heat haze off. Save and restart to compare the same sign. |
| Early Depth Test | Allows supported translated pixel shaders to reject hidden pixels before shading. |
| Skip Big Copy-Backs | Avoids some large GPU-to-CPU copies for write-combined memory. |
| Soft Views | Keeps eligible resource views in Victory's bookkeeping and materializes them when needed, reducing waits while copying. |
| Texture No Wait | Tries a nonblocking memory lock. During a remap, it reads through the kernel and retries cold, small failed reads before falling back to the usual streaming behavior. A new texture can appear a frame later. |
| Relay | Overlaps submissions and GPU readback. Higher stages send more work ahead and can expose synchronization artifacts. |
| Card Steps | Does eligible constant-buffer work and, at stage 2, copies on the GPU to avoid CPU waits. |
| Indirect Batch | Groups indirect GPU commands. Higher stages keep more commands in the same batch. |
| Card Compares From (KB) | Sets the minimum buffer size for eligible GPU-side comparisons. |
| Constants Heap in Place | Lets eligible shader constants use the shared title-memory heap directly. |

**Save for Next Start** keeps settings on disk. **Save and Restart Now** also
starts the game again. **Preload Shaders** restarts through the NVIDIA preloader.
**Reset All to Play Defaults** clears the saved menu overrides. **Close**
returns to the game.

Water Light affects shading. Water appearing inland with splashes is a
separate shoreline issue.
