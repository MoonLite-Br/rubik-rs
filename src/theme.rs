//! Light / dark palettes. "System" follows Android's night-mode flag.

use crate::gfx::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemePref {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color,
    pub card: Color,
    pub line: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub primary: Color,
    pub on_primary: Color,
    pub body: Color,
    pub shadow: Color,
    /// Sticker colours indexed by the face a sticker belongs to: U R F D L B.
    /// Scheme: yellow up, white down, green front, orange right, red left, blue back
    /// (a real cube turned so that white is on the bottom).
    pub sticker: [Color; 6],
}

const STICKERS: [Color; 6] = [
    Color::hex(0xF1C945), // U yellow
    Color::hex(0xE9803B), // R orange
    Color::hex(0x2FA46C), // F green
    Color::hex(0xF5F2EA), // D white
    Color::hex(0xD1403C), // L red
    Color::hex(0x2E6FD1), // B blue
];

impl Palette {
    pub fn light() -> Palette {
        Palette {
            dark: false,
            bg: Color::hex(0xFAF8F4),
            card: Color::hex(0xFFFFFF),
            line: Color::hex(0xE8E3D8),
            text: Color::hex(0x1D1B18),
            muted: Color::hex(0x8A857A),
            accent: Color::hex(0xB08D57),
            primary: Color::hex(0x1D1B18),
            on_primary: Color::hex(0xFAF8F4),
            body: Color::hex(0x1E1D1B),
            shadow: Color::hex(0x3A2F1C),
            sticker: STICKERS,
        }
    }

    pub fn dark() -> Palette {
        Palette {
            dark: true,
            bg: Color::hex(0x0E0E0F),
            card: Color::hex(0x19191B),
            line: Color::hex(0x2B2B2E),
            text: Color::hex(0xF1EDE4),
            muted: Color::hex(0x8E897F),
            accent: Color::hex(0xD4B26A),
            primary: Color::hex(0xD9BF8C),
            on_primary: Color::hex(0x14120E),
            body: Color::hex(0x2A2A2E),
            shadow: Color::hex(0x000000),
            sticker: STICKERS,
        }
    }

    pub fn resolve(pref: ThemePref, system_dark: bool) -> Palette {
        match pref {
            ThemePref::Light => Palette::light(),
            ThemePref::Dark => Palette::dark(),
            ThemePref::System => {
                if system_dark {
                    Palette::dark()
                } else {
                    Palette::light()
                }
            }
        }
    }
}
