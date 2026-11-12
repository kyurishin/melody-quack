//! `MQ_AUTOPILOT=<dir>`: plays through every screen offline and saves a
//! screenshot of each, then quits. Used to check the art and UI without a
//! person at the keyboard (and handy for store screenshots).

use crate::game::{Game, Overlay, Screen};
use crate::world::SpotKind;
use macroquad::prelude::*;
use mq_shared::{ClientMsg, Scene};

pub struct Autopilot {
    dir: String,
    step: usize,
    wait: i32,
    pub walk: Vec2,
}

impl Autopilot {
    pub fn from_env() -> Option<Autopilot> {
        let dir = std::env::var("MQ_AUTOPILOT").ok()?;
        let _ = std::fs::create_dir_all(&dir);
        Some(Autopilot { dir, step: 0, wait: 40, walk: Vec2::ZERO })
    }
}

fn shot(g: &Game, name: &str) {
    let dir = &g.autopilot.as_ref().unwrap().dir;
    let mut img = get_screen_data();
    // The framebuffer's alpha isn't meaningful; make the PNG opaque.
    for px in img.bytes.chunks_exact_mut(4) {
        px[3] = 255;
    }
    img.export_png(&format!("{dir}/{name}.png"));
    println!("autopilot: {name}");
}

fn spot(g: &Game, k: SpotKind) -> Vec2 {
    g.map().spots.iter().find(|s| s.kind == k).map(|s| s.pos).unwrap_or(g.me.pos)
}

pub fn step(g: &mut Game) {
    let ap = g.autopilot.as_mut().unwrap();
    if ap.wait > 0 {
        ap.wait -= 1;
        return;
    }
    let n = ap.step;
    ap.step += 1;
    let mut wait = 20;
    match n {
        0 => {
            g.settings.name = "kyuri".into();
            g.settings.avatar = 0;
            g.show_dj_key = true;
            g.settings.dj_key = "offline".into();
            wait = 5;
        }
        1 => shot(g, "01-title"),
        2 => {
            // MQ_AUTOPILOT_SERVER=ws://... runs the tour against a real server.
            match std::env::var("MQ_AUTOPILOT_SERVER") {
                Ok(url) => {
                    g.settings.server = url;
                    g.settings.dj_key = std::env::var("MQ_AUTOPILOT_DJ_KEY").unwrap_or_default();
                    g.connect(false);
                }
                Err(_) => g.connect(true),
            }
            wait = 30;
        }
        3 => {
            assert!(g.screen == Screen::Playing, "didn't get in");
            g.me.pos = g.town.spawn + vec2(-40.0, 10.0);
            g.cam = g.me.pos;
            g.me.bubble = Some(("anyone seen the flyers?".into(), 9.0));
            wait = 30;
        }
        4 => shot(g, "02-town-plaza"),
        5 => {
            let p = spot(g, SpotKind::ChapelDoor) + vec2(-20.0, 30.0);
            g.me.pos = p;
            g.cam = p + vec2(0.0, -60.0);
            wait = 30;
        }
        6 => shot(g, "03-chapel-outside"),
        7 => {
            let p = spot(g, SpotKind::EnterPcBang) + vec2(0.0, 6.0);
            g.me.pos = p;
            g.cam = p + vec2(0.0, -40.0);
            wait = 30;
        }
        8 => shot(g, "04-pcbang-outside"),
        9 => {
            let p = g.pcbang.spawn;
            g.go(Scene::PcBang, p);
            g.me.pos = g.pcbang.seats[11] + vec2(0.0, 26.0);
            wait = 40;
        }
        10 => shot(g, "05-pcbang-inside"),
        11 => {
            g.me.pos = g.pcbang.seats[7];
            g.me.seat = Some(7);
            g.computer.power_on();
            g.overlay = Overlay::Computer;
            wait = 110;
        }
        12 => shot(g, "06-desktop"),
        13 => {
            g.computer.open(crate::computer::App::Readme);
            g.computer.navigate("quack://home");
            wait = 20;
        }
        14 => shot(g, "07-browser"),
        15 => {
            g.computer.open(crate::computer::App::RaveMaker);
            wait = 10;
        }
        16 => shot(g, "08-ravemaker-form"),
        17 => {
            g.send(ClientMsg::HostRave {
                title: "Acid Mass".into(),
                sound: mq_shared::Sound::Gothic,
                bpm: 132.0,
                flyers: true,
                doors_in_min: 0,
            });
            g.send(ClientMsg::HostRave {
                title: "Bass Vespers".into(),
                sound: mq_shared::Sound::Acid,
                bpm: 138.0,
                flyers: true,
                doors_in_min: 15,
            });
            wait = 30;
        }
        18 => shot(g, "09-ravemaker-code"),
        19 => {
            g.computer.navigate("quack://underground");
            wait = 20;
        }
        20 => shot(g, "10-underground"),
        21 => {
            g.stand_up();
            let door = g.town.spots.iter().find(|s| s.kind == SpotKind::EnterPcBang).unwrap().pos;
            g.go(Scene::Town, door);
            let slot = g.wall.iter().position(|w| w.as_ref().map(|(f, _)| !f.expired).unwrap_or(false)).unwrap_or(2);
            let p = g.town.spots.iter().find(|s| s.kind == SpotKind::Poster(slot)).map(|s| s.pos).unwrap();
            g.me.pos = p + vec2(0.0, 6.0);
            g.cam = g.me.pos + vec2(0.0, -30.0);
            wait = 40;
        }
        22 => shot(g, "11-town-flyers"),
        23 => {
            let slot = g.wall.iter().position(|w| w.as_ref().map(|(f, _)| !f.expired).unwrap_or(false)).unwrap_or(0);
            if let Some(Some((f, t))) = g.wall.get(slot).cloned() {
                g.learn_code(&f.code);
                g.overlay = Overlay::Poster { flyer: f, tex: t };
            }
            wait = 10;
        }
        24 => shot(g, "12-flyer"),
        25 => {
            let p = spot(g, SpotKind::ChapelDoor);
            g.me.pos = p + vec2(0.0, 4.0);
            g.overlay = Overlay::Door { code: "acd-efg".into(), error: None, waiting: false };
            wait = 10;
        }
        26 => shot(g, "13-door"),
        27 => {
            let code = g.hosted[0].code.clone();
            g.try_code(&code);
            wait = 40;
        }
        28 => {
            // Skip ahead to the drop for the screenshot.
            let now = g.server_ms();
            if let Some(r) = g.rave.as_mut() {
                let ms_per_bar = 4.0 * 60_000.0 / r.bpm as f64;
                r.start_ms = (now - 33.0 * ms_per_bar) as u64;
            }
            g.me.pos = g.chapel.spawn + vec2(-40.0, -250.0);
            g.me.dance = 1;
            g.me.bubble = Some(("this drop!!".into(), 9.0));
            g.cam = g.me.pos + vec2(0.0, -40.0);
            wait = 45;
        }
        29 => shot(g, "14-rave"),
        30 => {
            g.me.pos = spot(g, SpotKind::Decks);
            g.inputs = crate::audio::Audio::input_devices();
            g.overlay = Overlay::Decks;
            wait = 10;
        }
        31 => shot(g, "15-decks"),
        32 => {
            g.overlay = Overlay::Pause;
            wait = 5;
        }
        33 => shot(g, "16-pause"),
        _ => std::process::exit(0),
    }
    g.autopilot.as_mut().unwrap().wait = wait;
}
