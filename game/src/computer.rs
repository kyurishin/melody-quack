//! "Quack 95": the desktop on every PC방 computer. A fat beige CRT, a teal
//! desktop, grey bevelled windows, a Start button and a taskbar clock.
//! Original look, inspired by the era; no real OS branding.

use crate::gfx::{hex, mix, pal, with_alpha, Fonts};
use crate::poster::{FLYER_H, FLYER_W};
use crate::ui::Ui;
use macroquad::prelude::*;
use mq_shared::{RaveInfo, Sound};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum App {
    Browser,
    RaveMaker,
    Readme,
}

impl App {
    fn title(self) -> &'static str {
        match self {
            App::Browser => "Quacknet Explorer",
            App::RaveMaker => "RAVE.EXE",
            App::Readme => "readme.txt - Notepad",
        }
    }
}

struct Win {
    app: App,
    rect: Rect,
}

pub enum Action {
    ShutDown,
    Host { title: String, sound: Sound, bpm: f32, flyers: bool, doors_in_min: u32 },
    Guestbook(String),
    Copy(String),
    OpenFlyer(usize),
}

pub struct Ctx<'a> {
    pub flyers: &'a [(crate::poster::Flyer, Texture2D)],
    pub guestbook: &'a [(String, String)],
    pub hosted: &'a [RaveInfo],
    pub hosted_flyers: &'a [Texture2D],
    pub is_dj: bool,
    pub name: &'a str,
    pub clock: String,
    pub crt: bool,
}

pub struct Computer {
    wins: Vec<Win>,
    start_open: bool,
    boot: f32,
    drag: Option<(usize, Vec2)>,
    last_icon_click: (usize, f32),
    // Browser
    url: String,
    history: Vec<String>,
    scroll: f32,
    gb_text: String,
    // Rave maker
    rm_title: String,
    rm_sound: Sound,
    rm_bpm: f32,
    rm_flyers: bool,
    rm_doors: u32,
    rm_view: usize,
}

const SCREEN_W: f32 = 400.0;
const SCREEN_H: f32 = 288.0;

impl Computer {
    pub fn new() -> Computer {
        Computer {
            wins: Vec::new(),
            start_open: false,
            boot: 0.0,
            drag: None,
            last_icon_click: (99, -1.0),
            url: "quack://home".into(),
            history: Vec::new(),
            scroll: 0.0,
            gb_text: String::new(),
            rm_title: String::new(),
            rm_sound: Sound::Gothic,
            rm_bpm: Sound::Gothic.default_bpm(),
            rm_flyers: true,
            rm_doors: 0,
            rm_view: 0,
        }
    }

    /// Fresh boot each time you sit down.
    pub fn power_on(&mut self) {
        self.wins.clear();
        self.start_open = false;
        self.boot = 0.0;
        self.drag = None;
        self.scroll = 0.0;
    }

    pub fn open(&mut self, app: App) {
        if let Some(i) = self.wins.iter().position(|w| w.app == app) {
            let w = self.wins.remove(i);
            self.wins.push(w);
            return;
        }
        let n = self.wins.len() as f32;
        let rect = match app {
            App::Browser => Rect::new(30.0 + n * 10.0, 12.0 + n * 8.0, 330.0, 236.0),
            App::RaveMaker => Rect::new(60.0 + n * 10.0, 20.0 + n * 8.0, 280.0, 236.0),
            App::Readme => Rect::new(90.0, 40.0, 230.0, 170.0),
        };
        self.wins.push(Win { app, rect });
    }

    pub fn navigate(&mut self, url: &str) {
        if self.url != url {
            self.history.push(self.url.clone());
        }
        self.url = url.to_string();
        self.scroll = 0.0;
        self.open(App::Browser);
    }

    pub fn show_hosted(&mut self, idx: usize) {
        self.rm_view = idx + 1;
        self.open(App::RaveMaker);
    }

    /// Draw and run one frame. Returns what the player asked for.
    pub fn frame(&mut self, ui: &mut Ui, fonts: &Fonts, ctx: &Ctx) -> Vec<Action> {
        let mut acts = Vec::new();
        self.boot += get_frame_time();
        let s = ui.s;

        // The monitor, centred.
        let bez = Rect::new((ui.w - SCREEN_W - 56.0) / 2.0, (ui.h - SCREEN_H - 50.0) / 2.0 - 6.0, SCREEN_W + 56.0, SCREEN_H + 50.0);
        ui.dim(0.75);
        // Beige bezel with depth.
        ui.rect(Rect::new(bez.x + 6.0, bez.y + 6.0, bez.w, bez.h), pal::INK);
        ui.rect(bez, pal::INK);
        let b = Rect::new(bez.x + 2.0, bez.y + 2.0, bez.w - 4.0, bez.h - 4.0);
        ui.rect(b, pal::BEIGE);
        ui.rect(Rect::new(b.x, b.y + b.h - 18.0, b.w, 18.0), pal::BEIGE_D);
        ui.rect(Rect::new(b.x + b.w - 40.0, b.y + b.h - 12.0, 10.0, 4.0), pal::ACID);
        ui.text(&fonts.small, "QUACKTRON 15", b.x + 16.0, b.y + b.h - 6.0, 7.0, mix(pal::BEIGE_D, pal::INK, 0.5));
        let scr = Rect::new(bez.x + 28.0, bez.y + 16.0, SCREEN_W, SCREEN_H);
        ui.rect(Rect::new(scr.x - 4.0, scr.y - 4.0, scr.w + 8.0, scr.h + 8.0), hex(0x1b1a22));

        // Origin of the desktop in virtual units.
        let o = vec2(scr.x, scr.y);
        let m = ui.mouse - o;

        if self.boot < 1.6 {
            ui.rect(scr, pal::BLACK);
            let t = self.boot;
            ui.text(&fonts.mono, "QUACKBIOS v4.20", o.x + 8.0, o.y + 16.0, 14.0, pal::W95_GREY);
            if t > 0.3 {
                ui.text(&fonts.mono, "640K RAM OK   CD-ROM OK   MODEM OK", o.x + 8.0, o.y + 30.0, 14.0, pal::W95_GREY);
            }
            if t > 0.7 {
                ui.rect(Rect::new(scr.x, scr.y, scr.w, scr.h), pal::W95_TEAL);
                ui.text_center(&fonts.title, "Quack 95", o.x + SCREEN_W / 2.0, o.y + SCREEN_H / 2.0, 30.0, pal::WHITE);
                ui.text_center(&fonts.small, "STARTING UP...", o.x + SCREEN_W / 2.0, o.y + SCREEN_H / 2.0 + 20.0, 8.0, pal::CREAM);
            }
            self.crt(ui, scr, ctx.crt);
            return acts;
        }

        // Desktop.
        ui.rect(scr, pal::W95_TEAL);
        // Icons.
        let icons: [(App, &str); 3] = [(App::Browser, "Quacknet"), (App::RaveMaker, "RAVE.EXE"), (App::Readme, "readme.txt")];
        for (i, (app, label)) in icons.iter().enumerate() {
            let r = Rect::new(o.x + 10.0, o.y + 10.0 + i as f32 * 50.0, 52.0, 44.0);
            let hover = ui.hover(r) && self.top_at(m).is_none();
            if hover {
                ui.set_hot();
                ui.rect(r, with_alpha(pal::W95_NAVY, 0.35));
            }
            self.icon(ui, *app, vec2(r.x + 16.0, r.y + 4.0));
            ui.text_center(&fonts.small, label, r.x + r.w / 2.0, r.y + 38.0, 7.0, pal::WHITE);
            if hover && ui.clicked {
                ui.click_used = true;
                let now = ui.time;
                if self.last_icon_click.0 == i && now - self.last_icon_click.1 < 0.45 {
                    self.open(*app);
                    if *app == App::RaveMaker {
                        self.rm_view = 0;
                    }
                } else {
                    self.last_icon_click = (i, now);
                }
            }
        }

        // Windows (back to front). Clicking brings to front.
        if ui.clicked && !self.start_open {
            if let Some(i) = self.top_at(m) {
                if i != self.wins.len() - 1 {
                    let w = self.wins.remove(i);
                    self.wins.push(w);
                }
            }
        }
        if !ui.down {
            self.drag = None;
        }
        if let Some((i, off)) = self.drag {
            if let Some(w) = self.wins.get_mut(i) {
                w.rect.x = (m.x - off.x).clamp(-w.rect.w + 40.0, SCREEN_W - 40.0);
                w.rect.y = (m.y - off.y).clamp(0.0, SCREEN_H - 40.0);
            }
        }
        let mut close = None;
        let n = self.wins.len();
        for i in 0..n {
            let active = i == n - 1;
            let wr = self.wins[i].rect;
            let r = Rect::new(o.x + wr.x, o.y + wr.y, wr.w, wr.h);
            bevel(ui, r, true);
            // Title bar.
            let tb = Rect::new(r.x + 3.0, r.y + 3.0, r.w - 6.0, 13.0);
            let (c1, c2) = if active { (pal::W95_NAVY, hex(0x4a6ad8)) } else { (pal::W95_DARK, hex(0xa8a5a0)) };
            for k in 0..8 {
                let kk = k as f32 / 8.0;
                ui.rect(Rect::new(tb.x + tb.w * kk, tb.y, tb.w / 8.0 + 0.5, tb.h), mix(c1, c2, kk));
            }
            ui.text(&fonts.body, self.wins[i].app.title(), tb.x + 4.0, tb.y + 10.0, 9.0, pal::WHITE);
            let xb = Rect::new(tb.x + tb.w - 12.0, tb.y + 2.0, 10.0, 9.0);
            bevel(ui, xb, true);
            ui.text(&fonts.small, "x", xb.x + 2.5, xb.y + 7.0, 7.0, pal::BLACK);
            if active && !self.start_open {
                if ui.hover(xb) {
                    ui.set_hot();
                    if ui.clicked {
                        close = Some(i);
                        ui.click_used = true;
                    }
                } else if ui.hover(tb) && ui.clicked {
                    self.drag = Some((i, m - vec2(wr.x, wr.y)));
                    ui.click_used = true;
                }
            }
            let body = Rect::new(r.x + 3.0, r.y + 18.0, r.w - 6.0, r.h - 21.0);
            let interactive = active && !self.start_open && self.drag.is_none();
            // Clip contents.
            let mut gl = unsafe { get_internal_gl() };
            gl.flush();
            gl.quad_gl.scissor(Some((
                (body.x * s) as i32,
                (body.y * s) as i32,
                (body.w * s) as i32,
                (body.h * s) as i32,
            )));
            let app = self.wins[i].app;
            match app {
                App::Browser => self.browser(ui, fonts, ctx, body, interactive, &mut acts),
                App::RaveMaker => self.rave_maker(ui, fonts, ctx, body, interactive, &mut acts),
                App::Readme => readme(ui, fonts, body, ctx),
            }
            let mut gl = unsafe { get_internal_gl() };
            gl.flush();
            gl.quad_gl.scissor(None);
        }
        if let Some(i) = close {
            self.wins.remove(i);
        }

        // Taskbar.
        let bar = Rect::new(scr.x, scr.y + scr.h - 18.0, scr.w, 18.0);
        ui.rect(bar, pal::W95_GREY);
        ui.rect(Rect::new(bar.x, bar.y, bar.w, 1.0), pal::WHITE);
        let start = Rect::new(bar.x + 2.0, bar.y + 2.0, 46.0, 14.0);
        bevel(ui, start, !self.start_open);
        self.icon_small(ui, vec2(start.x + 3.0, start.y + 2.0));
        ui.text(&fonts.body, "Start", start.x + 16.0, start.y + 10.5, 9.0, pal::BLACK);
        if ui.hover(start) {
            ui.set_hot();
            if ui.clicked {
                self.start_open = !self.start_open;
                ui.click_used = true;
            }
        }
        // Task buttons.
        let mut tx = start.x + 52.0;
        let n = self.wins.len();
        for i in 0..n {
            let r = Rect::new(tx, bar.y + 2.0, 74.0, 14.0);
            bevel(ui, r, i != n - 1);
            let t = self.wins[i].app.title();
            let short: String = t.chars().take(12).collect();
            ui.text(&fonts.body, &short, r.x + 4.0, r.y + 10.5, 8.0, pal::BLACK);
            if ui.hover(r) && ui.clicked {
                let w = self.wins.remove(i);
                self.wins.push(w);
                ui.click_used = true;
                break;
            }
            tx += 77.0;
        }
        let tray = Rect::new(bar.x + bar.w - 48.0, bar.y + 2.0, 46.0, 14.0);
        bevel(ui, tray, false);
        ui.text_center(&fonts.body, &ctx.clock, tray.x + tray.w / 2.0, tray.y + 10.5, 9.0, pal::BLACK);

        // Start menu.
        if self.start_open {
            let items: [(&str, Option<App>); 4] = [
                ("Quacknet Explorer", Some(App::Browser)),
                ("RAVE.EXE", Some(App::RaveMaker)),
                ("readme.txt", Some(App::Readme)),
                ("Shut Down...", None),
            ];
            let mr = Rect::new(bar.x + 2.0, bar.y - 4.0 - items.len() as f32 * 18.0 - 6.0, 130.0, items.len() as f32 * 18.0 + 6.0);
            bevel(ui, mr, true);
            ui.rect(Rect::new(mr.x + 3.0, mr.y + 3.0, 14.0, mr.h - 6.0), pal::W95_NAVY);
            for (k, (label, app)) in items.iter().enumerate() {
                let r = Rect::new(mr.x + 19.0, mr.y + 3.0 + k as f32 * 18.0, mr.w - 22.0, 18.0);
                let hov = ui.hover(r);
                if hov {
                    ui.set_hot();
                    ui.rect(r, pal::W95_NAVY);
                }
                ui.text(&fonts.body, label, r.x + 4.0, r.y + 12.5, 9.0, if hov { pal::WHITE } else { pal::BLACK });
                if hov && ui.clicked {
                    ui.click_used = true;
                    self.start_open = false;
                    match app {
                        Some(a) => self.open(*a),
                        None => acts.push(Action::ShutDown),
                    }
                }
            }
            if ui.clicked && !ui.click_used {
                self.start_open = false;
                ui.click_used = true;
            }
        }

        self.crt(ui, scr, ctx.crt);
        // Hint under the monitor.
        ui.text_center(
            &fonts.small,
            "ESC TO STAND UP",
            ui.w / 2.0,
            (bez.y + bez.h + 14.0).min(ui.h - 4.0),
            8.0,
            with_alpha(pal::CREAM, 0.7),
        );
        acts
    }

    fn top_at(&self, m: Vec2) -> Option<usize> {
        (0..self.wins.len()).rev().find(|&i| self.wins[i].rect.contains(m))
    }

    fn crt(&self, ui: &Ui, scr: Rect, on: bool) {
        if !on {
            return;
        }
        // Scanlines and a soft vignette.
        let mut y = scr.y;
        while y < scr.y + scr.h {
            ui.rect(Rect::new(scr.x, y, scr.w, 0.5), with_alpha(pal::BLACK, 0.12));
            y += 2.0;
        }
        for k in 0..6 {
            let a = 0.05 * (6 - k) as f32 / 6.0;
            let i = k as f32 * 2.0;
            ui.rect_lines(Rect::new(scr.x + i, scr.y + i, scr.w - i * 2.0, scr.h - i * 2.0), 2.0, with_alpha(pal::BLACK, a * 2.0));
        }
    }

    fn icon(&self, ui: &Ui, app: App, p: Vec2) {
        let r = |x: f32, y: f32, w: f32, h: f32, c: Color| ui.rect(Rect::new(p.x + x, p.y + y, w, h), c);
        match app {
            App::Browser => {
                r(2.0, 2.0, 18.0, 18.0, pal::INK);
                r(3.0, 3.0, 16.0, 16.0, pal::BLUE);
                r(6.0, 6.0, 6.0, 5.0, pal::GREEN);
                r(11.0, 12.0, 5.0, 4.0, pal::GREEN);
                r(6.0, 9.0, 18.0, 4.0, pal::YELLOW); // ring
                r(8.0, 10.0, 10.0, 1.0, pal::INK);
            }
            App::RaveMaker => {
                r(1.0, 1.0, 20.0, 20.0, pal::INK);
                r(2.0, 2.0, 18.0, 18.0, pal::BLACK);
                r(4.0, 4.0, 14.0, 14.0, hex(0xfff23d));
                r(7.0, 7.0, 2.0, 3.0, pal::INK);
                r(13.0, 7.0, 2.0, 3.0, pal::INK);
                r(7.0, 13.0, 8.0, 2.0, pal::INK);
                r(6.0, 12.0, 2.0, 1.0, pal::INK);
                r(14.0, 12.0, 2.0, 1.0, pal::INK);
            }
            App::Readme => {
                r(3.0, 1.0, 16.0, 20.0, pal::INK);
                r(4.0, 2.0, 14.0, 18.0, pal::WHITE);
                for k in 0..5 {
                    r(6.0, 5.0 + k as f32 * 3.0, 10.0, 1.0, pal::W95_DARK);
                }
            }
        }
    }

    fn icon_small(&self, ui: &Ui, p: Vec2) {
        // Little duck head.
        ui.rect(Rect::new(p.x + 1.0, p.y + 1.0, 8.0, 8.0), pal::INK);
        ui.rect(Rect::new(p.x + 2.0, p.y + 2.0, 6.0, 6.0), pal::YELLOW);
        ui.rect(Rect::new(p.x + 8.0, p.y + 5.0, 4.0, 2.0), hex(0xff9a3d));
        ui.rect(Rect::new(p.x + 5.0, p.y + 3.0, 1.0, 1.0), pal::INK);
    }

    // ----- Browser -----

    fn browser(&mut self, ui: &mut Ui, fonts: &Fonts, ctx: &Ctx, r: Rect, live: bool, acts: &mut Vec<Action>) {
        // Toolbar.
        ui.rect(r, pal::W95_GREY);
        let back = Rect::new(r.x + 2.0, r.y + 2.0, 30.0, 14.0);
        bevel(ui, back, true);
        ui.text(&fonts.body, "Back", back.x + 4.0, back.y + 10.5, 8.0, if self.history.is_empty() { pal::W95_DARK } else { pal::BLACK });
        if live && ui.hover(back) && ui.clicked {
            ui.click_used = true;
            if let Some(u) = self.history.pop() {
                self.url = u;
                self.scroll = 0.0;
            }
        }
        let home = Rect::new(r.x + 34.0, r.y + 2.0, 30.0, 14.0);
        bevel(ui, home, true);
        ui.text(&fonts.body, "Home", home.x + 4.0, home.y + 10.5, 8.0, pal::BLACK);
        if live && ui.hover(home) && ui.clicked {
            ui.click_used = true;
            self.navigate("quack://home");
        }
        let addr = Rect::new(r.x + 66.0, r.y + 2.0, r.w - 68.0, 14.0);
        let mut url = self.url.clone();
        let entered = if live {
            ui.field(fonts, "url", addr, &mut url, 40, "quack://")
        } else {
            ui.rect(addr, pal::WHITE);
            ui.text(&fonts.body, &url, addr.x + 5.0, addr.y + 10.5, 8.0, pal::BLACK);
            false
        };
        if url != self.url {
            self.url = url;
        }
        if entered {
            let mut u = self.url.trim().to_lowercase();
            if !u.starts_with("quack://") {
                u = format!("quack://{}", u.trim_start_matches("http://").trim_start_matches("www."));
            }
            ui.focus = None;
            self.history.push("quack://home".into());
            self.url = u;
            self.scroll = 0.0;
        }

        // Page.
        let page = Rect::new(r.x, r.y + 18.0, r.w, r.h - 18.0);
        if live && ui.hover(page) {
            self.scroll = (self.scroll - ui.wheel * 12.0).max(0.0);
        }
        let y0 = page.y - self.scroll;
        let mut nav: Option<String> = None;
        let mut link = |ui: &mut Ui, label: &str, x: f32, y: f32, to: &str, color: Color| {
            let w = ui.measure(&fonts.mono, label, 12.0);
            let rr = Rect::new(x, y - 9.0, w, 11.0);
            let hov = live && ui.hover(rr) && page.contains(ui.mouse);
            ui.text(&fonts.mono, label, x, y, 12.0, if hov { pal::MAGENTA } else { color });
            ui.rect(Rect::new(x, y + 1.0, w, 1.0), if hov { pal::MAGENTA } else { color });
            if hov {
                ui.set_hot();
                if ui.clicked {
                    ui.click_used = true;
                    nav = Some(to.to_string());
                }
            }
        };
        let path = self.url.trim_start_matches("quack://").trim_end_matches('/').to_string();
        let max_scroll;
        match path.as_str() {
            "home" | "" => {
                // Tiled background.
                ui.rect(page, hex(0xfdf3d0));
                let mut ty = page.y;
                while ty < page.y + page.h {
                    let mut tx = page.x + ((ty / 16.0) as i32 % 2) as f32 * 8.0;
                    while tx < page.x + page.w {
                        ui.rect(Rect::new(tx, ty, 2.0, 2.0), hex(0xf2dfa8));
                        tx += 16.0;
                    }
                    ty += 8.0;
                }
                let x = page.x + 10.0;
                let mut y = y0 + 22.0;
                ui.text_shadow(&fonts.title, "QUACKVILLE ONLINE", x, y, 18.0, pal::CORAL, 1.0);
                y += 12.0;
                // Marquee.
                let mq = "*** welcome to the information superpondway *** sign the guestbook *** ";
                let off = (ui.time * 30.0) % (ui.measure(&fonts.mono, mq, 12.0));
                ui.rect(Rect::new(page.x, y, page.w, 12.0), pal::INK);
                ui.text(&fonts.mono, mq, page.x - off, y + 9.0, 12.0, pal::ACID);
                ui.text(&fonts.mono, mq, page.x - off + ui.measure(&fonts.mono, mq, 12.0), y + 9.0, 12.0, pal::ACID);
                y += 26.0;
                ui.text(&fonts.mono, &format!("hi {}! you are visitor no.", ctx.name), x, y, 12.0, pal::INK);
                let counter = format!("{:06}", 1337 + ctx.guestbook.len() * 7);
                let cw = ui.measure(&fonts.mono, &counter, 12.0) + 6.0;
                let cx = x + ui.measure(&fonts.mono, &format!("hi {}! you are visitor no. ", ctx.name), 12.0);
                ui.rect(Rect::new(cx, y - 9.0, cw, 12.0), pal::BLACK);
                ui.text(&fonts.mono, &counter, cx + 3.0, y, 12.0, pal::ACID);
                y += 20.0;
                link(ui, "> town news", x, y, "quack://news", hex(0x1a2fd0));
                y += 15.0;
                link(ui, "> sign the guestbook", x, y, "quack://guestbook", hex(0x1a2fd0));
                y += 15.0;
                link(ui, "> weather", x, y, "quack://weather", hex(0x1a2fd0));
                y += 22.0;
                // Under construction stripes.
                for k in 0..12 {
                    let c = if k % 2 == 0 { pal::YELLOW } else { pal::BLACK };
                    ui.rect(Rect::new(x + k as f32 * 8.0, y, 8.0, 8.0), c);
                }
                ui.text(&fonts.mono, "UNDER CONSTRUCTION", x + 102.0, y + 8.0, 12.0, pal::INK);
                y += 26.0;
                ui.text(&fonts.mono, "best viewed at 640x480", x, y, 12.0, pal::W95_DARK);
                // The secret: a single dot.
                link(ui, ".", page.x + page.w - 10.0, y, "quack://underground", hex(0xfdf3d0));
                max_scroll = (y + 20.0 - y0 - page.h).max(0.0);
            }
            "news" => {
                ui.rect(page, pal::WHITE);
                let x = page.x + 10.0;
                let mut y = y0 + 20.0;
                ui.text(&fonts.title, "TOWN NEWS", x, y, 16.0, pal::INK);
                y += 18.0;
                for line in [
                    "* PC BANG now open 24h. first hour free.",
                    "* strange bass heard from the old chapel",
                    "  every night. council 'looking into it'.",
                    "* flyers keep appearing on the hoardings.",
                    "  nobody admits to putting them up.",
                    "* campfire: still on fire.",
                ] {
                    ui.text(&fonts.mono, line, x, y, 12.0, pal::INK);
                    y += 13.0;
                }
                y += 8.0;
                link(ui, "< home", x, y, "quack://home", hex(0x1a2fd0));
                max_scroll = (y + 20.0 - y0 - page.h).max(0.0);
            }
            "weather" => {
                ui.rect(page, pal::SKY);
                let x = page.x + 10.0;
                let y = y0 + 24.0;
                ui.text(&fonts.title, "WEATHER", x, y, 16.0, pal::INK);
                ui.text(&fonts.mono, "fog. lasers. 100% chance of bass.", x, y + 20.0, 12.0, pal::INK);
                ui.text(&fonts.mono, "wind: 138 bpm from the north east.", x, y + 34.0, 12.0, pal::INK);
                link(ui, "< home", x, y + 56.0, "quack://home", hex(0x1a2fd0));
                max_scroll = 0.0;
            }
            "guestbook" => {
                ui.rect(page, hex(0xffe6f2));
                let x = page.x + 10.0;
                let mut y = y0 + 20.0;
                ui.text(&fonts.title, "GUESTBOOK", x, y, 16.0, pal::INK);
                y += 10.0;
                let f = Rect::new(x, y, page.w - 80.0, 16.0);
                let mut sent = false;
                if live && f.y > page.y && f.bottom() < page.bottom() {
                    sent = ui.field(fonts, "gb", f, &mut self.gb_text, 120, "write something nice");
                    let b = Rect::new(f.right() + 6.0, y, 50.0, 16.0);
                    bevel(ui, b, true);
                    ui.text(&fonts.body, "Sign!", b.x + 10.0, b.y + 11.5, 9.0, pal::BLACK);
                    if ui.hover(b) && ui.clicked {
                        ui.click_used = true;
                        sent = true;
                    }
                }
                if sent && !self.gb_text.trim().is_empty() {
                    acts.push(Action::Guestbook(std::mem::take(&mut self.gb_text)));
                    ui.focus = None;
                }
                y += 32.0;
                for (who, what) in ctx.guestbook {
                    ui.text(&fonts.mono, &format!("{who}:"), x, y, 12.0, pal::MAGENTA);
                    y += 12.0;
                    for chunk in wrap(what, 44) {
                        ui.text(&fonts.mono, &chunk, x + 8.0, y, 12.0, pal::INK);
                        y += 12.0;
                    }
                    y += 4.0;
                }
                link(ui, "< home", x, y + 6.0, "quack://home", hex(0x1a2fd0));
                max_scroll = (y + 26.0 - y0 - page.h).max(0.0);
            }
            "underground" => {
                ui.rect(page, pal::BLACK);
                let x = page.x + 10.0;
                let mut y = y0 + 22.0;
                let glitch = ((ui.time * 7.0).sin() > 0.92) as i32 as f32 * 2.0;
                ui.text(&fonts.title, "U N D E R G R O U N D", x + glitch, y, 16.0, pal::MAGENTA);
                ui.text(&fonts.title, "U N D E R G R O U N D", x - glitch, y, 16.0, with_alpha(pal::CYAN, 0.7));
                y += 14.0;
                ui.text(&fonts.mono, "the code on the flyer is the way in. tell no one.", x, y, 12.0, pal::ACID);
                y += 12.0;
                ui.text(&fonts.mono, "throw your own: RAVE.EXE on the desktop.", x, y, 12.0, pal::W95_GREY);
                y += 10.0;
                if ctx.flyers.is_empty() {
                    ui.text(&fonts.mono, "nothing on tonight. yet.", x, y + 20.0, 12.0, pal::W95_DARK);
                    y += 30.0;
                } else {
                    let tw = 88.0;
                    let th = tw * FLYER_H / FLYER_W;
                    let cols = ((page.w - 20.0) / (tw + 8.0)).floor().max(1.0) as usize;
                    for (i, (fl, tex)) in ctx.flyers.iter().enumerate() {
                        let cx = x + (i % cols) as f32 * (tw + 8.0);
                        let cy = y + (i / cols) as f32 * (th + 20.0);
                        let rr = Rect::new(cx, cy, tw, th);
                        draw_texture_ex(
                            tex,
                            rr.x * ui.s,
                            rr.y * ui.s,
                            WHITE,
                            DrawTextureParams { dest_size: Some(vec2(rr.w * ui.s, rr.h * ui.s)), ..Default::default() },
                        );
                        let hov = live && ui.hover(rr) && page.contains(ui.mouse);
                        if hov {
                            ui.set_hot();
                            ui.rect_lines(rr, 2.0, pal::ACID);
                            if ui.clicked {
                                ui.click_used = true;
                                acts.push(Action::OpenFlyer(i));
                            }
                        }
                        let status = if fl.live { "LIVE" } else { "" };
                        ui.text(&fonts.small, status, cx, cy + th + 9.0, 7.0, pal::MAGENTA);
                    }
                    let rows = ctx.flyers.len().div_ceil(cols);
                    y += rows as f32 * (th + 20.0);
                }
                link(ui, "< leave", x, y + 10.0, "quack://home", pal::W95_GREY);
                max_scroll = (y + 30.0 - y0 - page.h).max(0.0);
            }
            other => {
                ui.rect(page, pal::WHITE);
                let x = page.x + 10.0;
                ui.text(&fonts.title, "404", x, y0 + 26.0, 22.0, pal::INK);
                ui.text(&fonts.mono, &format!("quack://{other} could not be found."), x, y0 + 44.0, 12.0, pal::INK);
                ui.text(&fonts.mono, "maybe it was never there.", x, y0 + 57.0, 12.0, pal::W95_DARK);
                link(ui, "< home", x, y0 + 80.0, "quack://home", hex(0x1a2fd0));
                max_scroll = 0.0;
            }
        }
        self.scroll = self.scroll.min(max_scroll);
        if let Some(to) = nav {
            self.navigate(&to);
        }
    }

    // ----- RAVE.EXE -----

    fn rave_maker(&mut self, ui: &mut Ui, fonts: &Fonts, ctx: &Ctx, r: Rect, live: bool, acts: &mut Vec<Action>) {
        ui.rect(r, pal::W95_GREY);
        let x = r.x + 8.0;
        let mut y = r.y + 14.0;

        if self.rm_view > 0 {
            if let Some(info) = ctx.hosted.get(self.rm_view - 1) {
                // Result screen.
                ui.text(&fonts.body, "Your rave is booked. The code:", x, y, 9.0, pal::BLACK);
                y += 6.0;
                let cr = Rect::new(x, y, 150.0, 30.0);
                bevel(ui, cr, false);
                ui.rect(Rect::new(cr.x + 2.0, cr.y + 2.0, cr.w - 4.0, cr.h - 4.0), pal::BLACK);
                ui.text(&fonts.mono, &info.code, cr.x + 10.0, cr.y + 22.0, 24.0, pal::ACID);
                if let Some(tex) = ctx.hosted_flyers.get(self.rm_view - 1) {
                    let fw = 84.0;
                    let fr = Rect::new(r.x + r.w - fw - 10.0, r.y + 8.0, fw, fw * FLYER_H / FLYER_W);
                    draw_texture_ex(
                        tex,
                        fr.x * ui.s,
                        fr.y * ui.s,
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(fr.w * ui.s, fr.h * ui.s)), ..Default::default() },
                    );
                }
                y += 42.0;
                let copy = Rect::new(x, y, 70.0, 16.0);
                if w95_button(ui, fonts, copy, "Copy code", live) {
                    acts.push(Action::Copy(info.code.clone()));
                }
                let again = Rect::new(x + 76.0, y, 70.0, 16.0);
                if w95_button(ui, fonts, again, "Host another", live) {
                    self.rm_view = 0;
                }
                y += 30.0;
                let tips: Vec<String> = vec![
                    "1. Give the code to the goose".into(),
                    "   at the St. Bassilica door.".into(),
                    "2. Send it to friends, or let".into(),
                    if info.flyers { "   them find the flyers in town.".into() } else { "   them in on the secret.".into() },
                    if ctx.is_dj { "3. You're a DJ: step up to the".into() } else { "3. No DJ? The chapel plays its".into() },
                    if ctx.is_dj { "   decks to play live from Serato.".into() } else { "   own generated set all night.".into() },
                ];
                for t in tips {
                    ui.text(&fonts.mono, &t, x, y, 12.0, pal::BLACK);
                    y += 12.0;
                }
                return;
            }
            self.rm_view = 0;
        }

        ui.text(&fonts.body, "Event name", x, y, 9.0, pal::BLACK);
        y += 4.0;
        let tf = Rect::new(x, y, r.w - 16.0, 16.0);
        if live {
            ui.field(fonts, "rm_title", tf, &mut self.rm_title, 24, "ACID MASS");
        }
        y += 28.0;
        ui.text(&fonts.body, "Sound (when no DJ is live)", x, y, 9.0, pal::BLACK);
        y += 4.0;
        for (i, snd) in Sound::ALL.iter().enumerate() {
            let rr = Rect::new(x + i as f32 * 88.0, y, 84.0, 16.0);
            let on = self.rm_sound == *snd;
            bevel(ui, rr, !on);
            if on {
                ui.rect(Rect::new(rr.x + 2.0, rr.y + 2.0, rr.w - 4.0, rr.h - 4.0), pal::W95_NAVY);
            }
            ui.text(&fonts.small, snd.label(), rr.x + 4.0, rr.y + 11.0, 7.0, if on { pal::WHITE } else { pal::BLACK });
            if live && ui.hover(rr) {
                ui.set_hot();
                if ui.clicked {
                    ui.click_used = true;
                    self.rm_sound = *snd;
                    self.rm_bpm = snd.default_bpm();
                }
            }
        }
        y += 30.0;
        ui.text(&fonts.body, &format!("Tempo: {} BPM", self.rm_bpm.round()), x, y, 9.0, pal::BLACK);
        let minus = Rect::new(x + 110.0, y - 11.0, 16.0, 14.0);
        let plus = Rect::new(x + 130.0, y - 11.0, 16.0, 14.0);
        if w95_button(ui, fonts, minus, "-", live) {
            self.rm_bpm = (self.rm_bpm - 2.0).max(110.0);
        }
        if w95_button(ui, fonts, plus, "+", live) {
            self.rm_bpm = (self.rm_bpm + 2.0).min(160.0);
        }
        y += 16.0;
        ui.text(&fonts.body, "Doors", x, y, 9.0, pal::BLACK);
        for (i, (label, mins)) in [("Now", 0u32), ("15 min", 15), ("1 hour", 60)].iter().enumerate() {
            let rr = Rect::new(x + 40.0 + i as f32 * 52.0, y - 11.0, 48.0, 14.0);
            let on = self.rm_doors == *mins;
            bevel(ui, rr, !on);
            ui.text(&fonts.body, label, rr.x + 6.0, rr.y + 10.0, 8.0, pal::BLACK);
            if live && ui.hover(rr) {
                ui.set_hot();
                if ui.clicked {
                    ui.click_used = true;
                    self.rm_doors = *mins;
                }
            }
        }
        y += 14.0;
        let cb = Rect::new(x, y, 10.0, 10.0);
        bevel(ui, cb, false);
        ui.rect(Rect::new(cb.x + 1.0, cb.y + 1.0, 8.0, 8.0), pal::WHITE);
        if self.rm_flyers {
            ui.text(&fonts.small, "x", cb.x + 2.0, cb.y + 8.0, 8.0, pal::BLACK);
        }
        ui.text(&fonts.body, "Paste flyers around town", x + 14.0, y + 8.5, 9.0, pal::BLACK);
        let cbr = Rect::new(x, y, 140.0, 10.0);
        if live && ui.hover(cbr) {
            ui.set_hot();
            if ui.clicked {
                ui.click_used = true;
                self.rm_flyers = !self.rm_flyers;
            }
        }
        y += 22.0;
        let go = Rect::new(x, y, 120.0, 22.0);
        if w95_button(ui, fonts, go, "GENERATE CODE", live) {
            let title = if self.rm_title.trim().is_empty() { "ACID MASS".to_string() } else { self.rm_title.clone() };
            acts.push(Action::Host {
                title,
                sound: self.rm_sound,
                bpm: self.rm_bpm,
                flyers: self.rm_flyers,
                doors_in_min: self.rm_doors,
            });
        }
        if !ctx.hosted.is_empty() {
            ui.text(&fonts.body, "Your raves:", x + 130.0, y + 8.0, 8.0, pal::BLACK);
            for (i, h) in ctx.hosted.iter().enumerate() {
                let rr = Rect::new(x + 130.0, y + 11.0 + i as f32 * 11.0, 120.0, 10.0);
                let hov = live && ui.hover(rr);
                ui.text(&fonts.mono, &h.code, rr.x, rr.y + 9.0, 12.0, if hov { pal::MAGENTA } else { pal::W95_NAVY });
                if hov {
                    ui.set_hot();
                    if ui.clicked {
                        ui.click_used = true;
                        self.rm_view = i + 1;
                    }
                }
            }
        }
    }
}

fn readme(ui: &Ui, fonts: &Fonts, r: Rect, ctx: &Ctx) {
    ui.rect(r, pal::WHITE);
    let mut y = r.y + 12.0;
    let mut lines = vec![
        "MELODY QUACK - readme.txt",
        "",
        "move ......... WASD / arrows",
        "run .......... shift",
        "use .......... E or space",
        "chat ......... enter",
        "dance ........ 1 2 3 4 (in the chapel)",
        "",
        "RAVE.EXE books a rave and gives you",
        "a code. the bouncer at St. Bassilica",
        "lets you in with it.",
    ];
    if ctx.is_dj {
        lines.push("");
        lines.push("you're a DJ (purple dot). at the decks");
        lines.push("pick your Serato output and go live.");
    }
    for l in lines {
        ui.text(&fonts.mono, l, r.x + 6.0, y, 12.0, pal::BLACK);
        y += 12.0;
    }
}

pub fn bevel(ui: &Ui, r: Rect, raised: bool) {
    let (tl, br) = if raised { (pal::WHITE, pal::W95_DARK) } else { (pal::W95_DARK, pal::WHITE) };
    ui.rect(r, pal::BLACK);
    ui.rect(Rect::new(r.x, r.y, r.w - 1.0, r.h - 1.0), tl);
    ui.rect(Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, r.h - 2.0), br);
    ui.rect(Rect::new(r.x + 1.0, r.y + 1.0, r.w - 3.0, r.h - 3.0), if raised { pal::W95_GREY } else { pal::W95_GREY });
    if !raised {
        ui.rect(Rect::new(r.x + 1.0, r.y + 1.0, r.w - 2.0, 1.0), mix(pal::W95_DARK, pal::BLACK, 0.3));
    }
}

pub fn w95_button(ui: &mut Ui, fonts: &Fonts, r: Rect, label: &str, live: bool) -> bool {
    let hov = live && ui.hover(r);
    let pressed = hov && ui.down;
    bevel(ui, r, !pressed);
    let tw = ui.measure(&fonts.body, label, 9.0);
    let off = if pressed { 1.0 } else { 0.0 };
    ui.text(&fonts.body, label, r.x + (r.w - tw) / 2.0 + off, r.y + r.h / 2.0 + 3.5 + off, 9.0, pal::BLACK);
    if hov {
        ui.set_hot();
        if ui.clicked {
            ui.click_used = true;
            return true;
        }
    }
    false
}

pub fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        if line.len() + word.len() + 1 > width && !line.is_empty() {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}
