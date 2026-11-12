//! Game state, input, networking glue and scene flow.

use crate::audio::{Audio, Capture, MusicCue, Sfx};
use crate::computer::{self, Computer};
use crate::gfx::{additive_material, glow_texture, Fonts};
use crate::net::{wall_ms, Net, Status};
use crate::poster::{self, Flyer};
use crate::settings::Settings;
use crate::sprites::{Sprites, CRITTERS};
use crate::ui::{Tone, Ui};
use crate::world::{self, Map, SpotKind};
use macroquad::prelude::*;
use mq_shared::{hash_str, normalize_code, ClientMsg, PlayerState, RaveInfo, Rng, Scene, ServerMsg, Sound};
use std::collections::{HashMap, VecDeque};

pub const WALK: f32 = 92.0;
pub const RUN: f32 = 150.0;

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Title,
    Connecting,
    Playing,
}

pub enum Overlay {
    None,
    Computer,
    Poster { flyer: Flyer, tex: Texture2D },
    Door { code: String, error: Option<String>, waiting: bool },
    Dialog { who: &'static str, lines: Vec<String> },
    Decks,
    Pause,
}

pub struct Remote {
    pub st: PlayerState,
    pub pos: Vec2,
    pub anim: f32,
    pub bubble: Option<(String, f32)>,
}

pub struct Me {
    pub pos: Vec2,
    pub facing: i8,
    pub moving: bool,
    pub dance: u8,
    pub seat: Option<usize>,
    pub anim: f32,
    pub bubble: Option<(String, f32)>,
}

pub struct ChatLine {
    pub name: String,
    pub dj: bool,
    pub text: String,
    pub age: f32,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max: f32,
    pub color: Color,
    pub size: f32,
}

pub struct Ghost {
    pub pos: Vec2,
    pub kind: usize,
    pub phase: f32,
    pub style: u8,
}

pub struct Game {
    pub screen: Screen,
    pub fonts: Fonts,
    pub sprites: Sprites,
    pub glow: Texture2D,
    pub add: Material,
    pub ui: Ui,
    pub audio: Audio,
    pub net: Option<Net>,
    pub settings: Settings,
    pub title_error: Option<String>,
    pub show_dj_key: bool,

    pub my_id: u32,
    pub my_name: String,
    pub is_dj: bool,
    pub scene: Scene,
    pub town: Map,
    pub pcbang: Map,
    pub chapel: Map,
    pub me: Me,
    pub remotes: HashMap<u32, Remote>,
    pub cam: Vec2,
    pub fade: f32,
    pub time: f32,

    pub public_raves: Vec<RaveInfo>,
    pub flyer_cache: HashMap<String, (Flyer, Texture2D)>,
    pub decoys: Vec<Flyer>,
    pub wall: Vec<Option<(Flyer, Texture2D)>>,
    pub underground: Vec<(Flyer, Texture2D)>,
    pub guestbook: Vec<(String, String)>,
    pub hosted: Vec<RaveInfo>,
    pub hosted_tex: Vec<Texture2D>,
    pub known_codes: Vec<String>,
    pub rave: Option<RaveInfo>,

    pub computer: Computer,
    pub overlay: Overlay,
    pub chat_open: bool,
    pub chat_text: String,
    pub chat: VecDeque<ChatLine>,
    pub send_timer: f32,
    pub capture: Option<Capture>,
    pub inputs: Vec<String>,
    pub input_sel: usize,
    pub particles: Vec<Particle>,
    pub ghosts: Vec<Ghost>,
    pub autopilot: Option<crate::autopilot::Autopilot>,
}

impl Game {
    pub fn new() -> Game {
        let town = world::town();
        let spawn = town.spawn;
        let mut audio = Audio::new();
        let settings = Settings::load();
        audio.set(|c| c.master = settings.volume);
        let _ = &mut audio;
        let decoys = Flyer::decoys(0xDEC0, 10);
        let chapel = world::chapel();
        let ghosts = make_ghosts(&chapel);
        Game {
            screen: Screen::Title,
            fonts: Fonts::load(),
            sprites: Sprites::build(),
            glow: glow_texture(),
            add: additive_material(),
            ui: Ui::new(),
            audio,
            net: None,
            show_dj_key: !settings.dj_key.is_empty(),
            settings,
            title_error: None,
            my_id: 0,
            my_name: String::new(),
            is_dj: false,
            scene: Scene::Town,
            town,
            pcbang: world::pcbang(),
            chapel,
            me: Me { pos: spawn, facing: 1, moving: false, dance: 0, seat: None, anim: 0.0, bubble: None },
            remotes: HashMap::new(),
            cam: spawn,
            fade: 1.0,
            time: 0.0,
            public_raves: Vec::new(),
            flyer_cache: HashMap::new(),
            decoys,
            wall: Vec::new(),
            underground: Vec::new(),
            guestbook: Vec::new(),
            hosted: Vec::new(),
            hosted_tex: Vec::new(),
            known_codes: Vec::new(),
            rave: None,
            computer: Computer::new(),
            overlay: Overlay::None,
            chat_open: false,
            chat_text: String::new(),
            chat: VecDeque::new(),
            send_timer: 0.0,
            capture: None,
            inputs: Vec::new(),
            input_sel: 0,
            particles: Vec::new(),
            ghosts,
            autopilot: crate::autopilot::Autopilot::from_env(),
        }
    }

    pub fn map(&self) -> &Map {
        match self.scene {
            Scene::Town => &self.town,
            Scene::PcBang => &self.pcbang,
            Scene::Rave => &self.chapel,
        }
    }

    pub fn server_ms(&self) -> f64 {
        self.net.as_ref().map(|n| n.server_ms()).unwrap_or_else(wall_ms)
    }

    pub fn sfx(&self, s: Sfx) {
        self.audio.play(s);
    }

    pub fn send(&mut self, m: ClientMsg) {
        if let Some(n) = self.net.as_mut() {
            n.send(m);
        }
    }

    // ---------------------------------------------------------------------
    // Connecting
    // ---------------------------------------------------------------------

    pub fn connect(&mut self, offline: bool) {
        self.settings.save();
        let mut net = if offline { Net::offline() } else { Net::online(&self.settings.server) };
        net.audio_in = Some(self.audio.packet_sink());
        let key = if self.show_dj_key { self.settings.dj_key.clone() } else { String::new() };
        net.hello(&self.settings.name, self.settings.avatar, &key);
        self.net = Some(net);
        self.screen = Screen::Connecting;
        self.title_error = None;
        self.sfx(Sfx::Select);
    }

    pub fn disconnect(&mut self, why: Option<String>) {
        self.stop_live();
        self.net = None;
        self.screen = Screen::Title;
        self.overlay = Overlay::None;
        self.remotes.clear();
        self.rave = None;
        self.scene = Scene::Town;
        self.title_error = why;
        self.audio.set(|c| {
            c.music = None;
            c.live = false;
        });
    }

    // ---------------------------------------------------------------------
    // Messages
    // ---------------------------------------------------------------------

    pub fn pump(&mut self) {
        let Some(net) = self.net.as_mut() else { return };
        let msgs = net.poll();
        let status = net.status.clone();
        if let Status::Lost(why) = status {
            let why = if self.screen == Screen::Playing { format!("disconnected: {why}") } else { why };
            self.disconnect(Some(why));
            self.ui.toast_y = 12.0;
            return;
        }
        for m in msgs {
            self.on_msg(m);
        }
    }

    fn on_msg(&mut self, m: ServerMsg) {
        match m {
            ServerMsg::Welcome { id, name, dj, motd, .. } => {
                self.my_id = id;
                self.my_name = name;
                self.is_dj = dj;
                self.screen = Screen::Playing;
                self.scene = Scene::Town;
                self.me.pos = self.town.spawn;
                self.cam = self.me.pos;
                self.fade = 1.0;
                self.send(ClientMsg::Enter { scene: Scene::Town, code: None });
                self.ui.toast(motd, Tone::Cream);
                if dj {
                    self.ui.toast("you're on the list. dj mode on.", Tone::Dj);
                }
                self.sfx(Sfx::Quack);
            }
            ServerMsg::Pong { .. } => {}
            ServerMsg::Snapshot { players } => {
                let mut seen = Vec::new();
                for p in players {
                    if p.id == self.my_id {
                        continue;
                    }
                    seen.push(p.id);
                    let target = vec2(p.x, p.y);
                    let r = self.remotes.entry(p.id).or_insert(Remote { st: p.clone(), pos: target, anim: 0.0, bubble: None });
                    r.st = p;
                }
                self.remotes.retain(|id, _| seen.contains(id));
            }
            ServerMsg::Left { id } => {
                self.remotes.remove(&id);
            }
            ServerMsg::Chat { id, name, dj, text } => {
                if id == self.my_id {
                    self.me.bubble = Some((text.clone(), 5.0));
                } else if let Some(r) = self.remotes.get_mut(&id) {
                    r.bubble = Some((text.clone(), 5.0));
                }
                self.chat.push_back(ChatLine { name, dj, text, age: 0.0 });
                while self.chat.len() > 30 {
                    self.chat.pop_front();
                }
                self.sfx(Sfx::Blip);
            }
            ServerMsg::Flyers { raves } => {
                self.public_raves = raves;
                self.refresh_flyers();
            }
            ServerMsg::RaveHosted { rave } => {
                let f = Flyer::from_rave(&rave, self.server_ms());
                let tex = poster::render(&f, &self.fonts);
                self.hosted.push(rave.clone());
                self.hosted_tex.push(tex);
                self.learn_code(&rave.code);
                self.computer.show_hosted(self.hosted.len() - 1);
                miniquad::window::clipboard_set(&rave.code);
                self.ui.toast(format!("rave booked. {} copied to clipboard", rave.code), Tone::Green);
                self.sfx(Sfx::Chime);
            }
            ServerMsg::Entered { scene, rave } => {
                if scene == Scene::Rave {
                    if let Some(r) = rave {
                        self.enter_rave(r);
                    }
                }
            }
            ServerMsg::RaveUpdate { rave } => {
                if self.rave.as_ref().map(|r| r.code == rave.code).unwrap_or(false) {
                    let was_live = self.rave.as_ref().map(|r| r.live).unwrap_or(false);
                    if rave.live && !was_live {
                        let who = if rave.host_is_dj { format!("DJ {}", rave.host_name) } else { rave.host_name.clone() };
                        self.ui.toast(format!("{who} is live on the decks"), Tone::Dj);
                    } else if !rave.live && was_live {
                        self.ui.toast("live set over. the chapel takes it from here.", Tone::Ink);
                        self.audio.clear_live();
                    }
                    self.rave = Some(rave);
                }
            }
            ServerMsg::Denied { reason } => {
                if let Overlay::Door { error, waiting, .. } = &mut self.overlay {
                    *error = Some(reason);
                    *waiting = false;
                    self.sfx(Sfx::Error);
                } else if self.screen != Screen::Playing {
                    self.disconnect(Some(reason));
                } else {
                    self.ui.toast(reason, Tone::Coral);
                    self.sfx(Sfx::Error);
                }
            }
            ServerMsg::Guestbook { entries } => {
                self.guestbook = entries;
            }
            ServerMsg::Notice { text } => {
                self.ui.toast(text, Tone::Yellow);
            }
        }
    }

    pub fn learn_code(&mut self, code: &str) {
        if !self.known_codes.iter().any(|c| c == code) {
            self.known_codes.insert(0, code.to_string());
            self.known_codes.truncate(6);
        }
    }

    /// Rebuild flyer textures and re-paste the town's walls.
    fn refresh_flyers(&mut self) {
        let now = self.server_ms();
        let mut live_codes = Vec::new();
        for r in &self.public_raves {
            let f = Flyer::from_rave(r, now);
            live_codes.push(r.code.clone());
            let stale = self.flyer_cache.get(&r.code).map(|(old, _)| old != &f).unwrap_or(true);
            if stale {
                let tex = poster::render(&f, &self.fonts);
                self.flyer_cache.insert(r.code.clone(), (f, tex));
            }
        }
        for d in &self.decoys {
            if !self.flyer_cache.contains_key(&d.code) {
                let tex = poster::render(d, &self.fonts);
                self.flyer_cache.insert(d.code.clone(), (d.clone(), tex));
            }
        }
        let decoy_codes: Vec<String> = self.decoys.iter().map(|d| d.code.clone()).collect();
        self.flyer_cache.retain(|k, _| live_codes.contains(k) || decoy_codes.contains(k));
        self.underground = live_codes.iter().filter_map(|c| self.flyer_cache.get(c).cloned()).collect();

        // Walls: every live flyer at least once (more if there's room),
        // old torn ones fill the gaps, shuffled by the current line-up.
        let slots = self.town.posters.len();
        let mut picks: Vec<String> = Vec::new();
        if !live_codes.is_empty() {
            let mut i = 0;
            while picks.len() < slots.min(live_codes.len() * 2) {
                picks.push(live_codes[i % live_codes.len()].clone());
                i += 1;
            }
        }
        let mut k = 0;
        while picks.len() < slots {
            picks.push(decoy_codes[k % decoy_codes.len()].clone());
            k += 1;
        }
        let mut rng = Rng::new(hash_str(&live_codes.join(",")));
        for i in (1..picks.len()).rev() {
            let j = rng.below(i + 1);
            picks.swap(i, j);
        }
        self.wall = picks.iter().map(|c| self.flyer_cache.get(c).cloned()).collect();
    }

    // ---------------------------------------------------------------------
    // Scenes
    // ---------------------------------------------------------------------

    pub fn go(&mut self, scene: Scene, pos: Vec2) {
        if self.scene == Scene::Rave && scene != Scene::Rave {
            self.stop_live();
            self.rave = None;
        }
        self.scene = scene;
        self.me.pos = pos;
        self.me.seat = None;
        self.me.dance = 0;
        self.cam = pos;
        self.fade = 1.0;
        self.remotes.clear();
        self.particles.clear();
        self.overlay = Overlay::None;
        if scene != Scene::Rave {
            self.send(ClientMsg::Enter { scene, code: None });
        }
        self.sfx(Sfx::Door);
    }

    fn enter_rave(&mut self, r: RaveInfo) {
        self.learn_code(&r.code);
        let spawn = self.chapel.spawn;
        self.scene = Scene::Rave;
        self.me.pos = spawn;
        self.me.seat = None;
        self.cam = spawn;
        self.fade = 1.0;
        self.remotes.clear();
        self.particles.clear();
        self.overlay = Overlay::None;
        self.ui.toast(format!("welcome to {}", r.title), Tone::Pink);
        self.rave = Some(r);
        self.audio.clear_live();
        self.sfx(Sfx::Door);
    }

    pub fn stop_live(&mut self) {
        if self.capture.take().is_some() {
            self.send(ClientMsg::SetLive { live: false });
        }
    }

    pub fn i_am_host(&self) -> bool {
        self.rave.as_ref().map(|r| self.hosted.iter().any(|h| h.code == r.code)).unwrap_or(false)
    }

    // ---------------------------------------------------------------------
    // Per-frame update
    // ---------------------------------------------------------------------

    pub fn update(&mut self) {
        let dt = get_frame_time().min(0.05);
        self.time += dt;
        self.fade = (self.fade - dt * 3.0).max(0.0);
        self.pump();
        if self.screen != Screen::Playing {
            self.update_audio();
            return;
        }

        let overlay_open = !matches!(self.overlay, Overlay::None);
        let typing = self.ui.typing() || self.chat_open;

        // Chat.
        if !overlay_open && !self.chat_open && self.ui.enter {
            self.chat_open = true;
            self.ui.focus = Some("chat".into());
            self.ui.enter = false;
        }

        // Escape.
        if self.ui.escape {
            if self.chat_open {
                self.chat_open = false;
                self.chat_text.clear();
                self.ui.focus = None;
            } else if self.ui.focus.is_some() {
                self.ui.focus = None;
            } else if matches!(self.overlay, Overlay::Computer) {
                self.stand_up();
            } else if overlay_open {
                self.overlay = Overlay::None;
                self.sfx(Sfx::Back);
            } else {
                self.overlay = Overlay::Pause;
                self.sfx(Sfx::Select);
            }
        }

        // Movement.
        let mut dir = Vec2::ZERO;
        if !overlay_open && !typing && self.me.seat.is_none() {
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                dir.x -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                dir.x += 1.0;
            }
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                dir.y -= 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                dir.y += 1.0;
            }
            if let Some(ap) = self.autopilot.as_ref() {
                dir = ap.walk;
            }
        }
        self.me.moving = dir != Vec2::ZERO;
        if self.me.moving {
            self.me.dance = 0;
            let speed = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) { RUN } else { WALK };
            let v = dir.normalize() * speed * dt;
            if dir.x != 0.0 {
                self.me.facing = dir.x.signum() as i8;
            }
            let map = self.map();
            let mut p = self.me.pos;
            if map.free(vec2(p.x + v.x, p.y)) {
                p.x += v.x;
            }
            if map.free(vec2(p.x, p.y + v.y)) {
                p.y += v.y;
            }
            self.me.pos = p;
            self.me.anim += dt;
        } else {
            self.me.anim = 0.0;
        }

        // Dancing.
        if self.scene == Scene::Rave && !overlay_open && !typing {
            for (k, d) in [(KeyCode::Key1, 1), (KeyCode::Key2, 2), (KeyCode::Key3, 3), (KeyCode::Key4, 4)] {
                if is_key_pressed(k) {
                    self.me.dance = if self.me.dance == d { 0 } else { d };
                }
            }
        }

        // Interact.
        if !overlay_open && !typing && (is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Space)) {
            if let Some(kind) = self.nearest_spot() {
                self.interact(kind);
            }
        }

        // Remotes.
        for r in self.remotes.values_mut() {
            let target = vec2(r.st.x, r.st.y);
            if r.pos.distance(target) > 160.0 {
                r.pos = target;
            } else {
                r.pos += (target - r.pos) * (dt * 12.0).min(1.0);
            }
            if r.st.moving {
                r.anim += dt;
            } else {
                r.anim = 0.0;
            }
            if let Some((_, t)) = r.bubble.as_mut() {
                *t -= dt;
            }
            if r.bubble.as_ref().map(|b| b.1 <= 0.0).unwrap_or(false) {
                r.bubble = None;
            }
        }
        if let Some((_, t)) = self.me.bubble.as_mut() {
            *t -= dt;
        }
        if self.me.bubble.as_ref().map(|b| b.1 <= 0.0).unwrap_or(false) {
            self.me.bubble = None;
        }
        for c in self.chat.iter_mut() {
            c.age += dt;
        }

        // Send state at ~12 Hz.
        self.send_timer -= dt;
        if self.send_timer <= 0.0 {
            self.send_timer = 0.08;
            let (pos, seated) = match self.me.seat {
                Some(i) => (self.pcbang.seats[i], true),
                None => (self.me.pos, false),
            };
            let m = ClientMsg::State {
                x: pos.x,
                y: pos.y,
                facing: self.me.facing,
                moving: self.me.moving,
                dance: self.me.dance,
                seated,
            };
            self.send(m);
        }

        // Camera.
        let target = self.me.pos + vec2(0.0, -16.0);
        self.cam += (target - self.cam) * (dt * 8.0).min(1.0);

        self.update_particles(dt);
        self.update_audio();

        // Re-render flyers once a minute so "doors soon" turns to "doors open".
        if (self.time % 60.0) < dt && !self.public_raves.is_empty() {
            self.refresh_flyers();
        }
    }

    pub fn nearest_spot(&self) -> Option<SpotKind> {
        let map = self.map();
        let mut best: Option<(f32, SpotKind)> = None;
        for s in &map.spots {
            let d = s.pos.distance(self.me.pos);
            if d < s.radius && best.map(|b| d < b.0).unwrap_or(true) {
                if let SpotKind::Seat(i) = s.kind {
                    if self.seat_taken(i) {
                        continue;
                    }
                }
                best = Some((d, s.kind));
            }
        }
        best.map(|b| b.1)
    }

    pub fn seat_taken(&self, i: usize) -> bool {
        let p = self.pcbang.seats[i];
        self.remotes.values().any(|r| r.st.seated && vec2(r.st.x, r.st.y).distance(p) < 4.0)
    }

    pub fn spot_label(&self, k: SpotKind) -> String {
        match k {
            SpotKind::EnterPcBang => "Enter the PC bang".into(),
            SpotKind::ExitPcBang => "Leave".into(),
            SpotKind::Seat(_) => "Use computer".into(),
            SpotKind::Owner => "Talk to Mr. Kim".into(),
            SpotKind::ChapelDoor => "Talk to the bouncer".into(),
            SpotKind::ExitChapel => "Leave the rave".into(),
            SpotKind::Decks => {
                if self.i_am_host() && self.is_dj {
                    "DJ decks".into()
                } else {
                    "Look at the decks".into()
                }
            }
            SpotKind::Poster(_) => "Read flyer".into(),
            SpotKind::Campfire => "Warm up".into(),
            SpotKind::Cottage => "Knock".into(),
        }
    }

    fn interact(&mut self, k: SpotKind) {
        self.sfx(Sfx::Select);
        match k {
            SpotKind::EnterPcBang => {
                let p = self.pcbang.spawn;
                self.go(Scene::PcBang, p);
            }
            SpotKind::ExitPcBang => {
                let door = self
                    .town
                    .spots
                    .iter()
                    .find(|s| s.kind == SpotKind::EnterPcBang)
                    .map(|s| s.pos)
                    .unwrap_or(self.town.spawn);
                self.go(Scene::Town, door + vec2(0.0, 22.0));
            }
            SpotKind::Seat(i) => {
                self.me.seat = Some(i);
                self.me.pos = self.pcbang.seats[i];
                self.computer.power_on();
                self.overlay = Overlay::Computer;
                self.sfx(Sfx::Chime);
            }
            SpotKind::Owner => {
                let lines = if self.hosted.is_empty() {
                    vec![
                        "first hour's free. the snacks are not.".to_string(),
                        "the kids use RAVE.EXE on those machines.".to_string(),
                        "they come out with codes and go to the old chapel.".to_string(),
                        "i don't ask.".to_string(),
                    ]
                } else {
                    vec!["you again. busy night?".to_string(), "don't spill ramyeon on the keyboards.".to_string()]
                };
                self.overlay = Overlay::Dialog { who: "MR. KIM", lines };
            }
            SpotKind::ChapelDoor => {
                self.overlay = Overlay::Door { code: String::new(), error: None, waiting: false };
                self.ui.focus = Some("door".into());
            }
            SpotKind::ExitChapel => {
                let door = self
                    .town
                    .spots
                    .iter()
                    .find(|s| s.kind == SpotKind::ChapelDoor)
                    .map(|s| s.pos)
                    .unwrap_or(self.town.spawn);
                self.go(Scene::Town, door + vec2(-30.0, 24.0));
            }
            SpotKind::Decks => {
                if self.i_am_host() && self.is_dj {
                    self.inputs = Audio::input_devices();
                    // Prefer a loopback device if there is one.
                    if let Some(i) = self.inputs.iter().position(|d| {
                        let d = d.to_lowercase();
                        d.contains("blackhole") || d.contains("loopback") || d.contains("soundflower") || d.contains("serato")
                    }) {
                        self.input_sel = i;
                    }
                    self.overlay = Overlay::Decks;
                } else if self.i_am_host() {
                    self.overlay = Overlay::Dialog {
                        who: "THE DECKS",
                        lines: vec![
                            "it's your night, but you're not on the dj list.".into(),
                            "the chapel's sound system plays a generated set instead.".into(),
                            "ask the server owner for a dj key to play live.".into(),
                        ],
                    };
                } else {
                    let host = self.rave.as_ref().map(|r| r.host_name.clone()).unwrap_or_default();
                    self.overlay = Overlay::Dialog {
                        who: "THE DECKS",
                        lines: vec![format!("these belong to {host} tonight."), "no touching.".into()],
                    };
                }
            }
            SpotKind::Poster(slot) => {
                if let Some(Some((f, tex))) = self.wall.get(slot).cloned() {
                    self.learn_code(&f.code);
                    self.overlay = Overlay::Poster { flyer: f, tex };
                }
            }
            SpotKind::Campfire => {
                self.overlay = Overlay::Dialog {
                    who: "CAMPFIRE",
                    lines: vec![
                        "the fire crackles.".into(),
                        "somewhere north east, a kick drum answers.".into(),
                    ],
                };
                self.sfx(Sfx::Quack);
            }
            SpotKind::Cottage => {
                self.overlay = Overlay::Dialog {
                    who: "COTTAGE",
                    lines: vec!["nobody's home.".into(), "a note on the door: 'gone raving. back at dawn.'".into()],
                };
            }
        }
    }

    pub fn stand_up(&mut self) {
        if let Some(i) = self.me.seat.take() {
            self.me.pos = self.pcbang.seats[i] + vec2(0.0, 10.0);
        }
        self.overlay = Overlay::None;
        self.ui.focus = None;
        self.sfx(Sfx::Back);
    }

    pub fn try_code(&mut self, raw: &str) {
        let Some(code) = normalize_code(raw) else {
            if let Overlay::Door { error, .. } = &mut self.overlay {
                *error = Some("nine letters and numbers, like QK7-MAX-9PE.".into());
            }
            self.sfx(Sfx::Error);
            return;
        };
        if self.decoys.iter().any(|d| d.code == code) {
            if let Overlay::Door { error, .. } = &mut self.overlay {
                *error = Some("that flyer's from last summer, mate. that night's long over.".into());
            }
            self.sfx(Sfx::Error);
            return;
        }
        if let Overlay::Door { waiting, error, .. } = &mut self.overlay {
            *waiting = true;
            *error = None;
        }
        self.send(ClientMsg::Enter { scene: Scene::Rave, code: Some(code) });
    }

    pub fn go_live(&mut self) {
        let Some(net) = self.net.as_ref() else { return };
        let uplink = net.uplink();
        let dev = self.inputs.get(self.input_sel).cloned();
        match self.audio.start_capture(dev.as_deref(), uplink) {
            Ok(c) => {
                self.ui.toast(format!("live from {}", c.device), Tone::Dj);
                self.capture = Some(c);
                self.send(ClientMsg::SetLive { live: true });
                self.sfx(Sfx::Select);
            }
            Err(e) => {
                self.ui.toast(format!("couldn't open input: {e}"), Tone::Coral);
                self.sfx(Sfx::Error);
            }
        }
    }

    pub fn computer_actions(&mut self, acts: Vec<computer::Action>) {
        for a in acts {
            match a {
                computer::Action::ShutDown => self.stand_up(),
                computer::Action::Host { title, sound, bpm, flyers, doors_in_min } => {
                    self.send(ClientMsg::HostRave { title, sound, bpm, flyers, doors_in_min });
                }
                computer::Action::Guestbook(text) => self.send(ClientMsg::GuestbookPost { text }),
                computer::Action::Copy(code) => {
                    miniquad::window::clipboard_set(&code);
                    self.ui.toast(format!("{code} copied"), Tone::Green);
                }
                computer::Action::OpenFlyer(i) => {
                    if let Some((f, tex)) = self.underground.get(i).cloned() {
                        self.learn_code(&f.code);
                        self.overlay = Overlay::Poster { flyer: f, tex };
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // Audio
    // ---------------------------------------------------------------------

    /// The generated set the chapel is playing right now (for the town's
    /// muffled bleed, the title screen and the rave itself).
    pub fn cue(&self) -> Option<MusicCue> {
        if let Some(r) = &self.rave {
            return Some(MusicCue { seed: r.seed, sound: r.sound, bpm: r.bpm, start_ms: r.start_ms });
        }
        if let Some(r) = self.public_raves.first() {
            return Some(MusicCue { seed: r.seed, sound: r.sound, bpm: r.bpm, start_ms: r.start_ms });
        }
        // The chapel's house set: always on, aligned to the hour.
        Some(MusicCue { seed: 0x5EED, sound: Sound::Gothic, bpm: 128.0, start_ms: 0 })
    }

    fn update_audio(&mut self) {
        let offset = self.net.as_ref().map(|n| n.clock_offset).unwrap_or(0.0);
        let cue = self.cue();
        let (gain, muffle, live) = match (self.screen, self.scene) {
            (Screen::Playing, Scene::Town) => {
                let door = self
                    .town
                    .spots
                    .iter()
                    .find(|s| s.kind == SpotKind::ChapelDoor)
                    .map(|s| s.pos)
                    .unwrap_or_default();
                let d = self.me.pos.distance(door);
                (((1.0 - d / 700.0).clamp(0.0, 1.0)).powf(1.6) * 0.8, 0.88, false)
            }
            (Screen::Playing, Scene::PcBang) => (0.0, 0.9, false),
            (Screen::Playing, Scene::Rave) => {
                let live = self.rave.as_ref().map(|r| r.live).unwrap_or(false);
                let me_live = self.capture.is_some();
                (if me_live { 0.0 } else { 1.0 }, 0.0, live && !me_live)
            }
            _ => (0.35, 0.75, false),
        };
        let gain = if matches!(self.overlay, Overlay::Computer) { gain * 0.4 } else { gain };
        let master = self.settings.volume;
        self.audio.set(|c| {
            c.music = cue;
            c.clock_offset = offset;
            c.music_gain = gain;
            c.muffle = muffle;
            c.live = live;
            c.master = master;
        });
    }

    fn update_particles(&mut self, dt: f32) {
        let mut rng = Rng::new((self.time * 1000.0) as u64);
        match self.scene {
            Scene::Town => {
                if let Some(f) = self.town.campfire {
                    if rng.chance(dt * 18.0) {
                        self.particles.push(Particle {
                            pos: f + vec2(rng.range(-6.0, 6.0), -6.0),
                            vel: vec2(rng.range(-8.0, 8.0), rng.range(-40.0, -20.0)),
                            life: 0.0,
                            max: rng.range(0.6, 1.4),
                            color: if rng.chance(0.5) { crate::gfx::pal::YELLOW } else { crate::gfx::pal::CORAL },
                            size: 1.0,
                        });
                    }
                }
            }
            Scene::Rave => {
                let size = self.chapel.px_size();
                if self.particles.len() < 40 && rng.chance(dt * 6.0) {
                    self.particles.push(Particle {
                        pos: vec2(rng.range(0.0, size.x), rng.range(200.0, size.y)),
                        vel: vec2(rng.range(-6.0, 6.0), rng.range(-4.0, 2.0)),
                        life: 0.0,
                        max: rng.range(6.0, 12.0),
                        color: crate::gfx::pal::DUSK2,
                        size: rng.range(40.0, 90.0),
                    });
                }
            }
            Scene::PcBang => {}
        }
        for p in self.particles.iter_mut() {
            p.pos += p.vel * dt;
            p.life += dt;
        }
        self.particles.retain(|p| p.life < p.max);
    }
}

fn make_ghosts(chapel: &Map) -> Vec<Ghost> {
    let mut r = Rng::new(99);
    let mut v = Vec::new();
    let size = chapel.px_size();
    while v.len() < 26 {
        let p = vec2(r.range(150.0, size.x - 150.0), r.range(250.0, size.y - 90.0));
        if chapel.free(p) && !v.iter().any(|g: &Ghost| g.pos.distance(p) < 26.0) {
            v.push(Ghost { pos: p, kind: r.below(CRITTERS.len()), phase: r.range(0.0, 1.0), style: r.below(4) as u8 });
        }
    }
    v
}
