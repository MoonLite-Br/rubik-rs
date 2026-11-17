//! Rubik - a tiny pure-Rust Rubik's cube tutor.
//!
//! Layers (each one only depends on the ones above it):
//!   cube      facelet model, moves, stage detection        (std only, unit-tested on the host)
//!   tutorial  lessons, algorithms, step player             (std only)
//!   i18n      Vietnamese / English strings
//!   theme     light / dark palettes
//!   gfx       software canvas, fontdue text, 3-D cube
//!   app       immediate-mode UI + screens (platform independent)
//!   platform  Android glue (android-activity + ANativeWindow)

pub mod app;
pub mod cube;
pub mod gfx;
pub mod i18n;
pub mod theme;
pub mod tutorial;

#[cfg(target_os = "android")]
mod platform;
