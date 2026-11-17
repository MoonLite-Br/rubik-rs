//! The four screens (lesson list, lesson, practice, settings) and the tab bar.

use super::settings::LangPref;
use super::ui::*;
use super::{App, Tab};
use crate::cube::{progress, Cube, Move};
use crate::gfx::{Canvas, Face};
use crate::i18n::s;
use crate::theme::ThemePref;
use crate::tutorial::{Case, Lesson, LESSONS};

enum Ctl {
    Restart,
    Prev,
    Play,
    Next,
}

impl App {
    /// Draw one frame.
    pub fn draw(&mut self, cv: &mut Canvas) {
        self.dirty = false; // state changes made while drawing (taps) set it again
        self.tap_used = false;
        self.cube_rect = None;
        self.scroll_rect = None;
        cv.reset_clip();
        let bg = self.pal.bg;
        cv.clear(bg);
        match self.tab {
            Tab::Learn => {
                if self.player.is_some() {
                    self.draw_lesson(cv);
                } else {
                    self.draw_learn(cv);
                }
            }
            Tab::Practice => self.draw_practice(cv),
            Tab::Settings => self.draw_settings(cv),
        }
        self.draw_tabbar(cv);
        self.pointer.tap = None;
    }

    // ------------------------------------------------------------------ tab bar
    fn draw_tabbar(&mut self, cv: &mut Canvas) {
        cv.reset_clip();
        let pal = self.pal;
        let lang = self.lang();
        let w = self.w as f32;
        let h = self.tabbar_h();
        let y0 = self.h as f32 - h;
        let line = self.hair();
        cv.fill_rect(0.0, y0, w, h, pal.card);
        cv.fill_rect(0.0, y0, w, line, pal.line);
        let tabs = [
            (Tab::Learn, IC_SCHOOL, s::TAB_LEARN),
            (Tab::Practice, IC_CUBE, s::TAB_PRACTICE),
            (Tab::Settings, IC_SETTINGS, s::TAB_SETTINGS),
        ];
        let th = self.dp(62.0);
        let mut go = None;
        for (i, (tab, ic, label)) in tabs.iter().enumerate() {
            let r = Rect::new(w / 3.0 * i as f32, y0, w / 3.0, th);
            let on = self.tab == *tab;
            let col = if on { pal.text } else { pal.muted };
            if on {
                cv.fill_rrect(r.cx() - self.dp(11.0), y0 + self.dp(1.0), self.dp(22.0), self.dp(2.5), self.dp(1.25), pal.accent);
            }
            self.icon(cv, 24.0, r.cx(), y0 + self.dp(23.0), *ic, col);
            self.text_center(cv, Face::Body, 11.0, r.cx(), y0 + self.dp(37.0), label.get(lang), col);
            if self.tapped(cv, r) {
                go = Some(*tab);
            }
        }
        if let Some(t) = go {
            self.switch_tab(t);
        }
    }

    // ------------------------------------------------------------------ learn list
    fn draw_learn(&mut self, cv: &mut Canvas) {
        let pal = self.pal;
        let lang = self.lang();
        let w = self.w as f32;
        let m = self.dp(22.0);
        let top = self.insets.top;
        let bottom = self.h as f32 - self.tabbar_h();
        let area = Rect::new(0.0, top, w, bottom - top);
        self.scroll_rect = Some(area);
        cv.set_clip(0, top as i32, self.w, bottom as i32);

        let mut y = top + self.dp(22.0) - self.scroll[0];
        let y_start = y;
        self.text(cv, Face::Heading, 40.0, m, y, "Rubik", pal.text);
        y += self.dp(58.0);
        self.text(cv, Face::Body, 14.0, m, y, s::TAGLINE.get(lang), pal.muted);
        y += self.dp(30.0);

        let hero = Rect::new(0.0, y, w, self.dp(230.0));
        self.cube_rect = Some(hero);
        self.render_cube(cv, hero, &Cube::solved(), false);
        y += hero.h + self.dp(8.0);

        self.label(cv, m, y, s::LESSONS.get(lang));
        y += self.dp(26.0);

        let mut open = None;
        let ch = self.dp(74.0);
        for (i, lesson) in LESSONS.iter().enumerate() {
            let r = Rect::new(m, y, w - 2.0 * m, ch);
            if self.lesson_card(cv, r, i, lesson) {
                open = Some(i);
            }
            y += ch + self.dp(10.0);
        }
        y += self.dp(16.0);

        self.scroll_max = (y - y_start - area.h).max(0.0);
        if self.scroll[0] > self.scroll_max {
            self.scroll[0] = self.scroll_max;
        }
        cv.reset_clip();
        if let Some(i) = open {
            self.open_lesson(i);
        }
    }

    fn lesson_card(&mut self, cv: &mut Canvas, r: Rect, i: usize, lesson: &Lesson) -> bool {
        let pal = self.pal;
        let lang = self.lang();
        let down = self.pressed(r);
        let rad = self.dp(18.0);
        cv.fill_rrect(r.x, r.y, r.w, r.h, rad, if down { pal.line } else { pal.card });
        let t = self.hair();
        cv.stroke_rrect(r.x, r.y, r.w, r.h, rad, t, pal.line);

        let d = self.dp(40.0);
        let c = Rect::new(r.x + self.dp(16.0), r.cy() - d * 0.5, d, d);
        cv.stroke_rrect(c.x, c.y, c.w, c.h, d * 0.5, self.dp(1.2), pal.accent);
        let num = format!("{}", i);
        self.text_mid(cv, Face::Heading, 17.0, c, &num, pal.accent);

        let tx = c.x + d + self.dp(14.0);
        let max_w = r.x + r.w - tx - self.dp(40.0);
        let px_t = self.dp(16.0);
        let title = self.fonts.fit(Face::Body, px_t, lesson.title.get(lang), max_w);
        self.text(cv, Face::Body, 16.0, tx, r.y + self.dp(14.0), &title, pal.text);
        let px_s = self.dp(12.0);
        let sub = self.fonts.fit(Face::Body, px_s, lesson.sub.get(lang), max_w);
        self.text(cv, Face::Body, 12.0, tx, r.y + self.dp(40.0), &sub, pal.muted);
        self.icon(cv, 22.0, r.x + r.w - self.dp(24.0), r.cy(), IC_CHEVRON, pal.muted);
        self.tapped(cv, r)
    }

    // ------------------------------------------------------------------ lesson
    fn draw_lesson(&mut self, cv: &mut Canvas) {
        let pal = self.pal;
        let lang = self.lang();
        let w = self.w as f32;
        let m = self.dp(22.0);
        let (li, ci, step, tokens) = {
            let p = self.player.as_ref().unwrap();
            (p.lesson, p.case, p.step, p.moves.iter().map(|mv| mv.name()).collect::<Vec<String>>())
        };
        let lesson: &'static Lesson = &LESSONS[li];
        let case: &'static Case = &lesson.cases[ci];
        let playing = self.playing;

        // header
        let top = self.insets.top + self.dp(6.0);
        let back = Rect::new(self.dp(10.0), top, self.dp(44.0), self.dp(44.0));
        let mut close = false;
        if self.icon_button(cv, back, IC_BACK, BtnStyle::Ghost, 24.0) {
            close = true;
        }
        let ty = top + self.dp(9.0);
        self.text(cv, Face::Heading, 21.0, self.dp(60.0), ty, lesson.title.get(lang), pal.text);

        // cube
        let cube_top = top + self.dp(52.0);
        let cube_h = (self.h as f32 * 0.30).clamp(self.dp(200.0), self.dp(330.0));
        let cube_rect = Rect::new(0.0, cube_top, w, cube_h);
        self.cube_rect = Some(cube_rect);
        let cube = self.cube_lesson;
        self.render_cube(cv, cube_rect, &cube, true);

        // controls
        let cy = cube_top + cube_h + self.dp(2.0);
        let (s1, s2, s3, g) = (self.dp(44.0), self.dp(48.0), self.dp(58.0), self.dp(10.0));
        let total = s1 + s2 + s3 + s2 + 3.0 * g;
        let mut x = (w - total) * 0.5;
        let mut ctl: Option<Ctl> = None;
        let cyc = cy + s3 * 0.5;
        if self.icon_button(cv, Rect::new(x, cyc - s1 * 0.5, s1, s1), IC_REPLAY, BtnStyle::Outline, 22.0) {
            ctl = Some(Ctl::Restart);
        }
        x += s1 + g;
        if self.icon_button(cv, Rect::new(x, cyc - s2 * 0.5, s2, s2), IC_PREV, BtnStyle::Outline, 24.0) {
            ctl = Some(Ctl::Prev);
        }
        x += s2 + g;
        let pi = if playing { IC_PAUSE } else { IC_PLAY };
        if self.icon_button(cv, Rect::new(x, cy, s3, s3), pi, BtnStyle::Primary, 30.0) {
            ctl = Some(Ctl::Play);
        }
        x += s3 + g;
        if self.icon_button(cv, Rect::new(x, cyc - s2 * 0.5, s2, s2), IC_NEXT, BtnStyle::Outline, 24.0) {
            ctl = Some(Ctl::Next);
        }
        let counter = format!("{}/{}", step, tokens.len());
        let cw = self.text_w(Face::Mono, 13.0, &counter);
        self.text(cv, Face::Mono, 13.0, w - m - cw, cyc - self.dp(9.0), &counter, pal.muted);

        // scrolling text area
        let sy = cy + s3 + self.dp(8.0);
        let sbot = self.h as f32 - self.tabbar_h();
        let area = Rect::new(0.0, sy, w, (sbot - sy).max(0.0));
        self.scroll_rect = Some(area);
        cv.set_clip(0, sy as i32, self.w, sbot as i32);
        let tw = w - 2.0 * m;
        let mut y = sy - self.scroll[1] + self.dp(8.0);
        let y_start = y;

        y += self.paragraph(cv, Face::Body, 14.0, m, y, tw, lesson.intro.get(lang), pal.text) + self.dp(8.0);
        y += self.paragraph(cv, Face::Body, 12.0, m, y, tw, s::ORIENTATION.get(lang), pal.muted) + self.dp(18.0);

        self.label(cv, m, y, s::SITUATIONS.get(lang));
        y += self.dp(24.0);
        let chip_h = self.dp(36.0);
        let mut x = m;
        let mut pick = None;
        for (i, c) in lesson.cases.iter().enumerate() {
            let name = c.name.get(lang);
            let wd = self.text_w(Face::Body, 13.0, name) + self.dp(30.0);
            if x + wd > w - m && x > m {
                x = m;
                y += chip_h + self.dp(8.0);
            }
            if self.chip(cv, Rect::new(x, y, wd, chip_h), name, i == ci) {
                pick = Some(i);
            }
            x += wd + self.dp(8.0);
        }
        y += chip_h + self.dp(14.0);

        y += self.paragraph(cv, Face::Body, 14.0, m, y, tw, case.hint.get(lang), pal.text) + self.dp(14.0);

        // algorithm card with one pill per move
        self.label(cv, m, y, s::ALGORITHM.get(lang));
        y += self.dp(24.0);
        let pad = self.dp(14.0);
        let tok_h = self.dp(38.0);
        let gap = self.dp(6.0);
        let avail = tw - 2.0 * pad;
        let widths: Vec<f32> = tokens.iter().map(|t| self.text_w(Face::Mono, 17.0, t) + self.dp(20.0)).collect();
        let mut rows = 1usize;
        let mut rx = 0.0f32;
        for wd in &widths {
            if rx + wd > avail && rx > 0.0 {
                rows += 1;
                rx = 0.0;
            }
            rx += wd + gap;
        }
        let card_h = 2.0 * pad + rows as f32 * tok_h + (rows - 1) as f32 * gap;
        self.card(cv, Rect::new(m, y, tw, card_h), 18.0);
        let mut tx = m + pad;
        let mut ty = y + pad;
        for (i, t) in tokens.iter().enumerate() {
            let wd = widths[i];
            if tx + wd > m + pad + avail && tx > m + pad {
                tx = m + pad;
                ty += tok_h + gap;
            }
            let r = Rect::new(tx, ty, wd, tok_h);
            let (bg, fg) = if i == step {
                (pal.primary, pal.on_primary)
            } else if i < step {
                (pal.bg, pal.muted)
            } else {
                (pal.bg, pal.text)
            };
            cv.fill_rrect(r.x, r.y, r.w, r.h, self.dp(11.0), bg);
            self.text_mid(cv, Face::Mono, 17.0, r, t, fg);
            tx += wd + gap;
        }
        y += card_h + self.dp(28.0);

        self.scroll_max = (y - y_start - area.h).max(0.0);
        if self.scroll[1] > self.scroll_max {
            self.scroll[1] = self.scroll_max;
        }
        cv.reset_clip();

        if close {
            self.close_lesson();
            return;
        }
        if let Some(i) = pick {
            self.select_case(i);
        }
        match ctl {
            Some(Ctl::Restart) => self.restart_case(),
            Some(Ctl::Prev) => self.step_prev(),
            Some(Ctl::Play) => self.toggle_play(),
            Some(Ctl::Next) => self.step_next(),
            None => {}
        }
    }

    // ------------------------------------------------------------------ practice
    fn draw_practice(&mut self, cv: &mut Canvas) {
        let pal = self.pal;
        let lang = self.lang();
        let w = self.w as f32;
        let m = self.dp(22.0);
        let top = self.insets.top + self.dp(14.0);
        self.text(cv, Face::Heading, 28.0, m, top, s::TAB_PRACTICE.get(lang), pal.text);
        self.text(cv, Face::Body, 13.0, m, top + self.dp(40.0), s::PRACTICE_SUB.get(lang), pal.muted);

        // bottom-up layout
        let bottom = self.h as f32 - self.tabbar_h() - self.dp(12.0);
        let gap = self.dp(8.0);
        let act_h = self.dp(46.0);
        let act_y = bottom - act_h;
        let bh = self.dp(44.0);
        let pad_h = bh * 3.0 + gap * 2.0;
        let pad_y = act_y - self.dp(14.0) - pad_h;
        let chip_h = self.dp(30.0);
        let chip_y = pad_y - self.dp(12.0) - chip_h;
        let cube_top = top + self.dp(68.0);
        let cube_rect = Rect::new(0.0, cube_top, w, (chip_y - self.dp(6.0) - cube_top).max(self.dp(120.0)));
        self.cube_rect = Some(cube_rect);
        let cube = self.cube_practice;
        self.render_cube(cv, cube_rect, &cube, true);

        // progress chips
        let pr = progress(&cube);
        let stages = [(pr.cross, s::ST_CROSS), (pr.layer1, s::ST_L1), (pr.f2l, s::ST_F2L), (pr.oll, s::ST_OLL), (pr.solved, s::ST_SOLVED)];
        let avail = w - 2.0 * m;
        let g = self.dp(6.0);
        let mut size = 11.0f32;
        let mut widths = [0f32; 5];
        loop {
            let mut total = g * 4.0;
            for (i, (_, t)) in stages.iter().enumerate() {
                widths[i] = self.text_w(Face::Body, size, t.get(lang)) + self.dp(18.0);
                total += widths[i];
            }
            if total <= avail || size <= 8.0 {
                break;
            }
            size -= 0.5;
        }
        let mut x = m;
        let t = self.hair();
        for (i, (done, label)) in stages.iter().enumerate() {
            let r = Rect::new(x, chip_y, widths[i], chip_h);
            if *done {
                cv.fill_rrect(r.x, r.y, r.w, r.h, r.h * 0.5, pal.accent.alpha(46));
                cv.stroke_rrect(r.x, r.y, r.w, r.h, r.h * 0.5, t, pal.accent);
            } else {
                cv.stroke_rrect(r.x, r.y, r.w, r.h, r.h * 0.5, t, pal.line);
            }
            let col = if *done { pal.text } else { pal.muted };
            self.text_mid(cv, Face::Body, size, r, label.get(lang), col);
            x += widths[i] + g;
        }

        // move pad: R L U D F B  x  (cw, ccw, half)
        let syms = ['R', 'L', 'U', 'D', 'F', 'B'];
        let bw = (w - 2.0 * m - gap * 5.0) / 6.0;
        let mut mv: Option<Move> = None;
        for row in 0..3usize {
            let turns = [1u8, 3, 2][row];
            for (c, &sym) in syms.iter().enumerate() {
                let r = Rect::new(m + c as f32 * (bw + gap), pad_y + row as f32 * (bh + gap), bw, bh);
                let mvv = Move::new(sym, turns);
                let name = mvv.name();
                if self.button(cv, r, &name, Face::Mono, 16.0, BtnStyle::Outline, self.dp(12.0)) {
                    mv = Some(mvv);
                }
            }
        }

        // actions
        let aw = (w - 2.0 * m - 2.0 * gap) / 3.0;
        let mut act = 0u8;
        if self.action_button(cv, Rect::new(m, act_y, aw, act_h), IC_SHUFFLE, s::SCRAMBLE.get(lang)) {
            act = 1;
        }
        if self.action_button(cv, Rect::new(m + aw + gap, act_y, aw, act_h), IC_UNDO, s::UNDO.get(lang)) {
            act = 2;
        }
        if self.action_button(cv, Rect::new(m + 2.0 * (aw + gap), act_y, aw, act_h), IC_REPLAY, s::RESET.get(lang)) {
            act = 3;
        }

        if let Some(mv) = mv {
            self.enqueue(mv, true);
        }
        match act {
            1 => self.scramble(),
            2 => self.undo(),
            3 => self.reset_practice(),
            _ => {}
        }
    }

    // ------------------------------------------------------------------ settings
    fn draw_settings(&mut self, cv: &mut Canvas) {
        let pal = self.pal;
        let lang = self.lang();
        let w = self.w as f32;
        let m = self.dp(22.0);
        let top = self.insets.top;
        let bottom = self.h as f32 - self.tabbar_h();
        let area = Rect::new(0.0, top, w, bottom - top);
        self.scroll_rect = Some(area);
        cv.set_clip(0, top as i32, self.w, bottom as i32);

        let mut y = top + self.dp(22.0) - self.scroll[2];
        let y_start = y;
        self.text(cv, Face::Heading, 28.0, m, y, s::TAB_SETTINGS.get(lang), pal.text);
        y += self.dp(62.0);

        let sh = self.dp(46.0);
        let tw = w - 2.0 * m;
        let mut new_lang = None;
        let mut new_theme = None;
        let mut new_speed = None;

        self.label(cv, m, y, s::LANGUAGE.get(lang));
        y += self.dp(24.0);
        let sel = match self.settings.lang {
            LangPref::System => 0,
            LangPref::Vi => 1,
            LangPref::En => 2,
        };
        if let Some(i) = self.segmented(cv, Rect::new(m, y, tw, sh), &[s::SYSTEM.get(lang), "Ti\u{1ebf}ng Vi\u{1ec7}t", "English"], sel) {
            new_lang = Some([LangPref::System, LangPref::Vi, LangPref::En][i]);
        }
        y += sh + self.dp(26.0);

        self.label(cv, m, y, s::APPEARANCE.get(lang));
        y += self.dp(24.0);
        let sel = match self.settings.theme {
            ThemePref::System => 0,
            ThemePref::Light => 1,
            ThemePref::Dark => 2,
        };
        if let Some(i) = self.segmented(cv, Rect::new(m, y, tw, sh), &[s::SYSTEM.get(lang), s::LIGHT.get(lang), s::DARK.get(lang)], sel) {
            new_theme = Some([ThemePref::System, ThemePref::Light, ThemePref::Dark][i]);
        }
        y += sh + self.dp(26.0);

        self.label(cv, m, y, s::SPEED.get(lang));
        y += self.dp(24.0);
        let sel = self.settings.speed.min(2) as usize;
        if let Some(i) = self.segmented(cv, Rect::new(m, y, tw, sh), &[s::SLOW.get(lang), s::NORMAL.get(lang), s::FAST.get(lang)], sel) {
            new_speed = Some(i as u8);
        }
        y += sh + self.dp(40.0);

        self.text_center(cv, Face::Heading, 18.0, w * 0.5, y, "Rubik", pal.text);
        y += self.dp(30.0);
        let ver = concat!("v", env!("CARGO_PKG_VERSION"));
        self.text_center(cv, Face::Body, 12.0, w * 0.5, y, ver, pal.muted);
        y += self.dp(22.0);
        self.text_center(cv, Face::Body, 12.0, w * 0.5, y, s::ABOUT.get(lang), pal.muted);
        y += self.dp(40.0);

        self.scroll_max = (y - y_start - area.h).max(0.0);
        if self.scroll[2] > self.scroll_max {
            self.scroll[2] = self.scroll_max;
        }
        cv.reset_clip();

        if let Some(l) = new_lang {
            self.set_lang_pref(l);
        }
        if let Some(t) = new_theme {
            self.set_theme_pref(t);
        }
        if let Some(v) = new_speed {
            self.set_speed(v);
        }
    }
}
