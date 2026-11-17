//! Platform-independent application state: navigation, animation queue, tutorial player,
//! pointer handling. Drawing lives in `screens.rs`, widgets in `ui.rs`.

pub mod screens;
pub mod settings;
pub mod ui;

use crate::cube::{random_scramble, Cube, Move, Rng};
use crate::gfx::cube3d::ViewRot;
use crate::gfx::Fonts;
use crate::i18n::Lang;
use crate::theme::{Palette, ThemePref};
use crate::tutorial::Player;
use settings::{LangPref, Settings};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use ui::Rect;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Learn,
    Practice,
    Settings,
}

/// Space (in pixels) taken by system bars when the window is edge-to-edge.
#[derive(Clone, Copy, Default, Debug)]
pub struct Insets {
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Drag {
    None,
    Cube,
    Scroll,
}

pub(crate) struct Pointer {
    pub x: f32,
    pub y: f32,
    pub px: f32,
    pub py: f32,
    pub down: bool,
    pub moved: f32,
    pub drag: Drag,
    pub tap: Option<(f32, f32)>,
}

pub(crate) struct Anim {
    pub mv: Move,
    pub t: f32,
    pub dur: f32,
    pub rec: bool,
}

pub struct App {
    pub(crate) settings: Settings,
    pub(crate) sys_dark: bool,
    pub(crate) sys_lang: Lang,
    pub(crate) pal: Palette,
    pub(crate) fonts: Fonts,
    pub(crate) w: i32,
    pub(crate) h: i32,
    pub(crate) scale: f32,
    pub(crate) insets: Insets,
    pub(crate) tab: Tab,

    pub(crate) cube_lesson: Cube,
    pub(crate) cube_practice: Cube,
    pub(crate) anim: Option<Anim>,
    pub(crate) queue: VecDeque<(Move, bool)>,
    pub(crate) history: Vec<Move>,
    pub(crate) view: ViewRot,

    pub(crate) player: Option<Player>,
    pub(crate) playing: bool,
    pub(crate) wait: f32,

    /// Scroll offsets: [learn list, lesson text, settings]
    pub(crate) scroll: [f32; 3],
    pub(crate) scroll_max: f32,
    pub(crate) cube_rect: Option<Rect>,
    pub(crate) scroll_rect: Option<Rect>,
    pub(crate) pointer: Pointer,
    pub(crate) tap_used: bool,

    rng: Rng,
    storage: Option<PathBuf>,
    pub dirty: bool,
}

impl App {
    pub fn new(storage: Option<PathBuf>) -> App {
        let settings = match &storage {
            Some(d) => settings::load(d),
            None => Settings::default(),
        };
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        let mut app = App {
            settings,
            sys_dark: false,
            sys_lang: Lang::En,
            pal: Palette::light(),
            fonts: Fonts::new(),
            w: 1080,
            h: 2400,
            scale: 2.77,
            insets: Insets::default(),
            tab: Tab::Learn,
            cube_lesson: Cube::solved(),
            cube_practice: Cube::solved(),
            anim: None,
            queue: VecDeque::new(),
            history: Vec::new(),
            view: ViewRot::default(),
            player: None,
            playing: false,
            wait: 0.0,
            scroll: [0.0; 3],
            scroll_max: 0.0,
            cube_rect: None,
            scroll_rect: None,
            pointer: Pointer { x: 0.0, y: 0.0, px: 0.0, py: 0.0, down: false, moved: 0.0, drag: Drag::None, tap: None },
            tap_used: false,
            rng: Rng::new(seed),
            storage,
            dirty: true,
        };
        app.refresh_theme();
        app
    }

    // ------------------------------------------------------------ system integration
    pub fn lang(&self) -> Lang {
        match self.settings.lang {
            LangPref::Vi => Lang::Vi,
            LangPref::En => Lang::En,
            LangPref::System => self.sys_lang,
        }
    }

    /// Called with Android's current night-mode flag and locale (e.g. "vi", "en-US").
    pub fn set_system(&mut self, dark: bool, lang_code: &str) {
        self.sys_dark = dark;
        self.sys_lang = Lang::from_code(lang_code);
        self.refresh_theme();
    }

    pub fn set_viewport(&mut self, w: i32, h: i32, insets: Insets) {
        self.w = w;
        self.h = h;
        self.scale = (w as f32 / 390.0).clamp(0.75, 3.5);
        self.insets = insets;
        self.dirty = true;
    }

    /// Free caches (Android low-memory callback).
    pub fn trim_memory(&mut self) {
        self.fonts.trim();
    }

    fn refresh_theme(&mut self) {
        self.pal = Palette::resolve(self.settings.theme, self.sys_dark);
        self.fonts.lang = self.lang();
        self.dirty = true;
    }

    fn save_settings(&self) {
        if let Some(d) = &self.storage {
            settings::save(d, &self.settings);
        }
    }

    pub(crate) fn set_lang_pref(&mut self, p: LangPref) {
        self.settings.lang = p;
        self.refresh_theme();
        self.save_settings();
    }

    pub(crate) fn set_theme_pref(&mut self, p: ThemePref) {
        self.settings.theme = p;
        self.refresh_theme();
        self.save_settings();
    }

    pub(crate) fn set_speed(&mut self, s: u8) {
        self.settings.speed = s.min(2);
        self.dirty = true;
        self.save_settings();
    }

    // ------------------------------------------------------------ navigation
    pub fn can_go_back(&self) -> bool {
        self.tab != Tab::Learn || self.player.is_some()
    }

    /// Android back button. Returns true if it was consumed.
    pub fn back(&mut self) -> bool {
        if self.tab == Tab::Learn && self.player.is_some() {
            self.close_lesson();
            true
        } else if self.tab != Tab::Learn {
            self.switch_tab(Tab::Learn);
            true
        } else {
            false
        }
    }

    pub(crate) fn switch_tab(&mut self, t: Tab) {
        if t != self.tab {
            self.flush();
            self.playing = false;
            self.tab = t;
            self.dirty = true;
        }
    }

    fn active_cube(&mut self) -> &mut Cube {
        if self.tab == Tab::Practice {
            &mut self.cube_practice
        } else {
            &mut self.cube_lesson
        }
    }

    // ------------------------------------------------------------ animation
    pub fn animating(&self) -> bool {
        self.anim.is_some() || !self.queue.is_empty() || self.playing
    }

    pub fn needs_frame(&self) -> bool {
        self.dirty || self.anim.is_some()
    }

    /// Finish everything that is queued or in flight instantly.
    pub(crate) fn flush(&mut self) {
        let mut items: Vec<(Move, bool)> = Vec::new();
        if let Some(a) = self.anim.take() {
            items.push((a.mv, a.rec));
        }
        while let Some(q) = self.queue.pop_front() {
            items.push(q);
        }
        for (m, rec) in items {
            self.active_cube().apply(m);
            if rec {
                self.history.push(m);
            }
        }
        self.dirty = true;
    }

    pub(crate) fn enqueue(&mut self, mv: Move, rec: bool) {
        if self.queue.len() < 24 {
            self.queue.push_back((mv, rec));
        }
        self.dirty = true;
    }

    fn start_anim(&mut self, mv: Move, rec: bool) {
        let speed = [1.5f32, 1.0, 0.6][self.settings.speed.min(2) as usize];
        let mut dur = 0.30 * speed;
        if mv.turns == 2 {
            dur *= 1.45;
        }
        if self.queue.len() > 2 {
            dur *= 0.5;
        }
        self.anim = Some(Anim { mv, t: 0.0, dur, rec });
    }

    /// Advance animations by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.min(0.05);
        let mut finished: Option<(Move, bool)> = None;
        if let Some(a) = self.anim.as_mut() {
            a.t += dt / a.dur;
            if a.t >= 1.0 {
                finished = Some((a.mv, a.rec));
            }
        }
        if self.anim.is_some() {
            self.dirty = true;
        }
        if let Some((mv, rec)) = finished {
            self.anim = None;
            self.active_cube().apply(mv);
            if rec {
                self.history.push(mv);
            }
            if self.playing {
                self.wait = 0.18;
            }
        }
        if self.anim.is_none() {
            if self.wait > 0.0 {
                self.wait -= dt;
                return;
            }
            if let Some((mv, rec)) = self.queue.pop_front() {
                self.start_anim(mv, rec);
            } else if self.playing {
                let next = self.player.as_mut().and_then(|p| p.next());
                match next {
                    Some(m) => self.start_anim(m, false),
                    None => {
                        self.playing = false;
                        self.dirty = true;
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ tutorial
    fn reload_case(&mut self) {
        self.anim = None;
        self.queue.clear();
        self.playing = false;
        self.wait = 0.0;
        if let Some(p) = &self.player {
            self.cube_lesson = p.start();
        }
        self.dirty = true;
    }

    pub(crate) fn open_lesson(&mut self, li: usize) {
        self.flush();
        self.player = Some(Player::new(li, 0));
        self.reload_case();
        self.scroll[1] = 0.0;
        self.tab = Tab::Learn;
    }

    pub(crate) fn close_lesson(&mut self) {
        self.flush();
        self.player = None;
        self.playing = false;
        self.dirty = true;
    }

    pub(crate) fn select_case(&mut self, ci: usize) {
        self.flush();
        let li = self.player.as_ref().map(|p| p.lesson);
        if let Some(li) = li {
            self.player = Some(Player::new(li, ci));
            self.reload_case();
        }
    }

    pub(crate) fn restart_case(&mut self) {
        if let Some(p) = self.player.as_mut() {
            p.step = 0;
        }
        self.reload_case();
    }

    pub(crate) fn step_next(&mut self) {
        self.playing = false;
        let m = self.player.as_mut().and_then(|p| p.next());
        if let Some(m) = m {
            self.enqueue(m, false);
        }
    }

    pub(crate) fn step_prev(&mut self) {
        self.playing = false;
        let m = self.player.as_mut().and_then(|p| p.prev());
        if let Some(m) = m {
            self.enqueue(m, false);
        }
    }

    pub(crate) fn toggle_play(&mut self) {
        if self.playing {
            self.playing = false;
            return;
        }
        let at_end = self.player.as_ref().map(|p| p.at_end()).unwrap_or(true);
        if at_end {
            self.restart_case();
        }
        self.playing = true;
        self.dirty = true;
    }

    // ------------------------------------------------------------ practice
    pub(crate) fn scramble(&mut self) {
        self.flush();
        let s = random_scramble(&mut self.rng, 22);
        self.cube_practice.apply_all(&s);
        self.history.clear();
        self.dirty = true;
    }

    pub(crate) fn reset_practice(&mut self) {
        self.anim = None;
        self.queue.clear();
        self.cube_practice = Cube::solved();
        self.history.clear();
        self.dirty = true;
    }

    pub(crate) fn undo(&mut self) {
        if let Some(m) = self.history.pop() {
            self.enqueue(m.inverse(), false);
        }
    }

    // ------------------------------------------------------------ pointer
    fn slop(&self) -> f32 {
        self.scale * 10.0
    }

    fn scroll_idx(&self) -> usize {
        match self.tab {
            Tab::Learn => {
                if self.player.is_some() {
                    1
                } else {
                    0
                }
            }
            _ => 2,
        }
    }

    pub fn pointer_down(&mut self, x: f32, y: f32) {
        let drag = if self.cube_rect.map(|r| r.contains(x, y)).unwrap_or(false) {
            Drag::Cube
        } else if self.scroll_rect.map(|r| r.contains(x, y)).unwrap_or(false) {
            Drag::Scroll
        } else {
            Drag::None
        };
        self.pointer = Pointer { x, y, px: x, py: y, down: true, moved: 0.0, drag, tap: None };
        self.dirty = true;
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        if !self.pointer.down {
            return;
        }
        let dx = x - self.pointer.x;
        let dy = y - self.pointer.y;
        self.pointer.x = x;
        self.pointer.y = y;
        self.pointer.moved += dx.abs() + dy.abs();
        if self.pointer.moved > self.slop() {
            match self.pointer.drag {
                Drag::Cube => {
                    self.view.yaw += dx * 0.011;
                    self.view.pitch = (self.view.pitch + dy * 0.011).clamp(-1.35, 1.35);
                }
                Drag::Scroll => {
                    let i = self.scroll_idx();
                    let max = self.scroll_max;
                    self.scroll[i] = (self.scroll[i] - dy).clamp(0.0, max);
                }
                Drag::None => {}
            }
        }
        self.dirty = true;
    }

    pub fn pointer_up(&mut self, x: f32, y: f32) {
        if self.pointer.down && self.pointer.moved <= self.slop() {
            self.pointer.tap = Some((x, y));
        }
        self.pointer.down = false;
        self.dirty = true;
    }

    pub fn pointer_cancel(&mut self) {
        self.pointer.down = false;
        self.dirty = true;
    }
}

impl App {
    /// Jump straight to a screen (used by the desktop preview example and for deep links).
    pub fn open_screen(&mut self, tab: Tab, lesson: Option<usize>) {
        self.flush();
        self.player = None;
        self.tab = tab;
        if let Some(i) = lesson {
            self.open_lesson(i);
        }
        self.dirty = true;
    }
}
