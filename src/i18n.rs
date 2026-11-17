//! Vietnamese / English strings. No framework: a string is just a pair.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Vi,
}

impl Lang {
    /// "vi", "vi-VN", "VI" -> Vi, everything else -> En.
    pub fn from_code(code: &str) -> Lang {
        if code.to_ascii_lowercase().starts_with("vi") {
            Lang::Vi
        } else {
            Lang::En
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tr {
    pub en: &'static str,
    pub vi: &'static str,
}

impl Tr {
    pub const fn new(en: &'static str, vi: &'static str) -> Tr {
        Tr { en, vi }
    }
    pub fn get(&self, l: Lang) -> &'static str {
        match l {
            Lang::En => self.en,
            Lang::Vi => self.vi,
        }
    }
}

/// UI strings.
pub mod s {
    use super::Tr;

    pub const TAB_LEARN: Tr = Tr::new("Learn", "Học");
    pub const TAB_PRACTICE: Tr = Tr::new("Practice", "Luyện tập");
    pub const TAB_SETTINGS: Tr = Tr::new("Settings", "Cài đặt");

    pub const TAGLINE: Tr = Tr::new("Learn to solve the cube, step by step.", "Học giải Rubik từng bước một.");
    pub const LESSONS: Tr = Tr::new("LESSONS", "BÀI HỌC");
    pub const SITUATIONS: Tr = Tr::new("SITUATIONS", "TÌNH HUỐNG");
    pub const ALGORITHM: Tr = Tr::new("ALGORITHM", "THUẬT TOÁN");
    pub const ORIENTATION: Tr = Tr::new(
        "Hold the cube with white on the bottom and green in front.",
        "Cầm khối với mặt trắng ở dưới và mặt xanh lá ở trước.",
    );

    pub const PRACTICE_SUB: Tr = Tr::new("Tap a move, or scramble and solve it yourself.", "Chạm để xoay, hoặc xáo trộn rồi tự giải.");
    pub const SCRAMBLE: Tr = Tr::new("Scramble", "Xáo trộn");
    pub const UNDO: Tr = Tr::new("Undo", "Hoàn tác");
    pub const RESET: Tr = Tr::new("Reset", "Đặt lại");
    pub const ST_CROSS: Tr = Tr::new("Cross", "Dấu cộng");
    pub const ST_L1: Tr = Tr::new("Layer 1", "Tầng 1");
    pub const ST_F2L: Tr = Tr::new("Layers 1-2", "Tầng 1-2");
    pub const ST_OLL: Tr = Tr::new("Yellow face", "Mặt vàng");
    pub const ST_SOLVED: Tr = Tr::new("Solved", "Hoàn thành");

    pub const LANGUAGE: Tr = Tr::new("LANGUAGE", "NGÔN NGỮ");
    pub const APPEARANCE: Tr = Tr::new("APPEARANCE", "GIAO DIỆN");
    pub const SPEED: Tr = Tr::new("TURN SPEED", "TỐC ĐỘ XOAY");
    pub const SYSTEM: Tr = Tr::new("System", "Hệ thống");
    pub const LIGHT: Tr = Tr::new("Light", "Sáng");
    pub const DARK: Tr = Tr::new("Dark", "Tối");
    pub const SLOW: Tr = Tr::new("Slow", "Chậm");
    pub const NORMAL: Tr = Tr::new("Normal", "Vừa");
    pub const FAST: Tr = Tr::new("Fast", "Nhanh");
    pub const ABOUT: Tr = Tr::new("Pure Rust - drawn on the CPU - no WebView", "Thuần Rust - tự vẽ bằng CPU - không WebView");
}
