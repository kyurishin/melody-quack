#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const PCBANG: (i32, i32, i32, i32) = (11, 12, 14, 14);
pub(crate) const CHAPEL: (i32, i32, i32, i32) = (14, 2, 19, 7);
pub(crate) const PCBANG_DOOR: Vec2 = Vec2::new(12.5, 14.4);
pub(crate) const CHAPEL_DOOR: Vec2 = Vec2::new(16.5, 7.4);
pub(crate) const GOOSE: Vec2 = Vec2::new(17.45, 7.3);
pub(crate) const BOARD: Vec2 = Vec2::new(14.7, 7.75);

pub(crate) const PC_ORIGIN: Vec2 = Vec2::new(200.0, 100.0);
pub(crate) const PC_SIZE: (f32, f32) = (9.0, 6.0);
pub(crate) const CH_ORIGIN: Vec2 = Vec2::new(300.0, 100.0);
pub(crate) const CH_SIZE: (f32, f32) = (11.0, 9.0);

#[derive(Clone, Copy, PartialEq, Default)]
pub(crate) enum Venue {
    #[default]
    None,
    PcBang,
    Chapel,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub(crate) enum Overlay {
    #[default]
    None,
    Flyer(usize),
    Door,
}

pub(crate) const SOUNDS: [&str; 3] = ["acid techno", "gothic techno", "hard trance"];

#[derive(Clone)]
pub(crate) struct Rave {
    pub(crate) code: String,
    pub(crate) title: String,
    pub(crate) seed: u32,
    pub(crate) sound: usize,
    pub(crate) bpm: f32,
    pub(crate) expired: bool,
}

#[derive(Default)]
pub(crate) struct VenueState {
    pub(crate) at: Venue,
    pub(crate) overlay: Overlay,
    pub(crate) raves: Vec<Rave>,
    pub(crate) decoys: Vec<Rave>,
    pub(crate) known: Vec<String>,
    pub(crate) rng: u32,
    pub(crate) pc_open: bool,
    pub(crate) pc_boot: f32,
    pub(crate) pc_app: u8,
    pub(crate) pc_page: u8,
    pub(crate) pc_title: String,
    pub(crate) pc_sound: usize,
    pub(crate) pc_bpm: f32,
    pub(crate) pc_made: Option<usize>,
    pub(crate) talk: f32,
    pub(crate) door_code: String,
    pub(crate) door_msg: String,
    pub(crate) now: Option<Rave>,
    pub(crate) secret: Option<Rave>,
    pub(crate) term: Vec<String>,
    pub(crate) term_in: String,
    pub(crate) term_goose: bool,
    pub(crate) term_pw: bool,
    pub(crate) flag: bool,
    pub(crate) rave_start: f64,
    pub(crate) last_step: i64,
    pub(crate) pulse: f32,
    pub(crate) crowd: Vec<Player>,
    pub(crate) live_clock: f32,
    pub(crate) live: String,
}

fn next(rng: &mut u32) -> u32 {
    let mut x = *rng;
    if x == 0 {
        x = 0x9e37_79b9;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *rng = x;
    x
}

const ALPHABET: &[u8] = b"ACDEFGHJKMNPQRTUVWXY3479";

pub(crate) fn make_code(rng: &mut u32) -> String {
    let mut s = String::new();
    for i in 0..9 {
        if i == 3 || i == 6 {
            s.push('-');
        }
        s.push(ALPHABET[(next(rng) % ALPHABET.len() as u32) as usize] as char);
    }
    s
}

pub(crate) fn tidy_code(input: &str) -> Option<String> {
    let chars: Vec<char> = input.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect();
    if chars.len() != 9 || !chars.iter().all(|c| ALPHABET.contains(&(*c as u8))) {
        return None;
    }
    let mut s = String::new();
    for (i, c) in chars.iter().enumerate() {
        if i == 3 || i == 6 {
            s.push('-');
        }
        s.push(*c);
    }
    Some(s)
}

pub(crate) fn rot13(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
            'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
            _ => c,
        })
        .collect()
}

impl VenueState {
    pub(crate) fn run(&mut self, line: &str) -> bool {
        let user = if self.term_goose { "goose" } else { "guest" };
        if self.term_pw {
            self.term_pw = false;
            self.term.push("password: ***".to_string());
            if line.trim() == "132" {
                self.term_goose = true;
                self.term.push("welcome back, goose. honk.".to_string());
            } else {
                self.term.push("wrong password. honk.".to_string());
            }
            return false;
        }
        self.term.push(format!("{user}@pc7:~$ {line}"));
        let line = line.trim();
        let (cmd, arg) = line.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((line, ""));
        let mut won = false;
        let mut say = |t: &str| self.term.push(t.to_string());
        match (cmd, arg) {
            ("", _) => {}
            ("help", _) => {
                say("commands: ls  ls -a  cat <file>  su <user>");
                say("          rot13 <text>  whoami  clear");
                say("goal: find tonight's secret rave code.");
            }
            ("ls", "-a") | ("ls", "-la") | ("ls", "-al") => say(".rave  notes.txt  ramyeon.log"),
            ("ls", _) => say("notes.txt  ramyeon.log"),
            ("whoami", _) => say(user),
            ("clear", _) => {}
            ("cat", "notes.txt") => {
                say("kim: stop asking me about the chapel.");
                say("the goose keeps tonight's code in a hidden");
                say("file on this machine. his password is the");
                say("tempo of his favourite night. he scribbled");
                say("it at the bottom of the underground page.");
            }
            ("cat", "ramyeon.log") => {
                say("03:12 spill on pc 4.");
                say("03:13 denied everything.");
            }
            ("cat", ".rave") => {
                if user == "goose" {
                    let r = self.secret.clone().unwrap();
                    say(&format!("tonight: {}", r.title));
                    say(&format!("code, scrambled: {}", rot13(&r.code)));
                    say("every letter moved 13 along. rot13 it back.");
                } else {
                    say("permission denied. this file belongs to goose.");
                    say("try: su goose");
                }
            }
            ("cat", "") => say("cat what? try: ls"),
            ("cat", _) => say("no such file."),
            ("su", "goose") => {
                self.term_pw = true;
            }
            ("su", _) => say("no such user. the goose runs this place."),
            ("rot13", "") => say("rot13 what?"),
            ("rot13", text) => {
                let out = rot13(text);
                let hit = tidy_code(&out).filter(|c| self.secret.as_ref().map(|r| &r.code == c).unwrap_or(false));
                self.term.push(out);
                if let Some(code) = hit {
                    self.term.push("flag captured. the goose will know this code.".to_string());
                    self.learn(&code);
                    self.flag = true;
                    won = true;
                }
            }
            _ => say("unknown command. type help."),
        }
        if cmd == "clear" {
            self.term.clear();
        }
        let n = self.term.len();
        if n > 40 {
            self.term.drain(..n - 40);
        }
        won
    }

    pub(crate) fn new() -> VenueState {
        let mut v = VenueState { pc_bpm: 132.0, pc_sound: 1, rng: 0x51ab_1e55, ..Default::default() };
        let titles = ["acid mass", "bass vespers", "feather fever", "pond life", "midnight moult", "duckstep"];
        for (i, t) in titles.iter().enumerate() {
            let code = make_code(&mut v.rng);
            let seed = next(&mut v.rng);
            v.decoys.push(Rave { code, title: t.to_string(), seed, sound: i % 3, bpm: 128.0 + (i * 3) as f32, expired: true });
        }
        v.rng ^= (get_time() * 1000.0) as u32 | 1;
        let code = make_code(&mut v.rng);
        let seed = next(&mut v.rng);
        v.secret = Some(Rave { code, title: "the long waddle".to_string(), seed, sound: 1, bpm: 132.0, expired: false });
        v.term = vec!["quack 95 terminal. type help.".to_string()];
        v
    }

    pub(crate) fn all_flyers(&self) -> Vec<&Rave> {
        self.raves.iter().rev().chain(self.decoys.iter()).collect()
    }

    pub(crate) fn learn(&mut self, code: &str) {
        if !self.known.iter().any(|c| c == code) {
            self.known.insert(0, code.to_string());
            self.known.truncate(5);
        }
    }

    pub(crate) fn host(&mut self, title: &str, sound: usize, bpm: f32) -> usize {
        let code = make_code(&mut self.rng);
        let seed = next(&mut self.rng);
        let title = if title.trim().is_empty() { "acid mass".to_string() } else { title.trim().to_lowercase() };
        self.learn(&code);
        self.raves.push(Rave { code, title, seed, sound, bpm, expired: false });
        self.raves.len() - 1
    }
}

pub(crate) fn stamp(world: &mut World) {
    for (x0, z0, x1, z1) in [PCBANG, CHAPEL] {
        for z in z0..z1 {
            for x in x0..x1 {
                if let Some(t) = world.get_mut(x, z) {
                    t.kind = Kind::House;
                    t.fire = 0.0;
                    t.stack = 0;
                }
            }
        }
    }
    let mut clear = |x: i32, z: i32, kind: Kind| {
        if let Some(t) = world.get_mut(x, z) {
            if t.solid() || kind == Kind::Path {
                t.kind = kind;
                t.fire = 0.0;
                t.stack = 0;
            }
        }
    };
    for x in PCBANG.0..PCBANG.2 {
        clear(x, PCBANG.3, Kind::Path);
        clear(x, PCBANG.3 + 1, Kind::Grass);
    }
    for x in 13..CHAPEL.2 {
        clear(x, CHAPEL.3, Kind::Path);
    }
    for x in 12..18 {
        clear(x, CHAPEL.3 + 1, Kind::Path);
    }
    for x in 12..19 {
        clear(x, CHAPEL.3 + 2, Kind::Grass);
    }
}

pub(crate) fn outdoor_hint(p: Vec2) -> Option<&'static str> {
    if p.distance(PCBANG_DOOR) < 0.8 {
        Some("E: go into the pc bang")
    } else if p.distance(BOARD) < 0.9 {
        Some("E: read the flyers")
    } else if p.distance(CHAPEL_DOOR) < 0.9 || p.distance(GOOSE) < 0.9 {
        Some("E: talk to the goose")
    } else {
        None
    }
}

const SIGN: [&str; 11] = [
    "XXXX...XXX..X...X....X.",
    "X...X.X...X.X...X....X.",
    "X...X.X.....XXXXX....XX",
    "X...X.X.....X...X....X.",
    "XXXX..X.....XXXXX....X.",
    "X.....X................",
    "X.....X........XXXXX...",
    "X.....X...X...X.....X..",
    "X......XXX....X.....X..",
    "..............X.....X..",
    "...............XXXXX...",
];

fn sign(b: &mut Batch, left: f32, top: f32, z: f32, px: f32, time: f32, fade: f32) {
    for (r, row) in SIGN.iter().enumerate() {
        for (i, ch) in row.chars().enumerate() {
            if ch == 'X' {
                let hue = if i < 11 { 0.5 } else { 0.86 };
                let glow = 0.85 + 0.15 * (time * 3.0 + i as f32 * 0.3).sin();
                let c = hsv(hue, 0.7, glow);
                b.cube(vec3(left + i as f32 * px, top - r as f32 * px, z), vec3(px, px, 0.03), Color::new(c.r, c.g, c.b, fade));
            }
        }
    }
}

pub(crate) fn lancet(b: &mut Batch, x: f32, base: f32, z: f32, w: f32, h: f32, col: Color, depth: f32) {
    b.cube(vec3(x, base + h * 0.5, z), vec3(w, h, depth), col);
    for (k, f) in [0.75, 0.5, 0.22].iter().enumerate() {
        let step = w * 0.28;
        b.cube(vec3(x, base + h + step * (k as f32 + 0.5), z), vec3(w * f, step, depth), col);
    }
}

fn behind(focus: Vec2, r: (i32, i32, i32, i32), reach: f32) -> bool {
    focus.y < r.1 as f32 && focus.y > r.1 as f32 - reach && focus.x > r.0 as f32 - 0.8 && focus.x < r.2 as f32 + 0.8
}

pub(crate) fn draw_exteriors(b: &mut Batch, dusk: f32, see_through: bool, focus: Vec2, time: f32) {
    let was_unlit = b.unlit;

    if behind(focus, PCBANG, 2.4) == see_through {
        let a = if see_through { 0.35 } else { 1.0 };
        let fade = |c: Color| Color::new(c.r, c.g, c.b, a);
        let (x0, z0, x1, z1) = PCBANG;
        let (fx0, fz0, fx1, fz1) = (x0 as f32, z0 as f32, x1 as f32, z1 as f32);
        let mid = vec3((fx0 + fx1) / 2.0, 0.0, (fz0 + fz1) / 2.0);
        let wall = Color::from_rgba(86, 80, 118, 255);
        let wall_dark = Color::from_rgba(70, 64, 100, 255);
        for level in 0..6 {
            let c = if level % 2 == 0 { wall } else { wall_dark };
            b.cube(vec3(mid.x, 0.5 + 0.14 + level as f32 * 0.28, mid.z), vec3(fx1 - fx0, 0.28, fz1 - fz0), fade(c));
        }
        b.cube(vec3(mid.x, 2.24, mid.z), vec3(fx1 - fx0 + 0.2, 0.12, fz1 - fz0 + 0.2), fade(Color::from_rgba(46, 42, 70, 255)));
        b.cube(vec3(fx0 + 0.6, 2.46, fz0 + 0.6), vec3(0.6, 0.32, 0.4), fade(Color::from_rgba(190, 190, 200, 255)));
        b.cube(vec3(fx0 + 0.6, 2.46, fz0 + 0.81), vec3(0.3, 0.2, 0.02), fade(Color::from_rgba(90, 90, 110, 255)));
        if !see_through {
            let front = fz1 + 0.02;
            b.cube(vec3(mid.x, 1.82, front), vec3(1.5, 0.66, 0.05), Color::from_rgba(16, 12, 28, 255));
            b.unlit = true;
            sign(b, mid.x - 0.55, 2.07, front + 0.04, 0.048, time, 1.0);
            let glass = mix(Color::from_rgba(70, 110, 200, 255), Color::from_rgba(110, 150, 255, 255), (dusk * 2.0).min(1.0));
            for wx in [fx0 + 0.7, fx1 - 0.7] {
                b.cube(vec3(wx, 1.0, front), vec3(1.0, 0.72, 0.04), glass);
                b.unlit = false;
                b.cube(vec3(wx, 0.86, front + 0.05), vec3(0.34, 0.3, 0.06), Color::from_rgba(226, 216, 190, 255));
                b.unlit = true;
                b.cube(vec3(wx, 0.88, front + 0.09), vec3(0.24, 0.18, 0.02), Color::from_rgba(40, 220, 210, 255));
            }
            b.cube(vec3(mid.x, 0.92, front), vec3(0.5, 0.84, 0.04), Color::from_rgba(120, 160, 230, 255));
            b.cube(vec3(mid.x, 1.42, front + 0.02), vec3(0.3, 0.08, 0.03), Color::from_rgba(255, 110, 90, 255));
            b.unlit = was_unlit;
            b.cube(vec3(mid.x, 0.92, front + 0.03), vec3(0.03, 0.84, 0.03), Color::from_rgba(50, 54, 90, 255));
            b.cube(vec3(mid.x, 0.53, front + 0.2), vec3(0.8, 0.06, 0.4), Color::from_rgba(150, 146, 170, 255));
        }
    }

    if behind(focus, CHAPEL, 2.8) == see_through {
        let a = if see_through { 0.35 } else { 1.0 };
        let fade = |c: Color| Color::new(c.r, c.g, c.b, a);
        let (x0, z0, x1, z1) = CHAPEL;
        let (fx0, fz0, fx1, fz1) = (x0 as f32, z0 as f32, x1 as f32, z1 as f32);
        let mid_x = (fx0 + fx1) / 2.0;
        let mid_z = (fz0 + fz1) / 2.0;
        let conc = Color::from_rgba(150, 146, 162, 255);
        let conc_dark = Color::from_rgba(118, 114, 132, 255);
        let conc_deep = Color::from_rgba(78, 74, 94, 255);
        for level in 0..8 {
            let c = if level % 2 == 0 { conc } else { conc_dark };
            b.cube(vec3(mid_x, 0.5 + 0.16 + level as f32 * 0.32, mid_z), vec3(fx1 - fx0, 0.32, fz1 - fz0), fade(c));
        }
        b.cube(vec3(mid_x, 3.14, mid_z), vec3(fx1 - fx0 + 0.3, 0.16, fz1 - fz0 + 0.3), fade(conc_deep));
        for level in 0..6 {
            let c = if level % 2 == 0 { conc_dark } else { conc };
            b.cube(vec3(mid_x, 3.4 + level as f32 * 0.32, fz1 - 0.9), vec3(1.5, 0.32, 1.5), fade(c));
        }
        for (dx, dz) in [(-0.6, -0.6), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6)] {
            b.cube(vec3(mid_x + dx, 5.3, fz1 - 0.9 + dz), vec3(0.3, 0.3, 0.3), fade(conc_deep));
        }
        for fx in [fx0 - 0.08, fx1 + 0.08] {
            for k in 0..3 {
                b.cube(vec3(fx, 1.6, fz0 + 0.8 + k as f32 * 1.7), vec3(0.24, 2.3, 0.4), fade(conc_dark));
            }
        }
        if !see_through {
            let front = fz1 + 0.02;
            let beat = 0.7 + 0.3 * (time * 2.2 * std::f32::consts::TAU).sin().max(0.0);
            lancet(b, mid_x, 0.5, front, 1.1, 1.25, conc_deep, 0.06);
            b.cube(vec3(mid_x - 0.22, 1.02, front + 0.03), vec3(0.4, 1.04, 0.04), Color::from_rgba(70, 66, 92, 255));
            b.cube(vec3(mid_x + 0.22, 1.02, front + 0.03), vec3(0.4, 1.04, 0.04), Color::from_rgba(70, 66, 92, 255));
            b.unlit = true;
            b.cube(vec3(mid_x, 1.95, front + 0.06), vec3(0.09, 0.09, 0.05), Color::from_rgba(255, 60, 90, 255));
            for (wx, hue) in [(fx0 + 0.75, 0.86), (fx0 + 1.6, 0.74), (fx1 - 1.6, 0.6), (fx1 - 0.75, 0.5)] {
                let c = hsv(hue, 0.75, beat);
                lancet(b, wx, 1.0, front, 0.34, 1.1, c, 0.04);
            }
            for k in 0..8 {
                let ang = k as f32 / 8.0 * std::f32::consts::TAU;
                let c = hsv(0.86 - 0.04 * k as f32, 0.7, beat);
                b.cube(vec3(mid_x + ang.cos() * 0.3, 2.55 + ang.sin() * 0.3, front + 0.02), vec3(0.14, 0.14, 0.04), c);
            }
            b.cube(vec3(mid_x, 4.2, fz1 - 0.14), vec3(0.2, 1.2, 0.04), hsv(0.86, 0.7, beat));
            b.unlit = was_unlit;
            b.cube(vec3(mid_x, 0.53, front + 0.25), vec3(1.6, 0.06, 0.5), conc_dark);
        }
    }
    if see_through {
        return;
    }

    let g = vec3(GOOSE.x, 0.5, GOOSE.y);
    let bob = (time * 1.6).sin() * 0.01;
    let black = Color::from_rgba(30, 28, 44, 255);
    b.cube(g + vec3(0.0, 0.3, 0.0), vec3(0.44, 0.5, 0.4), black);
    b.cube(g + vec3(0.0, 0.42, 0.21), vec3(0.1, 0.24, 0.02), WHITE);
    b.cube(g + vec3(0.0, 0.8 + bob, 0.04), vec3(0.16, 0.56, 0.16), WHITE);
    b.cube(g + vec3(0.0, 1.14 + bob, 0.06), vec3(0.24, 0.2, 0.26), WHITE);
    b.cube(g + vec3(0.0, 1.1 + bob, 0.26), vec3(0.14, 0.07, 0.16), Color::from_rgba(255, 154, 60, 255));
    b.cube(g + vec3(0.0, 1.17 + bob, 0.2), vec3(0.27, 0.06, 0.03), Color::from_rgba(10, 10, 20, 255));
    b.cube(g + vec3(-0.12, 0.03, 0.06), vec3(0.12, 0.06, 0.2), Color::from_rgba(255, 154, 60, 255));
    b.cube(g + vec3(0.12, 0.03, 0.06), vec3(0.12, 0.06, 0.2), Color::from_rgba(255, 154, 60, 255));

    let p = vec3(BOARD.x, 0.5, BOARD.y - 0.45);
    b.cube(p + vec3(-0.6, 0.45, 0.0), vec3(0.08, 0.9, 0.08), Color::from_rgba(78, 74, 94, 255));
    b.cube(p + vec3(0.6, 0.45, 0.0), vec3(0.08, 0.9, 0.08), Color::from_rgba(78, 74, 94, 255));
    b.cube(p + vec3(0.0, 0.7, 0.0), vec3(1.3, 0.7, 0.06), Color::from_rgba(150, 146, 162, 255));
    FLYER_COLOURS.with(|f| {
        for (i, (c1, c2)) in f.borrow().iter().enumerate().take(4) {
            let x = -0.45 + i as f32 * 0.3;
            let tilt = (hash(i as i32, 3, 7) - 0.5) * 0.08;
            b.cube(p + vec3(x, 0.72 + tilt, 0.04), vec3(0.24, 0.36, 0.02), *c1);
            b.cube(p + vec3(x, 0.78 + tilt, 0.055), vec3(0.14, 0.14, 0.01), *c2);
            b.cube(p + vec3(x + 0.03, 0.62 + tilt, 0.055), vec3(0.14, 0.05, 0.01), WHITE);
        }
    });
}

thread_local! {
    pub(crate) static FLYER_COLOURS: std::cell::RefCell<Vec<(Color, Color)>> = std::cell::RefCell::new(Vec::new());
}

pub(crate) fn flyer_palette(seed: u32) -> (Color, Color, Color, Color) {
    let schemes = [
        (rgba(255, 242, 61), rgba(17, 17, 17), rgba(255, 47, 208), rgba(17, 17, 17)),
        (rgba(47, 243, 255), rgba(90, 31, 209), rgba(255, 242, 61), rgba(16, 8, 42)),
        (rgba(255, 138, 31), rgba(31, 61, 255), rgba(255, 255, 255), rgba(10, 10, 42)),
        (rgba(182, 255, 61), rgba(255, 61, 154), rgba(17, 17, 17), rgba(17, 17, 17)),
        (rgba(255, 42, 42), rgba(255, 242, 61), rgba(31, 107, 255), rgba(17, 17, 17)),
        (rgba(217, 200, 255), rgba(36, 26, 71), rgba(255, 61, 242), rgba(13, 10, 26)),
    ];
    schemes[(seed % 6) as usize]
}

pub(crate) const FLYER_W: usize = 30;
pub(crate) const FLYER_H: usize = 42;

pub(crate) fn draw_flyer(r: &Rave, x: f32, y: f32, cell: f32) {
    let (a, b, c, ink) = flyer_palette(r.seed);
    let (w, h) = (FLYER_W as f32 * cell, FLYER_H as f32 * cell);
    ui_rect(x + cell, y + cell, w, h, Color::new(0.0, 0.0, 0.0, 0.45));
    let style = (r.seed >> 3) % 4;
    let (cx, cy) = (FLYER_W as f32 / 2.0, FLYER_H as f32 * 0.46);
    for py in 0..FLYER_H {
        for px in 0..FLYER_W {
            let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
            let on = match style {
                0 => ((dy.atan2(dx) / std::f32::consts::TAU + 0.5) * 16.0) as i32 % 2 == 0,
                1 => ((dx * dx + dy * dy).sqrt() / 2.6) as i32 % 2 == 0,
                2 => ((px as f32 + (py as f32 * 0.45).sin() * 3.0) / 3.0).floor() as i32 % 2 == 0,
                _ => {
                    if py as f32 > cy {
                        let depth = (py as f32 - cy) / (FLYER_H as f32 - cy);
                        ((dx / (0.4 + depth * 2.2)).floor() as i32 + (depth * depth * 9.0) as i32) % 2 == 0
                    } else {
                        py % 4 < 2
                    }
                }
            };
            ui_rect(x + px as f32 * cell, y + py as f32 * cell, cell, cell, if on { a } else { b });
        }
    }
    let rad = 6.5;
    for py in 0..FLYER_H {
        for px in 0..FLYER_W {
            let (dx, dy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
            let d = (dx * dx + dy * dy).sqrt();
            if d < rad {
                let eye = (dx.abs() - 2.4).abs() < 0.8 && (dy + 2.0).abs() < 1.4;
                let smile = (d - 4.2).abs() < 0.7 && dy > 1.2;
                let col = if d > rad - 1.0 || eye || smile { ink } else { rgba(255, 226, 60) };
                ui_rect(x + px as f32 * cell, y + py as f32 * cell, cell, cell, col);
            }
        }
    }
    if cell < 2.5 {
        let cw = w * 0.7;
        ui_rect(x + w * 0.15, y + h * 0.62, cw, 4.0 * cell, WHITE);
        ui_rect(x + cell, y + 2.0 * cell, w - 2.0 * cell, 5.0 * cell, Color::new(ink.r, ink.g, ink.b, 0.9));
        ui_rect(x + 4.0 * cell, y + 4.0 * cell, w - 8.0 * cell, cell, c);
        if r.expired {
            ui_rect(x, y, w, h, Color::new(0.82, 0.78, 0.7, 0.4));
        }
        return;
    }
    let band_y = y + 2.0 * cell;
    ui_rect(x + cell, band_y, w - 2.0 * cell, 7.0 * cell, Color::new(ink.r, ink.g, ink.b, 0.9));
    let title = r.title.to_uppercase();
    let mut size = 6.0 * cell;
    while ui_measure(&title, size) > w - 4.0 * cell && size > 3.0 {
        size -= 0.5;
    }
    ui_text(&title, x + (w - ui_measure(&title, size)) / 2.0 + cell * 0.4, band_y + 5.2 * cell + cell * 0.4, size, c);
    ui_text(&title, x + (w - ui_measure(&title, size)) / 2.0, band_y + 5.2 * cell, size, WHITE);
    let info_y = y + h - 11.0 * cell;
    ui_rect(x + cell, info_y, w - 2.0 * cell, 10.0 * cell, Color::new(ink.r, ink.g, ink.b, 0.88));
    let lines = [
        format!("{} / {} BPM", SOUNDS[r.sound % 3].to_uppercase(), r.bpm as i32),
        if r.expired { "ONE NIGHT ONLY".to_string() } else { "DOORS OPEN".to_string() },
        "ST. BASSILICA".to_string(),
    ];
    for (i, line) in lines.iter().enumerate() {
        let s = (2.6 * cell).max(5.0);
        ui_text(line, x + (w - ui_measure(line, s)) / 2.0, info_y + (3.0 + i as f32 * 3.0) * cell, s, if i == 0 { a } else { WHITE });
    }
    let s = (3.4 * cell).max(6.0);
    let cw = ui_measure(&r.code, s) + 3.0 * cell;
    let sx = x + cell * (1.0 + hash(r.seed as i32 & 0xffff, 1, 11) * 3.0);
    let sy = y + h * (0.56 + hash(r.seed as i32 & 0xffff, 2, 11) * 0.12);
    ui_rect(sx + cell * 0.6, sy + cell * 0.6, cw, 4.6 * cell, Color::new(0.0, 0.0, 0.0, 0.5));
    ui_rect(sx, sy, cw, 4.6 * cell, WHITE);
    ui_text(&r.code, sx + 1.5 * cell, sy + 3.5 * cell, s, BLACK);
    let (kx, ky) = ui_scale();
    let rot = (hash(r.seed as i32 & 0xffff, 3, 11) - 0.5) * 1.3;
    let (tx, ty) = (x + w * 0.2, y + h * (0.3 + hash(r.seed as i32 & 0xffff, 4, 11) * 0.1));
    for (ox, oy, col) in [(1.0, 1.0, BLACK), (0.0, 0.0, if r.seed % 2 == 0 { rgba(198, 255, 61) } else { rgba(255, 61, 242) })] {
        draw_text_ex(
            &r.code,
            (tx * kx + ox).round(),
            (ty * ky + oy).round(),
            TextParams { font_size: (s * 0.9 * ky).round() as u16, rotation: rot, color: col, ..Default::default() },
        );
    }
    if r.expired {
        ui_rect(x, y, w, h, Color::new(0.82, 0.78, 0.7, 0.4));
        draw_line(x * kx, (y + h * 0.15) * ky, (x + w) * kx, (y + h * 0.8) * ky, 2.0 * ky, Color::new(0.85, 0.16, 0.16, 0.85));
    }
}
