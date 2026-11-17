//! Android glue: NativeActivity via `android-activity`, software rendering straight into the
//! window's pixel buffer (ANativeWindow_lock / unlockAndPost). No EGL, no GL driver, no Java.
//!
//! System integration (all pure NDK, no JNI):
//!   * night mode   <- AConfiguration uiModeNight, refreshed on every ConfigChanged
//!   * language     <- AConfiguration language ("vi" -> Vietnamese, anything else -> English)
//!   * system bars  <- content rect of the window -> insets
//!   * storage      <- app's private internal data dir (settings.txt)
//!   * lifecycle    <- no drawing while paused / without a window; idle loop sleeps
//!   * back button  <- closes the lesson / returns to the Learn tab before leaving the app
//!   * low memory   <- drops the glyph cache

use crate::app::{App, Insets};
use crate::gfx::Canvas;
use android_activity::input::{InputEvent, KeyAction, Keycode, MotionAction};
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use std::ffi::CString;
use std::time::{Duration, Instant};

const WINDOW_FORMAT_RGBA_8888: i32 = 1;
const WINDOW_FORMAT_RGBX_8888: i32 = 2;
/// Render at most this many pixels wide; the compositor scales up (saves RAM and CPU).
const MAX_RENDER_WIDTH: i32 = 1080;

fn log(msg: &str) {
    if let (Ok(tag), Ok(text)) = (CString::new("rubik"), CString::new(msg)) {
        unsafe {
            ndk_sys::__android_log_write(4, tag.as_ptr(), text.as_ptr());
        }
    }
}

struct Runtime {
    ui: App,
    window: Option<ndk::native_window::NativeWindow>,
    /// render pixels / window pixels (touch coordinates arrive in window pixels)
    px_scale: f32,
    paused: bool,
}

impl Runtime {
    fn win_ptr(&self) -> Option<*mut ndk_sys::ANativeWindow> {
        self.window.as_ref().map(|w| w.ptr().as_ptr().cast())
    }

    fn read_system(&mut self, app: &AndroidApp) {
        let cfg = app.config();
        let dark = matches!(cfg.ui_mode_night(), ndk::configuration::UiModeNight::Yes);
        let lang = cfg.language().unwrap_or_else(|| "en".to_string());
        self.ui.set_system(dark, &lang);
    }

    fn setup_surface(&mut self, app: &AndroidApp) {
        let Some(ptr) = self.win_ptr() else { return };
        unsafe {
            let nw = ndk_sys::ANativeWindow_getWidth(ptr);
            let nh = ndk_sys::ANativeWindow_getHeight(ptr);
            if nw <= 0 || nh <= 0 {
                return;
            }
            let rw = nw.min(MAX_RENDER_WIDTH);
            let rh = ((nh as i64 * rw as i64) / nw as i64) as i32;
            ndk_sys::ANativeWindow_setBuffersGeometry(ptr, rw, rh, WINDOW_FORMAT_RGBA_8888);
            self.px_scale = rw as f32 / nw as f32;

            let r = app.content_rect();
            let (mut top, mut bottom) = (0.0f32, 0.0f32);
            if r.right > r.left && r.bottom > r.top {
                top = r.top.max(0) as f32 * self.px_scale;
                bottom = (nh - r.bottom).max(0) as f32 * self.px_scale;
            }
            self.ui.set_viewport(rw, rh, Insets { top, bottom });
        }
    }

    fn draw(&mut self) {
        let Some(ptr) = self.win_ptr() else { return };
        unsafe {
            let mut buf: ndk_sys::ANativeWindow_Buffer = std::mem::zeroed();
            if ndk_sys::ANativeWindow_lock(ptr, &mut buf, std::ptr::null_mut()) != 0 {
                return;
            }
            let fmt_ok = buf.format == WINDOW_FORMAT_RGBA_8888 || buf.format == WINDOW_FORMAT_RGBX_8888;
            if !buf.bits.is_null() && fmt_ok && buf.stride >= buf.width && buf.width > 0 && buf.height > 0 {
                let len = buf.stride as usize * buf.height as usize;
                let px = std::slice::from_raw_parts_mut(buf.bits as *mut u32, len);
                let mut cv = Canvas::new(px, buf.width, buf.height, buf.stride as usize);
                self.ui.draw(&mut cv);
            }
            ndk_sys::ANativeWindow_unlockAndPost(ptr);
        }
    }

    fn handle_input(&mut self, ev: &InputEvent) -> InputStatus {
        match ev {
            InputEvent::MotionEvent(m) => {
                let p = m.pointer_at_index(0);
                let (x, y) = (p.x() * self.px_scale, p.y() * self.px_scale);
                match m.action() {
                    MotionAction::Down => self.ui.pointer_down(x, y),
                    MotionAction::Move => self.ui.pointer_move(x, y),
                    MotionAction::Up => self.ui.pointer_up(x, y),
                    MotionAction::Cancel => self.ui.pointer_cancel(),
                    _ => {}
                }
                InputStatus::Handled
            }
            InputEvent::KeyEvent(k) => {
                if k.key_code() == Keycode::Back && self.ui.can_go_back() {
                    if matches!(k.action(), KeyAction::Up) {
                        self.ui.back();
                    }
                    InputStatus::Handled
                } else {
                    InputStatus::Unhandled
                }
            }
            _ => InputStatus::Unhandled,
        }
    }

    fn pump_input(&mut self, app: &AndroidApp) {
        let Ok(mut iter) = app.input_events_iter() else { return };
        loop {
            let more = iter.next(|ev| self.handle_input(ev));
            if !more {
                break;
            }
        }
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    log("start");
    let mut rt = Runtime { ui: App::new(app.internal_data_path()), window: None, px_scale: 1.0, paused: false };
    rt.read_system(&app);

    let mut last = Instant::now();
    let mut quit = false;
    while !quit {
        // Idle: wake only a few times a second. Animating: ~60 Hz.
        let timeout = if rt.ui.animating() && !rt.paused { Duration::from_millis(6) } else { Duration::from_millis(250) };

        app.poll_events(Some(timeout), |event| {
            if let PollEvent::Main(ev) = event {
                match ev {
                    MainEvent::InitWindow { .. } => {
                        rt.window = app.native_window();
                        rt.setup_surface(&app);
                        rt.ui.dirty = true;
                    }
                    MainEvent::TerminateWindow { .. } => {
                        rt.window = None;
                    }
                    MainEvent::WindowResized { .. } | MainEvent::ContentRectChanged { .. } | MainEvent::RedrawNeeded { .. } => {
                        rt.setup_surface(&app);
                        rt.ui.dirty = true;
                    }
                    MainEvent::ConfigChanged { .. } => {
                        rt.read_system(&app);
                    }
                    MainEvent::Resume { .. } => {
                        rt.paused = false;
                        rt.ui.dirty = true;
                    }
                    MainEvent::Pause { .. } => {
                        rt.paused = true;
                    }
                    MainEvent::LowMemory { .. } => {
                        rt.ui.trim_memory();
                    }
                    MainEvent::Destroy { .. } => {
                        quit = true;
                    }
                    _ => {}
                }
            }
        });

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f32();
        last = now;

        rt.pump_input(&app);
        if !rt.paused && rt.window.is_some() {
            if rt.ui.animating() {
                rt.ui.tick(dt);
            }
            if rt.ui.needs_frame() {
                rt.draw();
            }
        }
    }
    log("exit");
}
