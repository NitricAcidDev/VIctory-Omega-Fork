//! A compute program's group number counts on from where the host's dispatch
//! begins ([`super::USER_GROUP_BASE`]): a synthetic program lowered to DXIL,
//! validated by `dxil.dll` when present, and run on the host GPU and WARP.

use super::host::{Dispatch, Host, Root, View};
use super::{HostShader, USER_GROUP_BASE, lower, validator};
use crate::native::{ComputeIo, HwStage, NativeProgram, ResourceType, RootParam, RootType, StageIo, UsedResource};
use crate::{io, ir, resource, structure};

/// 32 threads a group: each writes its group's number along x, plus one, at
/// its own dword of a raw buffer (group * 32 + thread).
const WORDS: &[u32] = &[
    0xBE84_0300, // s_mov_b32 s4, s0 (the table)
    0xBE85_0380, // s_mov_b32 s5, 0
    0xF408_0202, 0xFA00_0000, // s_load_dwordx4 s[8:11], s[4:5], 0
    0x7E02_0201, // v_mov_b32 v1, s1 (the group)
    0x3402_0285, // v_lshlrev_b32 v1, 5, v1
    0x4A02_0300, // v_add_nc_u32 v1, v0, v1
    0x3402_0282, // v_lshlrev_b32 v1, 2, v1
    0x7E04_0201, // v_mov_b32 v2, s1
    0x4A04_0481, // v_add_nc_u32 v2, 1, v2
    0x38 << 26 | 0x1C << 18 | 1 << 12, 0x80 << 24 | 2 << 16 | 2 << 8 | 1, // buffer_store_dword v2, v1, s[8:11], 0 offen
    0xBF81_0000, // s_endpgm
];

/// The program, with `constants` root constants more than it reads.
fn build(constants: u32) -> HostShader {
    let native = NativeProgram {
        stage: HwStage::Cs,
        wave64: false,
        vgprs: 8,
        layout: [RootParam {
            root: RootType::Table,
            table_offset_dwords: 0,
            sgprs: 1,
            base_register: 0,
            user_data: 0,
            slot: 0,
        }]
        .into_iter()
        .chain((constants > 0).then_some(RootParam {
            root: RootType::Constants,
            table_offset_dwords: 0,
            sgprs: constants,
            base_register: 0,
            user_data: 1,
            slot: 1,
        }))
        .collect(),
        used: vec![UsedResource {
            kind: ResourceType::Uav,
            root: RootType::Table,
            table_offset_dwords: 0,
            lower_bound: 0,
            stride: 0,
            dwords: 4,
            user_data: 0,
            slot: 0,
        }],
        io: StageIo::Cs(ComputeIo {
            // One user SGPR, then the group's number along x.
            rsrc2: 1 << 1 | 1 << 7,
            threads: [32, 1, 1],
        }),
        code: WORDS.iter().flat_map(|w| w.to_le_bytes()).collect(),
    };
    let p = ir::build(&native.code, native.wave64, native.vgprs).expect("ir::build");
    let st = structure::structure(&p).expect("structure");
    let res = resource::track(&p, &native.layout, native.stage, Some(&st)).expect("track");
    let io = io::raise(&p, &native).expect("io::raise");
    let lowered = lower::lower(&lower::Parts {
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
    .expect("lower");
    HostShader {
        stage: HwStage::Cs,
        dxil: lowered.dxil,
        user_data: lowered.user_data,
        samplers: lowered.samplers,
        static_samplers: Vec::new(),
        root_cbvs: lowered.root_cbvs,
        root_srvs: lowered.root_srvs,
        root_uavs: lowered.root_uavs,
        root_indexed_views: Vec::new(),
    }
}

#[test]
fn groups_are_numbered_from_the_dispatch_base() {
    let shader = build(0);
    let base = shader
        .user_data
        .iter()
        .find(|u| u.root == RootType::Other(USER_GROUP_BASE))
        .expect("the program asks where its dispatch begins");
    assert!(shader.user_data_dwords() > base.index);
    if let Ok(mut v) = validator::Validator::load(validator::Validator::default_path().as_deref()) {
        v.validate(&mut shader.dxil.clone()).expect("DXIL validation failed");
    }
    let _one = super::valu_tests::ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    for (label, warp) in [("hardware", false), ("warp", true)] {
        let Ok(host) = Host::new(warp) else {
            eprintln!("no SM 6.6 {label} device; skipped");
            continue;
        };
        let (root, pso) = host.compute_pipeline(&shader).expect("compute_pipeline");
        for (first, groups) in [(0u32, 2u32), (3, 2), (1, 4)] {
            let mut user = vec![0u32; shader.user_data_dwords() as usize];
            user[base.index as usize] = first;
            let out = host
                .dispatch(
                    &root,
                    &pso,
                    &Dispatch {
                        roots: vec![(0, Root::Constants(user))],
                        views: &[View::RawUav(5 * 32)],
                        samplers: &[],
                        groups: [groups, 1, 1],
                    },
                )
                .unwrap_or_else(|e| panic!("{label}: dispatch failed: {e}"));
            for (i, &got) in out[0].iter().enumerate() {
                let group = i as u32 / 32;
                let expected = if (first..first + groups).contains(&group) { group + 1 } else { 0 };
                assert_eq!(got, expected, "{label}: dword {i} of {groups} groups from {first}");
            }
        }
    }
}

/// A program with most of its root constants taken gets no more of them:
/// its groups are numbered from 0, as they were.
#[test]
fn a_program_with_many_root_constants_asks_for_no_base() {
    let asks = |constants| {
        let shader = build(constants);
        shader.user_data.iter().any(|u| u.root == RootType::Other(USER_GROUP_BASE))
    };
    assert!(asks(0) && !asks(50));
}
