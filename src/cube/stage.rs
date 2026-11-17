//! Beginner-method stage detection. "First layer" is the D face (white in the default scheme),
//! "last layer" is the U face (yellow). Everything is compared with the face's own centre,
//! so it stays correct whatever the colour scheme.

use super::geom::*;
use super::state::Cube;

const D_EDGES: [[usize; 2]; 4] = [
    [idx(FD, 0, 1), idx(FF, 2, 1)],
    [idx(FD, 1, 2), idx(FR, 2, 1)],
    [idx(FD, 2, 1), idx(FB, 2, 1)],
    [idx(FD, 1, 0), idx(FL, 2, 1)],
];
const D_CORNERS: [[usize; 3]; 4] = [
    [idx(FD, 0, 2), idx(FF, 2, 2), idx(FR, 2, 0)],
    [idx(FD, 0, 0), idx(FF, 2, 0), idx(FL, 2, 2)],
    [idx(FD, 2, 2), idx(FR, 2, 2), idx(FB, 2, 0)],
    [idx(FD, 2, 0), idx(FL, 2, 0), idx(FB, 2, 2)],
];
const MID_EDGES: [[usize; 2]; 4] = [
    [idx(FF, 1, 2), idx(FR, 1, 0)],
    [idx(FF, 1, 0), idx(FL, 1, 2)],
    [idx(FB, 1, 0), idx(FR, 1, 2)],
    [idx(FB, 1, 2), idx(FL, 1, 0)],
];
const U_FACE: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];

fn ok(c: &Cube, ids: &[usize]) -> bool {
    ids.iter().all(|&i| c.f[i] == c.f[(i / 9) * 9 + 4])
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Progress {
    pub cross: bool,
    pub layer1: bool,
    pub f2l: bool,
    pub oll: bool,
    pub solved: bool,
}

pub fn progress(c: &Cube) -> Progress {
    let cross = D_EDGES.iter().all(|e| ok(c, e));
    let layer1 = cross && D_CORNERS.iter().all(|e| ok(c, e));
    let f2l = layer1 && MID_EDGES.iter().all(|e| ok(c, e));
    let oll = f2l && ok(c, &U_FACE);
    Progress { cross, layer1, f2l, oll, solved: c.is_solved() }
}
