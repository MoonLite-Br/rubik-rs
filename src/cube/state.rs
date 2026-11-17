use super::moves::Move;

/// 54 facelets, each holding the index (0..6) of the face its colour belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cube {
    pub f: [u8; 54],
}

impl Default for Cube {
    fn default() -> Self {
        Cube::solved()
    }
}

impl Cube {
    pub fn solved() -> Cube {
        let mut f = [0u8; 54];
        for (i, v) in f.iter_mut().enumerate() {
            *v = (i / 9) as u8;
        }
        Cube { f }
    }

    pub fn apply(&mut self, m: Move) {
        let p = m.perm();
        let old = self.f;
        for i in 0..54 {
            self.f[p[i] as usize] = old[i];
        }
    }

    pub fn apply_all(&mut self, ms: &[Move]) {
        for &m in ms {
            self.apply(m);
        }
    }

    /// Every face is a single colour (relative to its own centre, so slice moves are fine).
    pub fn is_solved(&self) -> bool {
        (0..54).all(|i| self.f[i] == self.f[(i / 9) * 9 + 4])
    }
}

/// xorshift64* - good enough for scrambles, no external crate.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// Random-move scramble that never turns the same axis twice in a row.
pub fn random_scramble(rng: &mut Rng, len: usize) -> Vec<Move> {
    const FACES: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];
    let mut out = Vec::with_capacity(len);
    let mut last_axis = 9usize;
    while out.len() < len {
        let f = rng.below(6) as usize;
        if f % 3 == last_axis {
            continue;
        }
        last_axis = f % 3;
        out.push(Move::new(FACES[f], 1 + rng.below(3) as u8));
    }
    out
}
