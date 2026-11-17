//! Text with fontdue. Fonts are embedded, parsed lazily (only the ones a screen really uses)
//! and glyph bitmaps are cached, so steady-state text drawing is just alpha blending.
//!
//! NOTE: the bundled Poppins has no Vietnamese-specific glyphs (88 of them are missing:
//! the horned o/u and every stacked tone mark), so Vietnamese body text uses Playfair Display,
//! which covers them all. To get a sans-serif Vietnamese UI, point `SERIF_BODY` below at a
//! Vietnamese-complete sans such as Be Vietnam Pro or Noto Sans.

use super::canvas::{Canvas, Color};
use crate::i18n::Lang;
use fontdue::{Font, FontSettings};
use std::collections::HashMap;

const SANS: &[u8] = include_bytes!("../../assets/fonts/Poppins-Regular.ttf");
const SERIF: &[u8] = include_bytes!("../../assets/fonts/PlayfairDisplay-Regular.ttf");
const MONO: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
const ICONS: &[u8] = include_bytes!("../../assets/fonts/MaterialIcons-Regular.ttf");
/// Font used for Vietnamese body text (must cover all Vietnamese letters).
const SERIF_BODY: &[u8] = SERIF;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Face {
    /// Reading text: Poppins (English) / Vietnamese-capable fallback.
    Body,
    /// Titles: Playfair Display.
    Heading,
    /// Algorithms: JetBrains Mono.
    Mono,
    /// Material Icons codepoints.
    Icon,
}

const S_SANS: usize = 0;
const S_SERIF: usize = 1;
const S_MONO: usize = 2;
const S_ICONS: usize = 3;

struct Slot {
    data: &'static [u8],
    font: Option<Font>,
}

impl Slot {
    fn new(data: &'static [u8]) -> Slot {
        Slot { data, font: None }
    }
    fn font(&mut self) -> &Font {
        if self.font.is_none() {
            self.font = Some(Font::from_bytes(self.data, FontSettings::default()).expect("bad font file"));
        }
        self.font.as_ref().unwrap()
    }
}

struct Glyph {
    w: usize,
    h: usize,
    xmin: i32,
    ymin: i32,
    adv: f32,
    bmp: Vec<u8>,
}

pub struct Fonts {
    slots: [Slot; 4],
    cache: HashMap<(u8, char, u16), Glyph>,
    pub lang: Lang,
}

impl Default for Fonts {
    fn default() -> Self {
        Fonts::new()
    }
}

impl Fonts {
    pub fn new() -> Fonts {
        Fonts {
            slots: [Slot::new(SANS), Slot::new(SERIF_BODY), Slot::new(MONO), Slot::new(ICONS)],
            cache: HashMap::new(),
            lang: Lang::En,
        }
    }

    /// Drop cached bitmaps (call on low-memory events).
    pub fn trim(&mut self) {
        self.cache.clear();
        self.cache.shrink_to_fit();
    }

    fn covers(&mut self, slot: usize, text: &str) -> bool {
        let f = self.slots[slot].font();
        text.chars().all(|c| c.is_ascii() || c.is_control() || f.lookup_glyph_index(c) != 0)
    }

    fn resolve(&mut self, face: Face, text: &str) -> usize {
        let chain: [usize; 2] = match (face, self.lang) {
            (Face::Icon, _) => return S_ICONS,
            (Face::Mono, _) => return S_MONO,
            (Face::Heading, _) => return S_SERIF,
            (Face::Body, Lang::En) => [S_SANS, S_SERIF],
            (Face::Body, Lang::Vi) => [S_SERIF, S_SANS],
        };
        if chain[0] == S_SANS && text.is_ascii() {
            return S_SANS; // Poppins has all of ASCII: skip the lookup
        }
        for &s in &chain {
            if self.covers(s, text) {
                return s;
            }
        }
        chain[0]
    }

    fn glyph(&mut self, slot: usize, ch: char, px: f32) -> &Glyph {
        let q = (px * 4.0).round();
        let key = (slot as u8, ch, q as u16);
        if self.cache.len() > 2500 {
            self.cache.clear();
        }
        if !self.cache.contains_key(&key) {
            let f = self.slots[slot].font();
            let (m, bmp) = f.rasterize(ch, q / 4.0);
            self.cache.insert(
                key,
                Glyph { w: m.width, h: m.height, xmin: m.xmin, ymin: m.ymin, adv: m.advance_width, bmp },
            );
        }
        &self.cache[&key]
    }

    fn line_metrics(&mut self, slot: usize, px: f32) -> (f32, f32) {
        match self.slots[slot].font().horizontal_line_metrics(px) {
            Some(m) => (m.ascent, m.new_line_size),
            None => (px * 0.95, px * 1.3),
        }
    }

    /// Line box height for `face` at `px`.
    pub fn line_height(&mut self, face: Face, px: f32) -> f32 {
        let s = self.resolve(face, "A");
        self.line_metrics(s, px).1
    }

    pub fn width(&mut self, face: Face, px: f32, text: &str) -> f32 {
        let s = self.resolve(face, text);
        let mut w = 0.0;
        for ch in text.chars() {
            w += self.glyph(s, ch, px).adv;
        }
        w
    }

    /// Draw `text` with the top of its line box at `y`. Returns the advance width.
    pub fn draw(&mut self, cv: &mut Canvas, face: Face, px: f32, x: f32, y: f32, text: &str, color: Color) -> f32 {
        let s = self.resolve(face, text);
        let (asc, _) = self.line_metrics(s, px);
        let base = (y + asc).round() as i32;
        let mut pen = x;
        for ch in text.chars() {
            let g = self.glyph(s, ch, px);
            let gx = (pen + g.xmin as f32).round() as i32;
            let gy = base - g.h as i32 - g.ymin;
            cv.blit_mask(gx, gy, g.w, g.h, &g.bmp, color);
            pen += g.adv;
        }
        pen - x
    }

    /// Draw a Material Icons glyph centred on (cx, cy).
    pub fn icon(&mut self, cv: &mut Canvas, px: f32, cx: f32, cy: f32, cp: char, color: Color) {
        let g = self.glyph(S_ICONS, cp, px);
        let gx = (cx - g.w as f32 * 0.5).round() as i32;
        let gy = (cy - g.h as f32 * 0.5).round() as i32;
        cv.blit_mask(gx, gy, g.w, g.h, &g.bmp, color);
    }

    /// Greedy word wrap. '\n' forces a break.
    pub fn wrap(&mut self, face: Face, px: f32, text: &str, max_w: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for para in text.split('\n') {
            let mut cur = String::new();
            for word in para.split_whitespace() {
                let cand = if cur.is_empty() { word.to_string() } else { format!("{} {}", cur, word) };
                if cur.is_empty() || self.width(face, px, &cand) <= max_w {
                    cur = cand;
                } else {
                    lines.push(cur);
                    cur = word.to_string();
                }
            }
            lines.push(cur);
        }
        lines
    }

    /// Truncate with an ellipsis so the text fits `max_w`.
    pub fn fit(&mut self, face: Face, px: f32, text: &str, max_w: f32) -> String {
        if self.width(face, px, text) <= max_w {
            return text.to_string();
        }
        let mut s = String::new();
        for ch in text.chars() {
            let mut t = s.clone();
            t.push(ch);
            t.push('\u{2026}');
            if self.width(face, px, &t) > max_w {
                break;
            }
            s.push(ch);
        }
        s.push('\u{2026}');
        s
    }
}

impl Fonts {
    /// Characters of `text` that the font chosen for (`face`, `text`) cannot draw (tofu check).
    pub fn missing(&mut self, face: Face, text: &str) -> Vec<char> {
        let s = self.resolve(face, text);
        let f = self.slots[s].font();
        text.chars().filter(|&c| !c.is_control() && c != ' ' && f.lookup_glyph_index(c) == 0).collect()
    }
}
