//! Host textures the title fills, placed in heaps that many of them share.
//!
//! A committed texture is a heap of its own, which the system makes and
//! zeroes: 170 to 410 us each on an RTX 5080 (Windows 11), on the thread
//! that records the frame, and a scene brings some 22,000 of them. A placed
//! one in a heap that is there already takes 4 to 5 us. Small ones also
//! start on 4 KiB instead of taking 64 KiB each.
//!
//! A range of a heap is never given out twice: the system zeroes a heap when
//! it makes it, so a texture sampled before its first upload has run reads
//! zeros as a committed one does, and never another texture's bytes. The
//! price is memory of textures let go in a heap where others live. Past
//! [`most_wasted`] of it, textures are committed again until heaps whose
//! last texture went have given theirs back.
//!
//! What keeps a heap: every texture in it holds it through an object of ours
//! in its private data, which the host releases with the texture's last
//! reference, whoever held that (a descriptor slot, a list,
//! [`crate::resource`]).

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use windows::Win32::Graphics::Direct3D12::*;
use windows::core::{GUID, IUnknown, IUnknown_Vtbl, Interface};

/// The size of a heap.
const HEAP: u64 = 32 << 20;
/// The biggest texture placed in one: the end of a heap wasted at most.
const LARGEST: u64 = 4 << 20;
const PAGE: u64 = 4 << 10;
const BLOCK: u64 = 64 << 10;

/// A heap, counted in [`HELD`] while it lives.
struct Heap {
    heap: ID3D12Heap,
}

// SAFETY: a heap is named in device calls only, which any thread may make.
unsafe impl Send for Heap {}
// SAFETY: as above.
unsafe impl Sync for Heap {}

impl Drop for Heap {
    fn drop(&mut self) {
        HELD.fetch_sub(HEAP, Ordering::Relaxed);
    }
}

/// Bytes of the heaps alive, and of the textures alive in them.
static HELD: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);

/// Textures placed, and committed instead for the waste, since the start.
static PLACED: [AtomicU64; 2] = [const { AtomicU64::new(0) }; 2];

/// For the memory line: the heaps' bytes, their textures' bytes; textures
/// placed, and committed instead for the waste, since the start.
#[cfg_attr(not(any(test, feature = "diagnostics")), allow(dead_code))]
pub(crate) fn counts() -> [u64; 4] {
    [&HELD, &LIVE, &PLACED[0], &PLACED[1]].map(|n| n.load(Ordering::Relaxed))
}

/// The heaps textures are placed in now, and how much of each is given out:
/// one for textures that start on 4 KiB, one for the others; the same two in
/// the PC's memory ([`shared_heap`]).
static OPEN: Mutex<[Option<(Arc<Heap>, u64)>; 4]> = Mutex::new([None, None, None, None]);

/// Heaps whose last texture went, to be let go outside that texture's own
/// release.
static GONE: Mutex<Vec<Arc<Heap>>> = Mutex::new(Vec::new());

/// What a placed texture holds through its private data: an `IUnknown` of
/// ours, released when the texture is destroyed.
#[repr(C)]
struct Token {
    vtable: *const IUnknown_Vtbl,
    references: AtomicU32,
    heap: Option<Arc<Heap>>,
    bytes: u64,
}

/// Names our private data on a texture.
const TOKEN: GUID = GUID::from_u128(0x5f0c_9a71_2b6e_4d53_9d1a_3a7e_08c4_61bb);

static TOKEN_VTABLE: IUnknown_Vtbl = IUnknown_Vtbl {
    QueryInterface: query,
    AddRef: add,
    Release: release,
};

unsafe extern "system" fn query(this: *mut c_void, iid: *const GUID, out: *mut *mut c_void) -> windows::core::HRESULT {
    // SAFETY: the caller's pointers, as COM has it; `this` is a token.
    unsafe {
        if out.is_null() || iid.is_null() {
            return windows::Win32::Foundation::E_POINTER;
        }
        if *iid == IUnknown::IID {
            add(this);
            *out = this;
            return windows::Win32::Foundation::S_OK;
        }
        *out = std::ptr::null_mut();
    }
    windows::Win32::Foundation::E_NOINTERFACE
}

unsafe extern "system" fn add(this: *mut c_void) -> u32 {
    // SAFETY: `this` is a token someone holds a reference of.
    unsafe { (*this.cast::<Token>()).references.fetch_add(1, Ordering::Relaxed) + 1 }
}

unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    let token = this.cast::<Token>();
    // SAFETY: as above; the last reference frees what `Token::make` made.
    unsafe {
        let left = (*token).references.fetch_sub(1, Ordering::AcqRel) - 1;
        if left == 0 {
            let mut token = Box::from_raw(token);
            LIVE.fetch_sub(token.bytes, Ordering::Relaxed);
            // The texture is being destroyed around this: its heap is let
            // go afterwards.
            if let Some(heap) = token.heap.take() {
                GONE.lock().unwrap_or_else(|e| e.into_inner()).push(heap);
            }
        }
        left
    }
}

impl Token {
    fn make(heap: Arc<Heap>, bytes: u64) -> IUnknown {
        LIVE.fetch_add(bytes, Ordering::Relaxed);
        let token = Box::new(Token {
            vtable: &TOKEN_VTABLE,
            references: AtomicU32::new(1),
            heap: Some(heap),
            bytes,
        });
        // SAFETY: the object starts with a pointer to its table of methods
        // and holds the one reference handed over here.
        unsafe { IUnknown::from_raw(Box::into_raw(token).cast()) }
    }
}

/// Whether textures are placed at all (`VICTORY_PLACED_TEXTURES=0`: all
/// committed, as before).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_PLACED_TEXTURES").is_none_or(|v| v != "0"))
}

/// The most bytes of heaps that no texture uses: a twentieth of the card's
/// budget, between 128 and 768 MiB (`VICTORY_PLACED_WASTE_MB`).
fn most_wasted() -> u64 {
    static MOST: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *MOST.get_or_init(|| {
        let set = std::env::var("VICTORY_PLACED_WASTE_MB")
            .ok()
            .and_then(|v| v.trim().parse::<u64>().ok());
        if let Some(mb) = set {
            return mb << 20;
        }
        let budget = crate::host::host()
            .and_then(|h| h.video_memory())
            .map_or(0, |[budget, ..]| budget);
        (budget / 20).clamp(128 << 20, 768 << 20)
    })
}

/// Lets go of the heaps whose last texture went since it was last called,
/// once what runs on the card now has finished.
pub(crate) fn let_go() {
    let gone = std::mem::take(&mut *GONE.lock().unwrap_or_else(|e| e.into_inner()));
    if !gone.is_empty() {
        crate::host::retire(gone);
    }
}

/// A texture described by `desc`, in `state`, in a heap with others; `None`
/// if it is not one that can be (a target, depth or unordered access
/// texture, several samples, too big) or is better not. `shared`: in the
/// PC's memory ([`shared_heap`]).
pub(crate) fn create(
    device: &ID3D12Device,
    desc: &D3D12_RESOURCE_DESC,
    state: D3D12_RESOURCE_STATES,
    shared: bool,
) -> Option<ID3D12Resource> {
    const WRITTEN: i32 = D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET.0
        | D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL.0
        | D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0;
    let texture = desc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER
        && desc.Flags.0 & WRITTEN == 0
        && desc.SampleDesc.Count <= 1;
    if !texture || !on() {
        return None;
    }
    let_go();
    let mut desc = *desc;
    let size = |desc: &D3D12_RESOURCE_DESC| {
        // SAFETY: a plain question.
        let info = unsafe { device.GetResourceAllocationInfo(0, &[*desc]) };
        (info.SizeInBytes != u64::MAX && info.SizeInBytes != 0).then_some((info.SizeInBytes, info.Alignment))
    };
    // On 4 KiB if the host takes it so.
    desc.Alignment = PAGE;
    let small = small_enough(&desc)
        .then(|| size(&desc))
        .flatten()
        .filter(|&(_, alignment)| alignment == PAGE);
    let (bytes, alignment) = match small {
        Some(found) => found,
        None => {
            desc.Alignment = 0;
            size(&desc).filter(|&(_, alignment)| alignment == BLOCK)?
        }
    };
    if bytes > LARGEST {
        return None;
    }
    let mut open = OPEN.lock().unwrap_or_else(|e| e.into_inner());
    // What is neither a texture's nor still to be given out.
    let unused: u64 = open.iter().flatten().map(|(_, used)| HEAP - used).sum();
    let wasted = HELD
        .load(Ordering::Relaxed)
        .saturating_sub(LIVE.load(Ordering::Relaxed))
        .saturating_sub(unused);
    if wasted > most_wasted() {
        PLACED[1].fetch_add(1, Ordering::Relaxed);
        return None;
    }
    let kind = usize::from(alignment == BLOCK) + 2 * usize::from(shared);
    let room = open[kind]
        .as_ref()
        .map(|(heap, used)| (heap.clone(), used.next_multiple_of(alignment)))
        .filter(|(_, at)| at + bytes <= HEAP);
    let (heap, at) = match room {
        Some(room) => room,
        None => (Arc::new(make_heap(device, shared)?), 0),
    };
    let mut made: Option<ID3D12Resource> = None;
    // SAFETY: the range lies in the heap and is no other texture's.
    let placed = unsafe { device.CreatePlacedResource(&heap.heap, at, &desc, state, None, &mut made) };
    if let Err(e) = &placed {
        warn_once(e);
    }
    let made = made?;
    let token = Token::make(heap.clone(), bytes);
    // SAFETY: a live texture, which takes a reference of the token.
    if unsafe { made.SetPrivateDataInterface(&TOKEN, &token) }.is_err() {
        // Without it nothing would keep the heap.
        return None;
    }
    // (The heap that was open before lives on in its textures.)
    open[kind] = Some((heap, at + bytes));
    PLACED[0].fetch_add(1, Ordering::Relaxed);
    Some(made)
}

/// Whether a texture's level 0 fits in 64 KiB, what one that starts on 4
/// KiB needs: the host is asked only then (it complains in its messages
/// otherwise).
fn small_enough(desc: &D3D12_RESOURCE_DESC) -> bool {
    let Some(format) = victory_formats::texture::format(desc.Format.0 as u32) else {
        return false;
    };
    let across = |texels: u64| texels.div_ceil(u64::from(format.block.max(1)));
    let layer = across(desc.Width) * across(u64::from(desc.Height)) * u64::from(format.bytes);
    desc.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE2D && layer <= BLOCK
}

/// A heap in the PC's memory that the card reads and writes over the bus
/// and the CPU does not see, so that textures in it have the card's own
/// layout.
pub(crate) fn shared_heap() -> D3D12_HEAP_PROPERTIES {
    D3D12_HEAP_PROPERTIES {
        Type: D3D12_HEAP_TYPE_CUSTOM,
        CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_NOT_AVAILABLE,
        MemoryPoolPreference: D3D12_MEMORY_POOL_L0,
        ..Default::default()
    }
}

fn make_heap(device: &ID3D12Device, shared: bool) -> Option<Heap> {
    let desc = D3D12_HEAP_DESC {
        SizeInBytes: HEAP,
        Properties: if shared {
            shared_heap()
        } else {
            D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                ..Default::default()
            }
        },
        Alignment: BLOCK,
        // (Zeroed: see the head of this file.)
        Flags: D3D12_HEAP_FLAG_ALLOW_ONLY_NON_RT_DS_TEXTURES,
    };
    let mut heap: Option<ID3D12Heap> = None;
    // SAFETY: plain creation.
    if let Err(e) = unsafe { device.CreateHeap(&desc, &mut heap) } {
        warn_once(&e);
    }
    let heap = heap?;
    HELD.fetch_add(HEAP, Ordering::Relaxed);
    Some(Heap { heap })
}

fn warn_once(error: &windows::core::Error) {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| tracing::warn!(%error, "a texture could not be placed in a shared heap; made on its own"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Graphics::Dxgi::Common::*;

    fn desc(side: u32, mips: u16, format: DXGI_FORMAT) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: u64::from(side),
            Height: side,
            DepthOrArraySize: 1,
            MipLevels: mips,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            ..Default::default()
        }
    }

    /// The tests here count heaps of the whole process: one at a time.
    static ONE: Mutex<()> = Mutex::new(());

    /// Many small and middling textures: each holds its own texels while
    /// the others are written, before and after the heaps they began in are
    /// closed; small ones share a 64 KiB block; a texture never written
    /// reads zeros; and the heaps go with their last texture.
    #[test]
    fn textures_in_shared_heaps_keep_their_own_texels() {
        keep_their_own_texels(false);
    }

    /// The same in the PC's memory, which the card reads over the bus.
    #[test]
    fn textures_in_the_pcs_memory_keep_their_own_texels() {
        keep_their_own_texels(true);
    }

    /// A texture too big to be placed is made there on its own, and counts as
    /// the PC's memory, not the card's.
    #[test]
    fn big_textures_are_made_in_the_pcs_memory_too() {
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = crate::host::host() else { return };
        let before = host.video_memory();
        let made: Vec<Option<ID3D12Resource>> = [DXGI_FORMAT_BC7_UNORM, DXGI_FORMAT_BC5_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM]
            .into_iter()
            .map(|format| {
                let mut texture = None;
                // SAFETY: plain creation.
                unsafe {
                    host.device
                        .CreateCommittedResource(
                            &shared_heap(),
                            D3D12_HEAP_FLAG_NONE,
                            &desc(4096, 13, format),
                            D3D12_RESOURCE_STATE_COPY_DEST,
                            None,
                            &mut texture,
                        )
                        .expect("a texture in the PC's memory");
                }
                texture
            })
            .collect();
        assert!(made.iter().all(Option::is_some));
        if let (Some(before), Some(after)) = (before, host.video_memory()) {
            // 133 MiB of them: 4/3 of 16, 16 and 64.
            let (own, shared) = (after[1].saturating_sub(before[1]) >> 20, after[3].saturating_sub(before[3]) >> 20);
            println!("made: {own} MB more of the card's memory, {shared} MB more of the PC's");
            assert!(own < 64, "{own} MB on the card");
            assert!(shared > 100, "{shared} MB in the PC's");
        }
        assert_eq!(crate::descriptor::host::device_removed(), None);
    }

    /// What it costs to fill and to copy out textures in the card's memory
    /// and in the PC's.
    #[test]
    #[ignore = "benchmark"]
    fn what_textures_in_the_pcs_memory_cost() {
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = crate::host::host() else { return };
        for shared in [false, true] {
            let mut rig = crate::inplace::Rig::new(&host.device).unwrap();
            let d = desc(1024, 1, DXGI_FORMAT_BC7_UNORM);
            const N: usize = 256;
            let began = std::time::Instant::now();
            let textures: Vec<_> = (0..N)
                .map(|_| create(&host.device, &d, D3D12_RESOURCE_STATE_COPY_DEST, shared).expect("placed"))
                .collect();
            let made = began.elapsed();
            let target = create(&host.device, &d, D3D12_RESOURCE_STATE_COPY_DEST, false).expect("placed");
            // Every one copied into one on the card: what a full sample of
            // it reads, at least.
            rig.run(|list| unsafe {
                for t in &textures {
                    list.ResourceBarrier(&[crate::host::transition(t, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
                }
            })
            .unwrap();
            let began = std::time::Instant::now();
            for _ in 0..4 {
                rig.run(|list| unsafe {
                    for t in &textures {
                        list.CopyResource(&target, t);
                    }
                })
                .unwrap();
            }
            let read = began.elapsed();
            println!(
                "{}: {N} textures of 1 MiB made in {made:?}; read at {:.1} GB/s",
                if shared { "the PC's memory" } else { "the card's memory" },
                (4 * N) as f64 / 1024.0 / read.as_secs_f64()
            );
        }
    }

    fn keep_their_own_texels(shared: bool) {
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = crate::host::host() else { return };
        let before = counts();
        let mut rig = crate::inplace::Rig::new(&host.device).unwrap();
        let formats = [
            (16u32, DXGI_FORMAT_R8G8B8A8_UNORM),
            (64, DXGI_FORMAT_R8G8B8A8_UNORM),
            (512, DXGI_FORMAT_R8G8B8A8_UNORM),
            (1024, DXGI_FORMAT_R8_UNORM),
        ];
        const N: usize = 160;
        let textures: Vec<ID3D12Resource> = (0..N)
            .map(|i| {
                let (side, format) = formats[i % formats.len()];
                create(&host.device, &desc(side, 1, format), D3D12_RESOURCE_STATE_COPY_DEST, shared).expect("placed")
            })
            .collect();
        let after = counts();
        assert!(after[2] - before[2] >= N as u64);
        assert!(after[1] > before[1] && after[0] >= after[1], "{after:?}");
        // The 80 small ones lie in 4 KiB steps, not in 64 KiB ones.
        let small: u64 = (0..N)
            .filter(|i| formats[i % formats.len()].0 <= 64)
            // SAFETY: a live resource.
            .map(|i| unsafe { host.device.GetResourceAllocationInfo(0, &[textures[i].GetDesc()]) }.SizeInBytes)
            .sum();
        assert!(small <= 80 * (32 << 10), "{small}");
        // Each texture (but every tenth) filled with its own number, in an
        // order other than theirs.
        let pitch = 4096u64;
        let bytes = pitch * 1024;
        let upload = rig.committed(D3D12_HEAP_TYPE_UPLOAD, bytes, D3D12_RESOURCE_STATE_GENERIC_READ).unwrap();
        let mut at = std::ptr::null_mut();
        // SAFETY: the mapping holds the buffer's bytes.
        unsafe { upload.Map(0, None, Some(&mut at)).unwrap() };
        let buffer_at = |buffer: &ID3D12Resource, (side, format): (u32, DXGI_FORMAT)| D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: a borrowed pointer the location never releases.
            pResource: unsafe { std::mem::transmute_copy(buffer) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                    Offset: 0,
                    Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                        Format: format,
                        Width: side,
                        Height: side,
                        Depth: 1,
                        RowPitch: pitch as u32,
                    },
                },
            },
        };
        let texture_at = |texture: &ID3D12Resource| D3D12_TEXTURE_COPY_LOCATION {
            // SAFETY: as above.
            pResource: unsafe { std::mem::transmute_copy(texture) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
        };
        let number = |i: usize| if i.is_multiple_of(10) { 0 } else { i as u8 + 1 };
        for i in (0..N).map(|k| k * 67 % N).filter(|&i| number(i) != 0) {
            let kind = formats[i % formats.len()];
            // SAFETY: the mapped upload buffer; the list is ours and waited for.
            unsafe {
                at.cast::<u8>().write_bytes(number(i), bytes as usize);
                rig.run(|list| {
                    list.CopyTextureRegion(&texture_at(&textures[i]), 0, 0, 0, &buffer_at(&upload, kind), None);
                })
                .unwrap();
            }
        }
        let back = rig.committed(D3D12_HEAP_TYPE_DEFAULT, bytes, D3D12_RESOURCE_STATE_COPY_DEST).unwrap();
        for (i, texture) in textures.iter().enumerate() {
            let kind = formats[i % formats.len()];
            let texel = if kind.1 == DXGI_FORMAT_R8_UNORM { 1 } else { 4 };
            // SAFETY: a list of ours; both resources are live.
            rig.run(|list| unsafe {
                list.ResourceBarrier(&[crate::host::transition(
                    texture,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                )]);
                list.CopyTextureRegion(&buffer_at(&back, kind), 0, 0, 0, &texture_at(texture), None);
            })
            .unwrap();
            for row in [0, kind.0 / 2, kind.0 - 1] {
                let got = rig.fetch(&back, u64::from(row) * pitch, (kind.0 * texel) as usize).unwrap();
                assert!(got.iter().all(|&b| b == number(i)), "texture {i} row {row}: {:?}", &got[..8]);
            }
        }
        // Half of them go: nothing is given back while others live there.
        let held = counts()[0];
        let mut textures = textures;
        textures.truncate(N / 2);
        let_go();
        assert!(counts()[0] + HEAP >= held);
        assert!(counts()[1] < after[1]);
        // All of them: their heaps go, but for those open now.
        drop(textures);
        for _ in 0..500 {
            let_go();
            if counts()[0] <= before[0] + 4 * HEAP && counts()[1] <= before[1] {
                break;
            }
            // (Submissions of other tests may be running.)
            let list = host.new_list().unwrap();
            list.close();
            host.execute(&[&list]);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(counts()[0] <= before[0] + 4 * HEAP, "{:?} {before:?}", counts());
        assert_eq!(crate::descriptor::host::device_removed(), None);
    }

    /// Targets, unordered access textures and big textures are not placed;
    /// past the waste allowed none is.
    #[test]
    fn what_is_not_placed() {
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = crate::host::host() else { return };
        let state = D3D12_RESOURCE_STATE_COPY_DEST;
        for flags in [
            D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
        ] {
            let d = D3D12_RESOURCE_DESC {
                Flags: flags,
                ..desc(64, 1, DXGI_FORMAT_R8G8B8A8_UNORM)
            };
            assert!(create(&host.device, &d, state, false).is_none());
        }
        assert!(create(&host.device, &desc(4096, 1, DXGI_FORMAT_R8G8B8A8_UNORM), state, false).is_none());
        let small = desc(256, 9, DXGI_FORMAT_BC7_UNORM);
        let kept = create(&host.device, &small, state, false).expect("placed");
        // (As if so much of the heaps were unused.)
        HELD.fetch_add(4 << 30, Ordering::Relaxed);
        let refused = create(&host.device, &small, state, false);
        HELD.fetch_sub(4 << 30, Ordering::Relaxed);
        assert!(refused.is_none());
        assert!(create(&host.device, &small, state, false).is_some());
        drop(kept);
    }

    /// The textures of a scene, made as they are with and without this.
    #[test]
    #[ignore = "a measurement: run it by name with --ignored --nocapture"]
    fn placing_costs() {
        let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(host) = crate::host::host() else { return };
        let state = D3D12_RESOURCE_STATE_COPY_DEST;
        let descs: Vec<D3D12_RESOURCE_DESC> = (0..3000u32)
            .map(|i| {
                let (side, mips) = [(64, 5), (256, 9), (512, 10), (1024, 11), (128, 8), (256, 9)][i as usize % 6];
                desc(side, mips, DXGI_FORMAT_BC7_UNORM)
            })
            .collect();
        let started = std::time::Instant::now();
        let placed: Vec<_> = descs.iter().map(|d| create(&host.device, d, state, false)).collect();
        let took = started.elapsed();
        let [held, live, ..] = counts();
        let made = placed.iter().flatten().count();
        eprintln!("{made} placed in {took:?}: heaps {} MiB, textures {} MiB", held >> 20, live >> 20);
        drop(placed);
        let props = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let started = std::time::Instant::now();
        let committed: Vec<Option<ID3D12Resource>> = descs
            .iter()
            .map(|d| {
                let mut t = None;
                // SAFETY: plain creation.
                unsafe { host.device.CreateCommittedResource(&props, D3D12_HEAP_FLAG_NONE, d, state, None, &mut t).unwrap() };
                t
            })
            .collect();
        let took = started.elapsed();
        // SAFETY: a plain question.
        let bytes: u64 = descs.iter().map(|d| unsafe { host.device.GetResourceAllocationInfo(0, &[*d]) }.SizeInBytes).sum();
        eprintln!("{} committed in {took:?}: {} MiB", committed.len(), bytes >> 20);
    }
}
