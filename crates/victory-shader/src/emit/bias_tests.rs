//! Buffer views that do not start on 16 bytes of their host buffer: the
//! host's raw view starts up to 12 bytes early and the shader adds the
//! difference (`shader::SLOT_BIASES`), run on the host GPU.

use super::host::{Dispatch, Host, Root, View};
use super::shader::{SLOT_BIASES, SLOT_STRIDES, slot_word};
use super::{HostShader, lower, validator};
use crate::native::{ComputeIo, StageIo};
use crate::{HwStage, NativeProgram, ResourceType, RootParam, RootType, UsedResource};

/// A program of one thread (constants in `s0..s3`): what `[s2, s3]` (element,
/// byte) of heap slot 1 holds, the dword at byte `s1` of it and its
/// record count go to heap slot 0; 7 goes to `[s2, s3]` of heap slot 2. The
/// descriptors of slots 1 and 2 come from a table whose used-resource entries
/// give `stride`, or (not `table`) are picked from the view heap, slot 1 by
/// `s0`.
fn program(table: bool, stride: u32) -> HostShader {
    let load: [u32; 2] = if table {
        [0xF408_0202, 0xFA00_0020] // s_load_dwordx4 s[8:11], s[4:5], 0x20
    } else {
        [0xF408_0202, 0x0C00_0000] // s_load_dwordx4 s[8:11], s[4:5], s6
    };
    let records: &[u32] = &[
        0x7E0A_020A, // v_mov_b32 v5, s10
        0xE070_0008,
        0x8003_0500, // buffer_store_dword v5, s[12:15], 8
    ];
    let words = [
        &[
            0xBE85_0380, // s_mov_b32 s5, 0
            0x8F06_8500, // s_lshl_b32 s6, s0, 5
        ][..],
        &load,
        &[
            0xF408_0302,
            0xFA00_0000, // s_load_dwordx4 s[12:15], s[4:5], 0x0
            0xF408_0502,
            0xFA00_0040, // s_load_dwordx4 s[20:23], s[4:5], 0x40
            0x7E00_0202, // v_mov_b32 v0, s2
            0x7E02_0203, // v_mov_b32 v1, s3
            0xE030_3000,
            0x8002_0200, // buffer_load_dword v2, v[0:1], s[8:11], 0 idxen offen
            0xF420_0404,
            0x0200_0000, // s_buffer_load_dword s16, s[8:11], s1
            0x7E06_0210, // v_mov_b32 v3, s16
            0xE070_0000,
            0x8003_0200, // buffer_store_dword v2, s[12:15], 0
            0xE070_0004,
            0x8003_0300, // buffer_store_dword v3, s[12:15], 4
        ],
        records,
        &[
            0x7E08_0287, // v_mov_b32 v4, 7
            0xE070_3000,
            0x8005_0400, // buffer_store_dword v4, v[0:1], s[20:23], 0 idxen offen
            0xBF81_0000, // s_endpgm
        ],
    ]
    .concat();
    let param = |root, sgprs, user_data, slot| RootParam {
        root,
        table_offset_dwords: 0,
        sgprs,
        base_register: 0,
        user_data,
        slot,
    };
    let used = |kind, table_offset_dwords, stride| UsedResource {
        kind,
        root: RootType::Table,
        table_offset_dwords,
        lower_bound: 0,
        stride,
        dwords: 4,
        user_data: 4,
        slot: 1,
    };
    let (root, used) = if table {
        let used = vec![
            used(ResourceType::Uav, 0, 1),
            used(ResourceType::Srv, 8, stride),
            used(ResourceType::Uav, 16, stride),
        ];
        (RootType::Table, used)
    } else {
        (RootType::ResourceHeap, vec![])
    };
    let native = NativeProgram {
        stage: HwStage::Cs,
        wave64: false,
        vgprs: 8,
        layout: vec![
            param(RootType::Constants, 4, 0, 0),
            param(root, 1, 4, 1),
        ],
        used,
        io: StageIo::Cs(ComputeIo {
            rsrc2: 5 << 1,
            threads: [1, 1, 1],
        }),
        code: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
    };
    let p = crate::ir::build(&native.code, native.wave64, native.vgprs).unwrap();
    let st = crate::structure::structure(&p).unwrap();
    let res = crate::resource::track(&p, &native.layout, native.stage, Some(&st)).unwrap();
    let io = crate::io::raise(&p, &native).unwrap();
    let mut l = lower::lower(&lower::Parts {
        native: &native,
        p: &p,
        st: &st,
        io: &io,
        res: &res,
        inputs: Vec::new(),
        outputs: Vec::new(),
        prims: Vec::new(),
        mesh_stride: 0,
    })
    .unwrap();
    if let Ok(mut v) = validator::Validator::load(validator::Validator::default_path().as_deref()) {
        v.validate(&mut l.dxil).unwrap();
    }
    HostShader {
        stage: HwStage::Cs,
        dxil: l.dxil,
        user_data: l.user_data,
        samplers: Vec::new(),
        static_samplers: Vec::new(),
        root_cbvs: l.root_cbvs,
        root_srvs: l.root_srvs,
        root_uavs: l.root_uavs,
        root_indexed_views: Vec::new(),
    }
}

/// Only a program that can meet such a view pays for the look-up, and only
/// one that picks buffers asks for the whole heap to be made ready.
#[test]
fn only_strides_off_16_bytes_are_looked_up() {
    assert_eq!(program(true, 12).root_srvs, [SLOT_BIASES]);
    assert_eq!(program(true, 4).root_srvs, [SLOT_BIASES]);
    assert!(program(true, 16).root_srvs.is_empty());
    assert!(program(true, 48).root_srvs.is_empty());
    assert_eq!(program(false, 0).root_srvs, [SLOT_STRIDES]);
}

/// Structured views starting anywhere on 4 bytes are read and written at
/// their own bytes, through tables and picked by index; reads past the last
/// element give zeros.
#[test]
fn views_off_16_bytes_reach_their_own_bytes() {
    let data: Vec<u32> = (100..228).collect();
    // Stride, the view's first element and its elements.
    let views = [
        (12, 0, 8),
        (12, 3, 8),
        (12, 1, 8),
        (12, 2, 8),
        (20, 1, 5),
        (4, 5, 9),
        (8, 1, 6),
        (16, 3, 5),
    ];
    let _one = super::valu_tests::ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for warp in [false, true] {
        let Ok(host) = Host::new(warp) else { continue };
        for (stride, first, count) in views {
            for table in [true, false] {
                let cs = program(table, stride);
                let (root, pso) = host.compute_pipeline(&cs).unwrap();
                let bias = first * stride % 16;
                let window = |uav| View::Window {
                    data: &data,
                    dword: (first * stride - bias) / 4,
                    dwords: (count * stride + bias) / 4,
                    uav,
                };
                let word = slot_word(stride, bias);
                let strides = vec![slot_word(4, 0), word, word, 0];
                let off = if stride > 4 { 4 } else { 0 };
                // Element 2, the view's dword 5; then past its end.
                for (element, byte) in [(2, 20), (count, count * stride)] {
                    let inside = element < count;
                    let mut user = vec![0; cs.user_data_dwords() as usize];
                    user[..4].copy_from_slice(&[1, byte, element, off]);
                    let mut roots = vec![(0, Root::Constants(user))];
                    if !cs.root_srvs.is_empty() {
                        let data = strides.clone();
                        roots.push((1, Root::View { srv: true, data }));
                    }
                    let out = host
                        .dispatch(
                            &root,
                            &pso,
                            &Dispatch {
                                roots,
                                views: &[View::RawUav(4), window(false), window(true)],
                                samplers: &[],
                                groups: [1, 1, 1],
                            },
                        )
                        .unwrap();
                    let at = |b: u32| data[((first * stride + b) / 4) as usize];
                    let what = format!("stride {stride} from {first}, table {table}, element {element}, warp {warp}");
                    if inside {
                        assert_eq!(out[0][0], at(element * stride + off), "{what}: element");
                        assert_eq!(out[0][1], at(byte), "{what}: scalar load");
                    } else {
                        assert_eq!(out[0][..2], [0, 0], "{what}: past the end");
                    }
                    assert_eq!(out[0][2], count, "{what}: records");
                    let mut want = vec![0; data.len()];
                    if inside {
                        want[((first * stride + element * stride + off) / 4) as usize] = 7;
                    }
                    assert_eq!(out[1], want, "{what}: store");
                }
            }
        }
    }
}
