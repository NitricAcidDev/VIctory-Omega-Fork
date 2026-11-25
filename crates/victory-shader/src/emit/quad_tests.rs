//! Pixel programs that read their quad neighbours' registers (the console's
//! compiler takes derivatives of computed values so), drawn on the host GPU
//! and WARP: along a triangle's edge the neighbours are helpers.

use super::bitcode::{Attr, Term};
use super::container::SigElement;
use super::host::{Draw, Host};
use super::shader::Shader;
use super::{HostShader, container, emit, signature, validator};
use crate::dxbc::{Container, FourCC};
use crate::native::{HwStage, NativeProgram, PsInput, StageIo, Usage};
use crate::{io, ir, resource, structure};

/// `SPI_PS_INPUT_ADDR`: i and j at the centre, in v0 and v1.
const PERSP_CENTER: u32 = 1 << 1;

/// The target's width and height.
const SIZE: u32 = 32;

fn element(name: &str, kind: u8, row: u32) -> SigElement {
    SigElement {
        name: name.into(),
        index: 0,
        kind,
        comp: 9,
        interp: 0,
        row: Some(row),
        col: 0,
        cols: 4,
        usage: 0,
        half: false,
    }
}

/// Translates a pixel program with one float4 attribute and one target.
fn pixel(words: &[u32]) -> HostShader {
    let native = NativeProgram {
        stage: HwStage::Ps,
        wave64: false,
        vgprs: 12,
        layout: vec![],
        used: vec![],
        io: StageIo::Ps {
            inputs: vec![PsInput {
                usage: Usage::Generic,
                row: 0,
                mask: 0b1111,
                attr: 0,
                flat: false,
                fp16: false,
                pack2: false,
                row2: 0,
            }],
            input_addr: PERSP_CENTER,
            db_shader_control: 0,
            patches: vec![],
        },
        code: words.iter().flat_map(|w| w.to_le_bytes()).collect(),
    };
    let mut target = element("SV_Target", 16, 0);
    target.usage = 15;
    let isg1 = container::signature(&[element("TEXCOORD", 0, 0)], true);
    let osg1 = container::signature(&[target], false);
    let bytes = crate::dxbc::build(&[(FourCC::ISG1, &isg1), (FourCC::OSG1, &osg1)]);
    let c = Container::parse(&bytes).unwrap();
    let p = ir::build(&native.patched_code(), native.wave64, native.vgprs).expect("ir::build");
    let res = resource::track(&p, &native.layout, native.stage, None).expect("track");
    let io = io::raise(&p, &native).expect("raise");
    let st = structure::structure(&p).expect("structure");
    emit(&c, &native, &p, &st, &io, &res).expect("emit")
}

/// A vertex shader with a position and a float4 in, for the inputs of `ps`.
fn vertex_shader(ps: &HostShader) -> HostShader {
    let parsed = Container::parse(&ps.dxil).unwrap();
    let inputs = signature(parsed.part(FourCC::ISG1).unwrap().data).unwrap();
    let coord = inputs.iter().find(|e| e.kind == 0).expect("the attribute").clone();
    let row = inputs.iter().filter_map(|e| e.row).max().unwrap() + 1;
    let position = inputs
        .iter()
        .find(|e| e.kind == 3)
        .cloned()
        .unwrap_or_else(|| element("SV_Position", 3, row));
    let mut vs = Shader::new(false);
    vs.inputs = vec![
        SigElement {
            usage: 15,
            ..element("POSITION", 0, 0)
        },
        SigElement {
            usage: 15,
            ..element("COORD", 0, 1)
        },
    ];
    vs.outputs = vec![
        SigElement {
            interp: 4,
            usage: 15,
            ..position
        },
        SigElement {
            interp: 2,
            usage: 15,
            ..coord
        },
    ];
    let b = vs.entry;
    for id in 0..2 {
        for col in 0..4 {
            let args = [vs.m.i32(id), vs.m.i32(0), vs.m.i8(col), vs.m.undef(vs.t.i32)];
            let v = vs.op(b, "loadInput.f32", Attr::ReadNone, vs.t.f32, 4, &args);
            let args = [vs.m.i32(id), vs.m.i32(0), vs.m.i8(col), v];
            vs.op(b, "storeOutput.f32", Attr::NoUnwind, vs.t.void, 5, &args);
        }
    }
    vs.m.terminate(b, Term::Ret);
    HostShader {
        stage: HwStage::Vs,
        dxil: vs.finish(),
        user_data: Vec::new(),
        samplers: Vec::new(),
        static_samplers: Vec::new(),
        root_cbvs: Vec::new(),
        root_srvs: Vec::new(),
        root_uavs: Vec::new(),
        root_indexed_views: Vec::new(),
    }
}

/// `v_mov_b32 v<dst>, v<src>` with DPP `quad_perm`, all rows and banks.
fn quad_perm(dst: u32, src: u32, lanes: [u32; 4]) -> [u32; 2] {
    let ctrl = lanes[0] | lanes[1] << 2 | lanes[2] << 4 | lanes[3] << 6;
    [
        0x7E00_0000 | dst << 17 | 1 << 9 | 0xFA,
        src | ctrl << 8 | 1 << 19 | 0xFF << 24,
    ]
}

/// v2, v3 = the attribute's x and y (a pixel's place in the target), and
/// v4 = x * x + y.
fn computed() -> Vec<u32> {
    vec![
        0xC808_0000,                            // v_interp_p1_f32 v2, v0, attr0.x
        0xC809_0001,                            // v_interp_p2_f32 v2, v1, attr0.x
        0xC80C_0100,                            // v_interp_p1_f32 v3, v0, attr0.y
        0xC80D_0101,                            // v_interp_p2_f32 v3, v1, attr0.y
        8 << 25 | 4 << 17 | 2 << 9 | (256 + 2), // v_mul_f32 v4, v2, v2
        3 << 25 | 4 << 17 | 3 << 9 | (256 + 4), // v_add_f32 v4, v4, v3
    ]
}

/// What [`computed`] gives for the pixel (x, y), drawn or a helper.
fn value(x: u32, y: u32) -> f32 {
    (x as f32 + 0.5) * (x as f32 + 0.5) + y as f32 + 0.5
}

/// Draws one triangle whose edges cross quads with `words` on the hardware
/// and WARP; `want` is what a drawn pixel must hold.
fn check(words: &[u32], want: impl Fn(u32, u32) -> [f32; 4]) {
    let ps = pixel(words);
    let vs = vertex_shader(&ps);
    if let Ok(mut v) = validator::Validator::load(validator::Validator::default_path().as_deref()) {
        v.validate(&mut ps.dxil.clone()).expect("the pixel program");
        v.validate(&mut vs.dxil.clone()).expect("the vertex shader");
    }
    let vertex = |x: f32, y: f32| {
        let half = SIZE as f32 / 2.0;
        [x, y, 0.5, 1.0, (x + 1.0) * half, (1.0 - y) * half, 0.0, 1.0]
    };
    let vertices = [vertex(-0.9, 0.9), vertex(0.83, -0.7), vertex(-0.8, -0.95)].concat();
    for (label, warp) in [("hardware", false), ("warp", true)] {
        let Ok(host) = Host::new(warp) else {
            eprintln!("no SM 6.6 {label} device; skipped");
            continue;
        };
        let (root, pso) = host.pipeline(&vs, &ps).expect("pipeline");
        let draw = Draw {
            roots: Vec::new(),
            textures: &[],
            samplers: &[],
            vertices: &vertices,
            stride: 32,
            size: (SIZE, SIZE),
            mesh_groups: 0,
        };
        let texels = host.draw(&root, &pso, &draw).expect("draw");
        let covered = |x: u32, y: u32| texels[(y * SIZE + x) as usize][0] != 0.0;
        let (mut drawn, mut edge, mut wrong) = (0, 0, Vec::new());
        for (i, t) in texels.iter().enumerate() {
            let (x, y) = (i as u32 % SIZE, i as u32 / SIZE);
            if !covered(x, y) {
                continue;
            }
            drawn += 1;
            edge += u32::from((0..4).any(|k| !covered((x & !1) + k % 2, (y & !1) + k / 2)));
            let want = want(x, y);
            if !t.iter().zip(want).all(|(a, b)| (a - b).abs() < 0.1) {
                wrong.push(format!("({x}, {y}): {t:?}, not {want:?}"));
            }
        }
        assert!(
            drawn > 100 && edge > 20,
            "{label}: {drawn} pixels drawn, {edge} in quads an edge crosses"
        );
        assert!(
            wrong.is_empty(),
            "{label}: {} of {drawn} pixels wrong ({edge} in quads an edge crosses), the first {}",
            wrong.len(),
            wrong[0]
        );
    }
}

/// A value each pixel computes, read from the neighbours across x and y and
/// from the quad's first pixel.
#[test]
fn quad_reads_reach_the_helpers_along_a_triangle_edge() {
    let mut words = computed();
    words.extend(quad_perm(5, 4, [1, 0, 3, 2]));
    words.extend(quad_perm(6, 4, [2, 3, 0, 1]));
    words.extend(quad_perm(7, 4, [0, 0, 0, 0]));
    words.extend([0xF800_180F, 0x0706_0504, 0xBF81_0000]); // exp mrt0 v4..v7 done vm
    check(&words, |x, y| {
        [value(x, y), value(x ^ 1, y), value(x, y ^ 1), value(x & !1, y & !1)]
    });
}

/// A waterfall loop over a number each pixel has (its column): every lane
/// takes the first lane's number and does its part when it is its own. A
/// helper lane leaves the loop too, with its own number, which its
/// neighbour then reads.
#[test]
fn helper_lanes_leave_a_waterfall_loop_with_their_own_value() {
    let mut words = computed();
    words.extend([
        0x7E08_0F02, // v_cvt_u32_f32 v4, v2
        0xBE88_037E, // s_mov_b32 s8, exec_lo
        // the loop
        0x7E04_0504, // v_readfirstlane_b32 s2, v4
        0x7D84_0802, // v_cmp_eq_u32 vcc_lo, s2, v4
        0xBE83_3C6A, // s_and_saveexec_b32 s3, vcc_lo
        0x7E0A_0C02, // v_cvt_f32_u32 v5, s2
        0x897E_037E, // s_xor_b32 exec_lo, exec_lo, s3
        0xBF89_FFFA, // s_cbranch_execnz the loop
        0xBEFE_0308, // s_mov_b32 exec_lo, s8
        0x7E08_0D04, // v_cvt_f32_u32 v4, v4
    ]);
    // v4 = the column + 1, so that column 0 is told from what is not drawn.
    words.push(3 << 25 | 4 << 17 | 4 << 9 | 242); // v_add_f32 v4, 1.0, v4
    words.extend(quad_perm(6, 5, [1, 0, 3, 2]));
    words.extend(quad_perm(7, 5, [2, 3, 0, 1]));
    words.extend([0xF800_180F, 0x0706_0504, 0xBF81_0000]); // exp mrt0 v4..v7 done vm
    check(&words, |x, _| [x as f32 + 1.0, x as f32, (x ^ 1) as f32, x as f32]);
}
