//! Tiny software canvas drawing straight into a 32-bit RGBA surface
//! (byte order R,G,B,A in memory = ANDROID WINDOW_FORMAT_RGBA_8888).
//! Anti-aliased rounded rects, ellipses, convex polygons and alpha masks (for glyphs).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn hex(v: u32) -> Color {
        Color { r: (v >> 16) as u8, g: (v >> 8) as u8, b: v as u8, a: 255 }
    }

    pub const fn alpha(self, a: u8) -> Color {
        Color { r: self.r, g: self.g, b: self.b, a }
    }

    pub fn scale(self, k: f32) -> Color {
        let f = |c: u8| (c as f32 * k).clamp(0.0, 255.0) as u8;
        Color { r: f(self.r), g: f(self.g), b: f(self.b), a: self.a }
    }

    pub fn mix(self, o: Color, t: f32) -> Color {
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8;
        Color { r: f(self.r, o.r), g: f(self.g, o.g), b: f(self.b, o.b), a: self.a }
    }

    #[inline(always)]
    fn pack(self) -> u32 {
        0xFF00_0000 | ((self.b as u32) << 16) | ((self.g as u32) << 8) | self.r as u32
    }
}

#[inline(always)]
fn blend(dst: u32, c: Color, a: u32) -> u32 {
    if a >= 255 {
        return c.pack();
    }
    let ia = 255 - a;
    let r = (c.r as u32 * a + (dst & 0xFF) * ia + 127) / 255;
    let g = (c.g as u32 * a + ((dst >> 8) & 0xFF) * ia + 127) / 255;
    let b = (c.b as u32 * a + ((dst >> 16) & 0xFF) * ia + 127) / 255;
    0xFF00_0000 | (b << 16) | (g << 8) | r
}

pub struct Canvas<'a> {
    buf: &'a mut [u32],
    pub w: i32,
    pub h: i32,
    stride: usize,
    clip: (i32, i32, i32, i32), // x0, y0, x1, y1 (exclusive)
}

impl<'a> Canvas<'a> {
    /// `stride` is the row length of `buf` in pixels (>= w).
    pub fn new(buf: &'a mut [u32], w: i32, h: i32, stride: usize) -> Canvas<'a> {
        Canvas { buf, w, h, stride, clip: (0, 0, w, h) }
    }

    pub fn clip(&self) -> (i32, i32, i32, i32) {
        self.clip
    }

    pub fn set_clip(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.clip = (x0.max(0), y0.max(0), x1.min(self.w), y1.min(self.h));
    }

    pub fn reset_clip(&mut self) {
        self.clip = (0, 0, self.w, self.h);
    }

    pub fn clear(&mut self, c: Color) {
        let v = c.pack();
        for y in 0..self.h as usize {
            let row = y * self.stride;
            self.buf[row..row + self.w as usize].fill(v);
        }
    }

    #[inline(always)]
    fn blend_at(&mut self, x: i32, y: i32, c: Color, cov: u32) {
        let a = (c.a as u32 * cov + 127) / 255;
        if a == 0 {
            return;
        }
        let i = y as usize * self.stride + x as usize;
        self.buf[i] = blend(self.buf[i], c, a);
    }

    /// Axis-aligned rectangle, edges snapped to whole pixels.
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let (cx0, cy0, cx1, cy1) = self.clip;
        let x0 = (x.round() as i32).max(cx0);
        let y0 = (y.round() as i32).max(cy0);
        let x1 = ((x + w).round() as i32).min(cx1);
        let y1 = ((y + h).round() as i32).min(cy1);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for yy in y0..y1 {
            let row = yy as usize * self.stride;
            if c.a == 255 {
                self.buf[row + x0 as usize..row + x1 as usize].fill(c.pack());
            } else {
                for xx in x0..x1 {
                    self.blend_at(xx, yy, c, 255);
                }
            }
        }
    }

    pub fn fill_rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
        self.rrect(x, y, w, h, r, None, c);
    }

    pub fn stroke_rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, t: f32, c: Color) {
        self.rrect(x, y, w, h, r, Some(t), c);
    }

    fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, stroke: Option<f32>, c: Color) {
        let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
        let (hx, hy) = (w * 0.5, h * 0.5);
        let (cx, cy) = (x + hx, y + hy);
        let (cx0, cy0, cx1, cy1) = self.clip;
        let x0 = (x.floor() as i32 - 1).max(cx0);
        let y0 = (y.floor() as i32 - 1).max(cy0);
        let x1 = ((x + w).ceil() as i32 + 1).min(cx1);
        let y1 = ((y + h).ceil() as i32 + 1).min(cy1);
        for py in y0..y1 {
            let dy = (py as f32 + 0.5 - cy).abs() - (hy - r);
            for px in x0..x1 {
                let dx = (px as f32 + 0.5 - cx).abs() - (hx - r);
                let ox = dx.max(0.0);
                let oy = dy.max(0.0);
                let d = (ox * ox + oy * oy).sqrt() + dx.max(dy).min(0.0) - r;
                let mut cov = (0.5 - d).clamp(0.0, 1.0);
                if let Some(t) = stroke {
                    cov *= (d + t + 0.5).clamp(0.0, 1.0);
                }
                if cov > 0.0 {
                    self.blend_at(px, py, c, (cov * 255.0) as u32);
                }
            }
        }
    }

    /// Approximate anti-aliased ellipse (good enough for soft shadows).
    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Color) {
        let (cx0, cy0, cx1, cy1) = self.clip;
        let x0 = ((cx - rx).floor() as i32 - 1).max(cx0);
        let y0 = ((cy - ry).floor() as i32 - 1).max(cy0);
        let x1 = ((cx + rx).ceil() as i32 + 1).min(cx1);
        let y1 = ((cy + ry).ceil() as i32 + 1).min(cy1);
        let m = rx.min(ry);
        for py in y0..y1 {
            let ny = (py as f32 + 0.5 - cy) / ry;
            for px in x0..x1 {
                let nx = (px as f32 + 0.5 - cx) / rx;
                let d = ((nx * nx + ny * ny).sqrt() - 1.0) * m;
                let cov = (0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_at(px, py, c, (cov * 255.0) as u32);
                }
            }
        }
    }

    /// Convex polygon (any winding). Coverage = clamp(0.5 + distance to nearest edge).
    pub fn fill_poly(&mut self, pts: &[(f32, f32)], c: Color) {
        let n = pts.len().min(12);
        if n < 3 {
            return;
        }
        let mut area = 0.0f32;
        let (mut minx, mut miny, mut maxx, mut maxy) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for i in 0..n {
            let (ax, ay) = pts[i];
            let (bx, by) = pts[(i + 1) % n];
            area += ax * by - bx * ay;
            minx = minx.min(ax);
            miny = miny.min(ay);
            maxx = maxx.max(ax);
            maxy = maxy.max(ay);
        }
        if area.abs() < 1e-3 {
            return;
        }
        let sgn = if area > 0.0 { 1.0 } else { -1.0 };
        let mut e = [[0f32; 3]; 12];
        let mut ne = 0;
        for i in 0..n {
            let (ax, ay) = pts[i];
            let (bx, by) = pts[(i + 1) % n];
            let (dx, dy) = (bx - ax, by - ay);
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-4 {
                continue;
            }
            let s = sgn / len;
            e[ne] = [-dy * s, dx * s, (dy * ax - dx * ay) * s];
            ne += 1;
        }
        let (cx0, cy0, cx1, cy1) = self.clip;
        let x0 = (minx.floor() as i32 - 1).max(cx0);
        let y0 = (miny.floor() as i32 - 1).max(cy0);
        let x1 = (maxx.ceil() as i32 + 1).min(cx1);
        let y1 = (maxy.ceil() as i32 + 1).min(cy1);
        for y in y0..y1 {
            let py = y as f32 + 0.5;
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let mut d = f32::MAX;
                for k in 0..ne {
                    let v = e[k][0] * px + e[k][1] * py + e[k][2];
                    if v < d {
                        d = v;
                        if d < -0.5 {
                            break;
                        }
                    }
                }
                if d < -0.5 {
                    continue;
                }
                let cov = ((d + 0.5).min(1.0) * 255.0) as u32;
                self.blend_at(x, y, c, cov);
            }
        }
    }

    /// Blend an 8-bit coverage mask (a rasterised glyph) with its top-left at (x, y).
    pub fn blit_mask(&mut self, x: i32, y: i32, w: usize, h: usize, mask: &[u8], c: Color) {
        let (cx0, cy0, cx1, cy1) = self.clip;
        for j in 0..h as i32 {
            let yy = y + j;
            if yy < cy0 || yy >= cy1 {
                continue;
            }
            for i in 0..w as i32 {
                let xx = x + i;
                if xx < cx0 || xx >= cx1 {
                    continue;
                }
                let m = mask[j as usize * w + i as usize] as u32;
                if m != 0 {
                    self.blend_at(xx, yy, c, m);
                }
            }
        }
    }
}
