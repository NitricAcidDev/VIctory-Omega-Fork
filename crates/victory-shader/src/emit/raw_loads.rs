//! The dwords of one console load, emitted in groups of up to four.

use super::{Attr, Bin, Lower, V};

impl Lower<'_> {
    /// Whether the dwords of one console load are read by one call for up
    /// to four of them (`bitcode::opt::WIDE` of `VICTORY_SHADER_OPT`), not
    /// by a call each.
    pub(super) fn wide_loads() -> bool {
        super::super::shader_opt() & super::super::bitcode::opt::WIDE != 0
    }

    /// `count` dwords from byte `addr` on of the raw view `h`, as integers,
    /// read in the current block by one call for every four of them
    /// ([`Self::wide_loads`]). They are the dwords of ONE console load,
    /// which the console reads as one (address plus 4, 8, ... without
    /// wrapping at 32 bits): the same bytes as a call a dword reads. Loads
    /// of different console instructions are never joined: the program
    /// computes their offsets in 32-bit registers, and `x - 4` and `x` at
    /// `x = 0` are one dword out of the view and its first one, where one
    /// call from `x - 4` on would read both out of it.
    pub(super) fn raw_dwords(&mut self, h: V, addr: V, count: u32) -> Vec<V> {
        let c = self.cur;
        let (u, al) = (self.s.m.undef(self.s.t.i32), self.s.m.i32(4));
        let mut out = Vec::with_capacity(count as usize);
        for first in (0..count).step_by(4) {
            let n = (count - first).min(4);
            let off = self.s.m.i32(4 * first);
            let a = self.s.m.bin(c, Bin::Add, addr, off);
            let mask = self.s.m.i8((1u8 << n) - 1);
            let r = self.s.op(
                c,
                "rawBufferLoad.f32",
                Attr::ReadOnly,
                self.s.t.res_ret,
                139,
                &[h, a, u, mask, al],
            );
            for k in 0..n {
                let x = self.s.m.extract(c, r, k);
                out.push(self.as_int(x));
            }
        }
        out
    }
}
