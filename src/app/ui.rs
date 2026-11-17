//! Immediate-mode widgets. There is no widget tree: every frame the screen code calls
//! `button(...)`, `segmented(...)` etc. with a rectangle; the call draws the widget and
//! returns whether it was tapped this frame. Taps are consumed by the first widget that
//! contains them (and only if the tap is inside the current clip rectangle).

use super::App;
use crate::cube::Cube;
use crate::gfx::cube3d::draw_cube;
use crate::gfx::{Canvas, Color, Face};

// Material Icons codepoints (checked against the bundled MaterialIcons-Regular.ttf).
pub const IC_BACK: char = '\u{e5c4}'; // arrow_back
pub const IC_CHEVRON: char = '\u{e5cc}'; // chevron_right
pub const IC_PLAY: char = '\u{e037}'; // play_arrow
pub const IC_PAUSE: char = '\u{e034}'; // pause
pub const IC_NEXT: char = '\u{e044}'; // skip_next
pub const IC_PREV: char = '\u{e045}'; // skip_previous
pub const IC_REPLAY: char = '\u{e042}'; // replay
pub const IC_SHUFFLE: char = '\u{e043}'; // shuffle
pub const IC_UNDO: char = '\u{e166}'; // undo
pub const IC_SCHOOL: char = '\u{e80c}'; // school
pub const IC_CUBE: char = '\u{e9fe}'; // view_in_ar
pub const IC_SETTINGS: char = '\u{e8b8}'; // settings

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
    pub fn cx(&self) -> f32 {
        self.x + self.w * 0.5
    }
    pub fn cy(&self) -> f32 {
        self.y + self.h * 0.5
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BtnStyle {
    Primary,
    Outline,
    Ghost,
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl App {
    pub(crate) fn dp(&self, v: f32) -> f32 {
        v * self.scale
    }

    pub(crate) fn hair(&self) -> f32 {
        (self.scale * 0.8).max(1.0)
    }

    pub(crate) fn tabbar_h(&self) -> f32 {
        self.dp(62.0) + self.insets.bottom
    }

    // ---------------------------------------------------------------- input
    pub(crate) fn tapped(&mut self, cv: &Canvas, r: Rect) -> bool {
        if self.tap_used {
            return false;
        }
        if let Some((x, y)) = self.pointer.tap {
            let (c0, c1, c2, c3) = cv.clip();
            let in_clip = x >= c0 as f32 && x < c2 as f32 && y >= c1 as f32 && y < c3 as f32;
            if in_clip && r.contains(x, y) {
                self.tap_used = true;
                return true;
            }
        }
        false
    }

    pub(crate) fn pressed(&self, r: Rect) -> bool {
        let p = &self.pointer;
        p.down && p.moved <= self.scale * 10.0 && r.contains(p.px, p.py) && r.contains(p.x, p.y)
    }

    // ---------------------------------------------------------------- text
    pub(crate) fn text(&mut self, cv: &mut Canvas, face: Face, size: f32, x: f32, y: f32, s: &str, col: Color) -> f32 {
        let px = self.dp(size);
        self.fonts.draw(cv, face, px, x, y, s, col)
    }

    pub(crate) fn text_w(&mut self, face: Face, size: f32, s: &str) -> f32 {
        let px = self.dp(size);
        self.fonts.width(face, px, s)
    }

    pub(crate) fn text_center(&mut self, cv: &mut Canvas, face: Face, size: f32, cx: f32, y: f32, s: &str, col: Color) {
        let w = self.text_w(face, size, s);
        self.text(cv, face, size, cx - w * 0.5, y, s, col);
    }

    /// Text centred horizontally and vertically inside `r`.
    pub(crate) fn text_mid(&mut self, cv: &mut Canvas, face: Face, size: f32, r: Rect, s: &str, col: Color) {
        let px = self.dp(size);
        let lh = self.fonts.line_height(face, px);
        let w = self.fonts.width(face, px, s);
        self.fonts.draw(cv, face, px, r.x + (r.w - w) * 0.5, r.y + (r.h - lh) * 0.5, s, col);
    }

    /// Wrapped paragraph; returns its height. Lines outside the clip are skipped.
    pub(crate) fn paragraph(&mut self, cv: &mut Canvas, face: Face, size: f32, x: f32, y: f32, max_w: f32, s: &str, col: Color) -> f32 {
        let px = self.dp(size);
        let lines = self.fonts.wrap(face, px, s, max_w);
        let step = px * 1.55;
        let (_, cy0, _, cy1) = cv.clip();
        for (i, l) in lines.iter().enumerate() {
            let ly = y + i as f32 * step;
            if ly + step < cy0 as f32 || ly > cy1 as f32 {
                continue;
            }
            self.fonts.draw(cv, face, px, x, ly, l, col);
        }
        lines.len() as f32 * step
    }

    /// Small section label (muted, uppercase strings come from i18n).
    pub(crate) fn label(&mut self, cv: &mut Canvas, x: f32, y: f32, s: &str) {
        let col = self.pal.muted;
        self.text(cv, Face::Body, 11.0, x, y, s, col);
    }

    pub(crate) fn icon(&mut self, cv: &mut Canvas, size: f32, cx: f32, cy: f32, ch: char, col: Color) {
        let px = self.dp(size);
        self.fonts.icon(cv, px, cx, cy, ch, col);
    }

    // ---------------------------------------------------------------- widgets
    pub(crate) fn card(&mut self, cv: &mut Canvas, r: Rect, rad_dp: f32) {
        let pal = self.pal;
        let rad = self.dp(rad_dp);
        cv.fill_rrect(r.x, r.y, r.w, r.h, rad, pal.card);
        let t = self.hair();
        cv.stroke_rrect(r.x, r.y, r.w, r.h, rad, t, pal.line);
    }

    pub(crate) fn button(&mut self, cv: &mut Canvas, r: Rect, label: &str, face: Face, size: f32, style: BtnStyle, rad: f32) -> bool {
        let pal = self.pal;
        let down = self.pressed(r);
        match style {
            BtnStyle::Primary => {
                let bg = if down { pal.primary.scale(0.88) } else { pal.primary };
                cv.fill_rrect(r.x, r.y, r.w, r.h, rad, bg);
                self.text_mid(cv, face, size, r, label, pal.on_primary);
            }
            _ => {
                let bg = if down { pal.line } else { pal.card };
                cv.fill_rrect(r.x, r.y, r.w, r.h, rad, bg);
                let t = self.hair();
                cv.stroke_rrect(r.x, r.y, r.w, r.h, rad, t, pal.line);
                self.text_mid(cv, face, size, r, label, pal.text);
            }
        }
        self.tapped(cv, r)
    }

    pub(crate) fn chip(&mut self, cv: &mut Canvas, r: Rect, label: &str, selected: bool) -> bool {
        let style = if selected { BtnStyle::Primary } else { BtnStyle::Outline };
        self.button(cv, r, label, Face::Body, 13.0, style, r.h * 0.5)
    }

    pub(crate) fn icon_button(&mut self, cv: &mut Canvas, r: Rect, ch: char, style: BtnStyle, size: f32) -> bool {
        let pal = self.pal;
        let down = self.pressed(r);
        let rad = r.w.min(r.h) * 0.5;
        let fg = match style {
            BtnStyle::Primary => {
                let bg = if down { pal.primary.scale(0.88) } else { pal.primary };
                cv.fill_rrect(r.x, r.y, r.w, r.h, rad, bg);
                pal.on_primary
            }
            BtnStyle::Outline => {
                let bg = if down { pal.line } else { pal.card };
                cv.fill_rrect(r.x, r.y, r.w, r.h, rad, bg);
                let t = self.hair();
                cv.stroke_rrect(r.x, r.y, r.w, r.h, rad, t, pal.line);
                pal.text
            }
            BtnStyle::Ghost => {
                if down {
                    cv.fill_rrect(r.x, r.y, r.w, r.h, rad, pal.line);
                }
                pal.text
            }
        };
        self.icon(cv, size, r.cx(), r.cy(), ch, fg);
        self.tapped(cv, r)
    }

    /// Outline button with a leading icon.
    pub(crate) fn action_button(&mut self, cv: &mut Canvas, r: Rect, ic: char, label: &str) -> bool {
        let pal = self.pal;
        let down = self.pressed(r);
        let bg = if down { pal.line } else { pal.card };
        cv.fill_rrect(r.x, r.y, r.w, r.h, r.h * 0.5, bg);
        let t = self.hair();
        cv.stroke_rrect(r.x, r.y, r.w, r.h, r.h * 0.5, t, pal.line);
        let tw = self.text_w(Face::Body, 13.0, label);
        let iw = self.dp(18.0);
        let gap = self.dp(6.0);
        let x0 = r.x + (r.w - (iw + gap + tw)) * 0.5;
        self.icon(cv, 18.0, x0 + iw * 0.5, r.cy(), ic, pal.accent);
        let px = self.dp(13.0);
        let lh = self.fonts.line_height(Face::Body, px);
        self.text(cv, Face::Body, 13.0, x0 + iw + gap, r.y + (r.h - lh) * 0.5, label, pal.text);
        self.tapped(cv, r)
    }

    /// Segmented control; returns the index tapped this frame.
    pub(crate) fn segmented(&mut self, cv: &mut Canvas, r: Rect, labels: &[&str], sel: usize) -> Option<usize> {
        let pal = self.pal;
        let rad = r.h * 0.5;
        cv.fill_rrect(r.x, r.y, r.w, r.h, rad, pal.card);
        let t = self.hair();
        cv.stroke_rrect(r.x, r.y, r.w, r.h, rad, t, pal.line);
        let pad = self.dp(4.0);
        let n = labels.len().max(1);
        let sw = (r.w - 2.0 * pad) / n as f32;
        let mut hit = None;
        for (i, lab) in labels.iter().enumerate() {
            let sr = Rect::new(r.x + pad + sw * i as f32, r.y + pad, sw, r.h - 2.0 * pad);
            let on = i == sel;
            if on {
                cv.fill_rrect(sr.x, sr.y, sr.w, sr.h, sr.h * 0.5, pal.primary);
            }
            let col = if on { pal.on_primary } else { pal.text };
            self.text_mid(cv, Face::Body, 13.0, sr, lab, col);
            if self.tapped(cv, sr) {
                hit = Some(i);
            }
        }
        hit
    }

    pub(crate) fn render_cube(&self, cv: &mut Canvas, r: Rect, cube: &Cube, animated: bool) {
        let a = if animated { self.anim.as_ref().map(|a| (a.mv, ease(a.t))) } else { None };
        draw_cube(cv, &self.pal, cube, a, self.view, (r.x, r.y, r.w, r.h));
    }
}
