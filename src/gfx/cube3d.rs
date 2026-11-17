//! 3-D cube renderer: 26 cubies, painter's algorithm, perspective projection,
//! per-face Lambert shading and a soft ground shadow. A layer turn is drawn by rotating the
//! cubies of that layer about the move axis by `angle * progress`, using the same layer
//! definition as the logical model (Move::in_layers), so animation and state never disagree.

use super::canvas::Canvas;
use crate::cube::geom::{dot, facelet_index, NORMALS, V3};
use crate::cube::{Cube, Move};
use crate::theme::Palette;

type F3 = [f32; 3];

#[derive(Clone, Copy)]
pub struct ViewRot {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for ViewRot {
    fn default() -> Self {
        ViewRot { yaw: -0.62, pitch: 0.52 }
    }
}

fn f3(v: V3) -> F3 {
    [v[0] as f32, v[1] as f32, v[2] as f32]
}

fn dot3(a: F3, b: F3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Rodrigues rotation of `v` about unit axis `k` by `ang` radians (right-handed).
fn rot_axis(v: F3, k: F3, ang: f32) -> F3 {
    let (s, c) = ang.sin_cos();
    let d = dot3(k, v);
    let cr = [k[1] * v[2] - k[2] * v[1], k[2] * v[0] - k[0] * v[2], k[0] * v[1] - k[1] * v[0]];
    [
        v[0] * c + cr[0] * s + k[0] * d * (1.0 - c),
        v[1] * c + cr[1] * s + k[1] * d * (1.0 - c),
        v[2] * c + cr[2] * s + k[2] * d * (1.0 - c),
    ]
}

type Spin = Option<(Move, V3, f32)>;

/// Model space -> view space for a point/direction belonging to cubie `p`.
fn xf(spin: &Spin, p: V3, v: F3, view: ViewRot) -> F3 {
    let mut v = v;
    if let Some((m, k, ang)) = spin {
        if m.in_layers(p) {
            v = rot_axis(v, f3(*k), *ang);
        }
    }
    rot_axis(rot_axis(v, [0.0, 1.0, 0.0], view.yaw), [1.0, 0.0, 0.0], view.pitch)
}

fn tangents(face: usize) -> (F3, F3) {
    match face {
        0 | 3 => ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        1 | 4 => ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
        _ => ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    }
}

/// Draw the cube centred in `area` = (x, y, w, h). `anim` = (move, eased progress 0..1).
pub fn draw_cube(cv: &mut Canvas, pal: &Palette, cube: &Cube, anim: Option<(Move, f32)>, view: ViewRot, area: (f32, f32, f32, f32)) {
    let (ax, ay, aw, ah) = area;
    let cx = ax + aw * 0.5;
    let cy = ay + ah * 0.5;
    let ppu = aw.min(ah) * 0.185; // pixels per cube unit at z = 0
    let dist = 9.0f32; // camera distance
    let ll = [-0.35f32, 0.75, 0.55];
    let ln = (ll[0] * ll[0] + ll[1] * ll[1] + ll[2] * ll[2]).sqrt();
    let light = [ll[0] / ln, ll[1] / ln, ll[2] / ln];

    let spin: Spin = anim.map(|(m, t)| (m, m.axis_layers().0, m.angle() * t));
    let proj = |v: F3| -> (f32, f32) {
        let s = dist / (dist - v[2]);
        (cx + v[0] * ppu * s, cy - v[1] * ppu * s)
    };

    // soft ground shadow
    let g = rot_axis(rot_axis([0.0, -1.85, 0.0], [0.0, 1.0, 0.0], view.yaw), [1.0, 0.0, 0.0], view.pitch);
    let (sx, sy) = proj(g);
    let squash = 0.28 + 0.5 * view.pitch.sin().abs();
    for i in 0..6 {
        let k = i as f32 / 5.0;
        cv.fill_ellipse(sx, sy, ppu * (2.5 - 1.0 * k), ppu * (2.5 - 1.0 * k) * squash, pal.shadow.alpha(if pal.dark { 34 } else { 16 }));
    }

    // far-to-near order of the 26 cubies
    let mut order: Vec<(f32, V3)> = Vec::with_capacity(26);
    for x in -1..=1i8 {
        for y in -1..=1i8 {
            for z in -1..=1i8 {
                if x == 0 && y == 0 && z == 0 {
                    continue;
                }
                let p = [x, y, z];
                let c = xf(&spin, p, f3(p), view);
                order.push((c[2], p));
            }
        }
    }
    order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    for &(_, p) in &order {
        let c = xf(&spin, p, f3(p), view);
        for (f, &n) in NORMALS.iter().enumerate() {
            let nv = xf(&spin, p, f3(n), view);
            let fc = [c[0] + nv[0] * 0.5, c[1] + nv[1] * 0.5, c[2] + nv[2] * 0.5];
            // back-face culling against the camera at (0, 0, dist)
            if nv[0] * (0.0 - fc[0]) + nv[1] * (0.0 - fc[1]) + nv[2] * (dist - fc[2]) <= 0.0 {
                continue;
            }
            let (u, v) = tangents(f);
            let uv = xf(&spin, p, u, view);
            let vv = xf(&spin, p, v, view);
            let pt = |a: f32, b: f32| -> (f32, f32) {
                proj([fc[0] + uv[0] * a + vv[0] * b, fc[1] + uv[1] * a + vv[1] * b, fc[2] + uv[2] * a + vv[2] * b])
            };
            let lit = 0.78 + 0.22 * dot3(nv, light).max(0.0);
            let body = [pt(-0.5, -0.5), pt(0.5, -0.5), pt(0.5, 0.5), pt(-0.5, 0.5)];
            cv.fill_poly(&body, pal.body.scale(0.9 + 0.1 * lit));
            if dot(p, n) == 1 {
                let col = pal.sticker[cube.f[facelet_index(p, n)] as usize].scale(lit);
                let (h, ch) = (0.43f32, 0.13f32);
                let oct = [
                    pt(-(h - ch), -h),
                    pt(h - ch, -h),
                    pt(h, -(h - ch)),
                    pt(h, h - ch),
                    pt(h - ch, h),
                    pt(-(h - ch), h),
                    pt(-h, h - ch),
                    pt(-h, -(h - ch)),
                ];
                cv.fill_poly(&oct, col);
            }
        }
    }
}
