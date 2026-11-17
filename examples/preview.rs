//! Desktop preview without an emulator:  cargo run --example preview
//! Writes preview-<screen>-<light|dark>-<en|vi>.ppm into the current directory.

use rubik::app::{App, Insets, Tab};
use rubik::gfx::Canvas;
use std::io::Write;

fn main() {
    let (w, h) = (780usize, 1688usize);
    let screens: [(&str, Tab, Option<usize>); 4] =
        [("learn", Tab::Learn, None), ("lesson", Tab::Learn, Some(2)), ("practice", Tab::Practice, None), ("settings", Tab::Settings, None)];
    for dark in [false, true] {
        for lang in ["en", "vi"] {
            for (name, tab, lesson) in screens {
                let mut buf = vec![0u32; w * h];
                let mut app = App::new(None);
                app.set_system(dark, lang);
                app.set_viewport(w as i32, h as i32, Insets { top: 48.0, bottom: 0.0 });
                app.open_screen(tab, lesson);
                let mut cv = Canvas::new(&mut buf, w as i32, h as i32, w);
                app.draw(&mut cv);
                let file = format!("preview-{}-{}-{}.ppm", name, if dark { "dark" } else { "light" }, lang);
                let mut out = std::io::BufWriter::new(std::fs::File::create(&file).unwrap());
                write!(out, "P6\n{} {}\n255\n", w, h).unwrap();
                for p in &buf {
                    out.write_all(&[*p as u8, (*p >> 8) as u8, (*p >> 16) as u8]).unwrap();
                }
                println!("wrote {}", file);
            }
        }
    }
}
