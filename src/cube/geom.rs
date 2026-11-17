//! Cube geometry. Every facelet is a (cubie position, outward normal) pair on a 3x3x3 grid.
//! Moves are rotations of that geometry, so one definition drives both the logical
//! state (54-byte facelet array) and the 3-D animation.
//!
//! Axes: +x right, +y up, +z towards the viewer (front).
//! Facelet order is the usual Kociemba one: U R F D L B, nine stickers per face, row-major
//! as seen when looking straight at the face in the standard unfolded net.

pub type V3 = [i8; 3];

pub const FU: usize = 0;
pub const FR: usize = 1;
pub const FF: usize = 2;
pub const FD: usize = 3;
pub const FL: usize = 4;
pub const FB: usize = 5;

/// Outward normals in face order U R F D L B.
pub const NORMALS: [V3; 6] = [[0, 1, 0], [1, 0, 0], [0, 0, 1], [0, -1, 0], [-1, 0, 0], [0, 0, -1]];

pub const fn idx(face: usize, row: usize, col: usize) -> usize {
    face * 9 + row * 3 + col
}

pub fn dot(a: V3, b: V3) -> i8 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// One clockwise quarter turn (seen from outside the face) about the outward axis `k`:
/// v' = k (k . v) - k x v
pub fn rot_cw(v: V3, k: V3) -> V3 {
    let d = dot(k, v);
    let c = cross(k, v);
    [k[0] * d - c[0], k[1] * d - c[1], k[2] * d - c[2]]
}

/// Position and normal of facelet `i`.
pub fn facelet(i: usize) -> (V3, V3) {
    let f = i / 9;
    let r = ((i % 9) / 3) as i8;
    let c = (i % 3) as i8;
    let pos = match f {
        0 => [c - 1, 1, r - 1],
        1 => [1, 1 - r, 1 - c],
        2 => [c - 1, 1 - r, 1],
        3 => [c - 1, -1, 1 - r],
        4 => [-1, 1 - r, c - 1],
        _ => [1 - c, 1 - r, -1],
    };
    (pos, NORMALS[f])
}

/// Inverse of [`facelet`].
pub fn facelet_index(p: V3, n: V3) -> usize {
    let f = NORMALS.iter().position(|&m| m == n).expect("normal must be an axis");
    let (x, y, z) = (p[0], p[1], p[2]);
    let (r, c) = match f {
        0 => (z + 1, x + 1),
        1 => (1 - y, 1 - z),
        2 => (1 - y, x + 1),
        3 => (1 - z, x + 1),
        4 => (1 - y, z + 1),
        _ => (1 - y, 1 - x),
    };
    f * 9 + (r as usize) * 3 + (c as usize)
}
