//! Host-side tests (run with `cargo test`): cube model, notation, tutorial data, fonts, renderer.

use rubik::app::{App, Insets, Tab};
use rubik::cube::{invert_alg, parse_alg, progress, random_scramble, Cube, Rng};
use rubik::gfx::{Canvas, Face, Fonts};
use rubik::i18n::Lang;
use rubik::tutorial::LESSONS;

fn facelets(c: &Cube) -> String {
    c.f.iter().map(|&x| b"URFDLB"[x as usize] as char).collect()
}

fn run(alg: &str) -> Cube {
    let mut c = Cube::solved();
    c.apply_all(&parse_alg(alg));
    c
}

fn order(alg: &str) -> usize {
    let a = parse_alg(alg);
    let mut c = Cube::solved();
    for n in 1..=1000 {
        c.apply_all(&a);
        if c.is_solved() {
            return n;
        }
    }
    0
}

#[test]
fn matches_kociemba_reference_for_r_u_f() {
    assert_eq!(facelets(&run("R")), "UUFUUFUUFRRRRRRRRRFFDFFDFFDDDBDDBDDBLLLLLLLLLUBBUBBUBB");
    assert_eq!(facelets(&run("U")), "UUUUUUUUUBBBRRRRRRRRRFFFFFFDDDDDDDDDFFFLLLLLLLLLBBBBBB");
    assert_eq!(facelets(&run("F")), "UUUUUULLLURRURRURRFFFFFFFFFRRRDDDDDDLLDLLDLLDBBBBBBBBB");
}

#[test]
fn move_orders() {
    assert_eq!(order("R"), 4);
    assert_eq!(order("R2"), 2);
    assert_eq!(order("R U R' U'"), 6);
    assert_eq!(order("R U R' U R U2 R'"), 6); // Sune
    assert_eq!(order("R' F R' B2 R F' R' B2 R2"), 3); // corner 3-cycle
    assert_eq!(order("R U' R U R U R U' R' U' R2"), 3); // edge 3-cycle
}

#[test]
fn slices_and_rotations_are_consistent() {
    assert_eq!(run("x"), run("R M' L'"));
    assert_eq!(run("y"), run("U E' D'"));
    assert_eq!(run("z"), run("F S B'"));
    assert_eq!(run("r"), run("R M'"));
    assert_eq!(run("Rw"), run("r"));
}

#[test]
fn scramble_then_inverse_is_identity() {
    let mut rng = Rng::new(12345);
    for _ in 0..50 {
        let s = random_scramble(&mut rng, 30);
        let mut c = Cube::solved();
        c.apply_all(&s);
        c.apply_all(&invert_alg(&s));
        assert!(c.is_solved());
    }
}

#[test]
fn every_lesson_case_is_valid_and_ends_solved() {
    // what the start state must already satisfy, per lesson
    for (li, lesson) in LESSONS.iter().enumerate() {
        for case in lesson.cases.iter() {
            let tokens = case.alg.split_whitespace().count();
            assert_eq!(parse_alg(case.alg).len(), tokens, "unparseable token in {}", case.alg);
            let mut c = case.start();
            let p = progress(&c);
            match li {
                1 => assert!(!p.cross, "{}", case.alg),
                2 => assert!(p.cross && !p.layer1, "{}", case.alg),
                3 => assert!(p.layer1 && !p.f2l, "{}", case.alg),
                4 | 5 => assert!(p.f2l && !p.oll, "{}", case.alg),
                6 | 7 => assert!(p.oll && !p.solved, "{}", case.alg),
                _ => {}
            }
            c.apply_all(&case.moves());
            if !case.from_solved {
                assert!(c.is_solved(), "{} does not solve its own case", case.alg);
            }
        }
    }
}

#[test]
fn text_has_no_missing_glyphs_in_both_languages() {
    let mut fonts = Fonts::new();
    for lang in [Lang::En, Lang::Vi] {
        fonts.lang = lang;
        for l in LESSONS.iter() {
            for t in [l.title, l.sub, l.intro] {
                let m = fonts.missing(Face::Body, t.get(lang));
                assert!(m.is_empty(), "{:?} missing {:?} in {:?}", lang, m, t.get(lang));
            }
            for c in l.cases.iter() {
                for t in [c.name, c.hint] {
                    let m = fonts.missing(Face::Body, t.get(lang));
                    assert!(m.is_empty(), "{:?} missing {:?}", lang, m);
                }
                assert!(fonts.missing(Face::Mono, c.alg).is_empty());
            }
            assert!(fonts.missing(Face::Heading, l.title.get(lang)).is_empty());
        }
    }
}

fn render(tab: Tab, lesson: Option<usize>, dark: bool, lang: &str) -> Vec<u32> {
    let (w, h) = (540usize, 1200usize);
    let mut buf = vec![0u32; w * h];
    let mut app = App::new(None);
    app.set_system(dark, lang);
    app.set_viewport(w as i32, h as i32, Insets { top: 20.0, bottom: 20.0 });
    app.open_screen(tab, lesson);
    let mut cv = Canvas::new(&mut buf, w as i32, h as i32, w);
    app.draw(&mut cv);
    buf
}

#[test]
fn every_screen_draws_in_both_themes_and_languages() {
    for dark in [false, true] {
        for lang in ["en", "vi"] {
            for (tab, lesson) in [(Tab::Learn, None), (Tab::Learn, Some(2)), (Tab::Practice, None), (Tab::Settings, None)] {
                let buf = render(tab, lesson, dark, lang);
                let bg = buf[0];
                let painted = buf.iter().filter(|&&p| p != bg).count();
                assert!(painted > 5_000, "{:?} {:?} dark={} {} drew only {} px", tab, lesson, dark, lang, painted);
            }
        }
    }
}
