//! A fingerprint of bytes, for telling whether memory of our own title
//! changed since it was last looked at: 128 bits, some 40 GB/s on one thread
//! where BLAKE3 takes 5. Not a hash anyone must not be able to fool: the
//! worst a wrong answer does is leave a texture as it was, about once in
//! 2^64 changes.

/// Odd numbers with no pattern (the first digits of pi, as hexadecimal).
const K: [u64; 8] = [
    0x243f_6a88_85a3_08d3,
    0x1319_8a2e_0370_7345,
    0xa409_3822_299f_31d1,
    0x082e_fa98_ec4e_6c89,
    0x4528_21e6_38d0_1377,
    0xbe54_66cf_34e9_0c6d,
    0xc0ac_29b7_c97c_50dd,
    0x3f84_d5b5_b547_0917,
];

/// The 128 bits of a product, folded.
#[inline(always)]
fn fold(a: u64, b: u64) -> u64 {
    let p = u128::from(a) * u128::from(b);
    p as u64 ^ (p >> 64) as u64
}

/// Every bit of `x` over every bit of the result; no two inputs the same.
fn mix(mut x: u64) -> u64 {
    x = (x ^ x >> 32).wrapping_mul(K[4]);
    x = (x ^ x >> 29).wrapping_mul(K[5]);
    x ^ x >> 32
}

/// What is taken in so far: four lanes of 16 bytes each, side by side.
struct Lanes([u64; 4]);

impl Lanes {
    #[inline(always)]
    fn take(&mut self, block: &[u8; 64]) {
        let word = |at: usize| {
            let mut w = [0; 8];
            w.copy_from_slice(&block[at..at + 8]);
            u64::from_le_bytes(w)
        };
        for (i, lane) in self.0.iter_mut().enumerate() {
            // (The lane added: a factor of zero loses nothing from before.)
            *lane = fold(word(16 * i) ^ K[i], word(16 * i + 8) ^ *lane).wrapping_add(lane.rotate_left(29));
        }
    }
}

/// The fingerprint of `bytes`.
pub(crate) fn print(bytes: &[u8]) -> [u64; 2] {
    let mut lanes = Lanes([K[4], K[5], K[6], K[7]]);
    let mut blocks = bytes.chunks_exact(64);
    for block in &mut blocks {
        if let Ok(block) = block.try_into() {
            lanes.take(block);
        }
    }
    let rest = blocks.remainder();
    if !rest.is_empty() {
        let mut last = [0u8; 64];
        last[..rest.len()].copy_from_slice(rest);
        lanes.take(&last);
    }
    let len = bytes.len() as u64;
    let end = |seed: u64| {
        lanes
            .0
            .iter()
            .enumerate()
            .fold(mix(len ^ seed), |h, (i, lane)| mix(h.wrapping_add(lane.rotate_left(7 * i as u32))).wrapping_add(seed))
    };
    [end(K[0]), end(K[1])]
}

/// The print of a run of pieces, taken one after the other: `so_far` with
/// the print of the next one added.
pub(crate) fn chain(so_far: u64, bytes: &[u8]) -> u64 {
    let [a, b] = print(bytes);
    mix(mix(so_far ^ a).wrapping_add(b))
}

#[cfg(test)]
mod tests {
    use super::print;

    fn bytes(n: usize) -> Vec<u8> {
        let mut x = 0x1234_5678_9abc_def1u64;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x >> 32) as u8
            })
            .collect()
    }

    /// One bit changed anywhere, bytes moved, zeros added: another print,
    /// in both halves.
    #[test]
    fn every_change_shows() {
        for n in [1usize, 7, 8, 15, 16, 17, 63, 64, 65, 127, 128, 129, 300, 4096, 4100] {
            for mut b in [bytes(n), vec![0; n], vec![0xff; n]] {
                let was = print(&b);
                assert_eq!(print(&b), was);
                for at in (0..n).step_by((n / 97).max(1)).chain([n - 1]) {
                    for bit in [0, 3, 7] {
                        b[at] ^= 1 << bit;
                        let now = print(&b);
                        assert!(now[0] != was[0] && now[1] != was[1], "byte {at} bit {bit} of {n}");
                        b[at] ^= 1 << bit;
                    }
                }
                b.push(0);
                assert_ne!(print(&b), was, "a zero more, {n}");
            }
        }
        // Blocks in another order.
        let a = bytes(256);
        for (x, y) in [(0, 1), (1, 3), (0, 2)] {
            let mut b = a.clone();
            for i in 0..64 {
                b.swap(64 * x + i, 64 * y + i);
            }
            assert_ne!(print(&a), print(&b));
        }
        // Lanes of one block in another order.
        for lane in 1..4 {
            let mut b = a.clone();
            for i in 0..16 {
                b.swap(i, 16 * lane + i);
            }
            assert_ne!(print(&a), print(&b));
        }
    }

    /// Prints of nearly alike bytes are spread: no two of 20,000 alike in
    /// either half, and every bit of them about as often set as not.
    #[test]
    fn prints_are_spread() {
        let mut b = vec![0u8; 1024];
        let mut seen = [std::collections::HashSet::new(), std::collections::HashSet::new()];
        let mut set = [[0u32; 64]; 2];
        const N: u32 = 20_000;
        for i in 0..N {
            b[512..516].copy_from_slice(&i.to_le_bytes());
            let p = print(&b);
            for half in 0..2 {
                assert!(seen[half].insert(p[half]));
                for (bit, n) in set[half].iter_mut().enumerate() {
                    *n += (p[half] >> bit & 1) as u32;
                }
            }
        }
        assert!(set.iter().flatten().all(|&n| n.abs_diff(N / 2) < N / 40), "{set:?}");
    }

    #[test]
    #[ignore = "a measurement: run it by name with --ignored --nocapture"]
    fn print_speed() {
        for n in [1 << 10, 64 << 10, 1 << 20, 16 << 20] {
            let b = bytes(n);
            let rounds = ((256 << 20) / n).max(1);
            let started = std::time::Instant::now();
            for _ in 0..rounds {
                std::hint::black_box(print(std::hint::black_box(&b)));
            }
            let s = started.elapsed().as_secs_f64();
            eprintln!("{:>6} KiB: {:.1} GB/s", n >> 10, (rounds * n) as f64 / s / 1e9);
        }
    }
}
