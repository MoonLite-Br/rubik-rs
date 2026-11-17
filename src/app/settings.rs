//! Persistent settings: a three-line text file in the app's private storage.

use crate::theme::ThemePref;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LangPref {
    System,
    Vi,
    En,
}

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub lang: LangPref,
    pub theme: ThemePref,
    /// 0 = slow, 1 = normal, 2 = fast
    pub speed: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { lang: LangPref::System, theme: ThemePref::System, speed: 1 }
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match (k.trim(), v.trim()) {
                    ("lang", "vi") => s.lang = LangPref::Vi,
                    ("lang", "en") => s.lang = LangPref::En,
                    ("lang", "system") => s.lang = LangPref::System,
                    ("theme", "light") => s.theme = ThemePref::Light,
                    ("theme", "dark") => s.theme = ThemePref::Dark,
                    ("theme", "system") => s.theme = ThemePref::System,
                    ("speed", n) => {
                        if let Ok(n) = n.parse::<u8>() {
                            s.speed = n.min(2);
                        }
                    }
                    _ => {}
                }
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        let lang = match self.lang {
            LangPref::System => "system",
            LangPref::Vi => "vi",
            LangPref::En => "en",
        };
        let theme = match self.theme {
            ThemePref::System => "system",
            ThemePref::Light => "light",
            ThemePref::Dark => "dark",
        };
        format!("lang={}\ntheme={}\nspeed={}\n", lang, theme, self.speed)
    }
}

pub fn load(dir: &Path) -> Settings {
    std::fs::read_to_string(dir.join("settings.txt")).map(|t| Settings::parse(&t)).unwrap_or_default()
}

pub fn save(dir: &Path, s: &Settings) {
    let _ = std::fs::write(dir.join("settings.txt"), s.to_text());
}
