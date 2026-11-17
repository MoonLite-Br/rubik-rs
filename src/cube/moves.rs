//! Move notation: U D L R F B (faces), M E S (slices), x y z (rotations),
//! lowercase u d l r f b or "Rw" style (wide). Suffix ' = counter-clockwise, 2 = half turn.

use super::geom::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub sym: char,
    /// 1 = clockwise, 2 = half turn, 3 = counter-clockwise
    pub turns: u8,
}

impl Move {
    pub const fn new(sym: char, turns: u8) -> Move {
        Move { sym, turns }
    }

    pub fn parse(tok: &str) -> Option<Move> {
        let t = tok.replace('\u{2019}', "'");
        let mut it = t.chars();
        let mut sym = it.next()?;
        let mut rest: String = it.collect();
        if rest.starts_with('w') {
            sym = sym.to_ascii_lowercase();
            let _ = rest.remove(0);
        }
        if !"URFDLBMESxyzurfdlb".contains(sym) {
            return None;
        }
        let turns = match rest.as_str() {
            "" => 1,
            "'" => 3,
            "2" | "2'" | "'2" => 2,
            _ => return None,
        };
        Some(Move { sym, turns })
    }

    pub fn inverse(self) -> Move {
        Move { sym: self.sym, turns: 4 - self.turns }
    }

    pub fn name(&self) -> String {
        let mut s = String::with_capacity(3);
        s.push(self.sym);
        match self.turns {
            3 => s.push('\''),
            2 => s.push('2'),
            _ => {}
        }
        s
    }

    /// Rotation axis (outward normal of the reference face) and layer mask.
    /// Bit (d + 1) of the mask selects cubies whose coordinate along the axis is d.
    pub fn axis_layers(&self) -> (V3, u8) {
        match self.sym {
            'U' => ([0, 1, 0], 0b100),
            'R' => ([1, 0, 0], 0b100),
            'F' => ([0, 0, 1], 0b100),
            'D' => ([0, -1, 0], 0b100),
            'L' => ([-1, 0, 0], 0b100),
            'B' => ([0, 0, -1], 0b100),
            'M' => ([-1, 0, 0], 0b010), // follows L
            'E' => ([0, -1, 0], 0b010), // follows D
            'S' => ([0, 0, 1], 0b010),  // follows F
            'x' => ([1, 0, 0], 0b111),
            'y' => ([0, 1, 0], 0b111),
            'z' => ([0, 0, 1], 0b111),
            'u' => ([0, 1, 0], 0b110),
            'r' => ([1, 0, 0], 0b110),
            'f' => ([0, 0, 1], 0b110),
            'd' => ([0, -1, 0], 0b110),
            'l' => ([-1, 0, 0], 0b110),
            _ => ([0, 0, -1], 0b110), // 'b'
        }
    }

    pub fn in_layers(&self, p: V3) -> bool {
        let (k, mask) = self.axis_layers();
        let bit = (dot(p, k) + 1) as u8;
        mask & (1u8 << bit) != 0
    }

    /// Animation angle (radians, right-handed about the axis) for the whole move.
    pub fn angle(&self) -> f32 {
        -(std::f32::consts::FRAC_PI_2) * self.turns as f32
    }

    /// Facelet permutation: the sticker at `i` moves to `perm[i]`.
    pub fn perm(&self) -> [u8; 54] {
        let (k, _) = self.axis_layers();
        let mut out = [0u8; 54];
        for i in 0..54 {
            let (mut p, mut n) = facelet(i);
            if self.in_layers(p) {
                for _ in 0..self.turns {
                    p = rot_cw(p, k);
                    n = rot_cw(n, k);
                }
            }
            out[i] = facelet_index(p, n) as u8;
        }
        out
    }
}

pub fn parse_alg(s: &str) -> Vec<Move> {
    s.split_whitespace().filter_map(Move::parse).collect()
}

pub fn invert_alg(a: &[Move]) -> Vec<Move> {
    a.iter().rev().map(|m| m.inverse()).collect()
}

pub fn repeat_alg(a: &[Move], n: usize) -> Vec<Move> {
    let mut v = Vec::with_capacity(a.len() * n);
    for _ in 0..n {
        v.extend_from_slice(a);
    }
    v
}
