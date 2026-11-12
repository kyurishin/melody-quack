//! Shared code for the Melody Quack client and server: the wire protocol,
//! rave codes, a tiny deterministic RNG and the `Hub`, which holds all
//! multiplayer state. The server wraps the hub in websockets; the game runs
//! the very same hub in-process for offline play.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub const PROTOCOL_VERSION: u32 = 2;
pub const DEFAULT_PORT: u16 = 7777;

// ---------------------------------------------------------------------------
// RNG
// ---------------------------------------------------------------------------

/// SplitMix64. Small, fast and identical on every platform, so a rave seed
/// produces the same flyer and the same music for every player.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }
    /// Uniform integer in [0, n).
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

pub fn hash_str(s: &str) -> u64 {
    // FNV-1a
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------------------
// Rave codes
// ---------------------------------------------------------------------------

/// Letters and digits that can't be confused when scrawled on a poster
/// (no 0/O, 1/I/L, 5/S, 2/Z, 8/B).
pub const CODE_ALPHABET: &[u8] = b"ACDEFGHJKMNPQRTUVWXY3479";

/// A Zoom-style invitation code: `QK7-MAX-9PE`.
pub fn generate_code(rng: &mut Rng) -> String {
    let mut s = String::with_capacity(11);
    for i in 0..9 {
        if i == 3 || i == 6 {
            s.push('-');
        }
        s.push(CODE_ALPHABET[rng.below(CODE_ALPHABET.len())] as char);
    }
    s
}

/// Accepts what a person might type ("qk7 max 9pe", "QK7MAX9PE") and returns
/// the canonical `QK7-MAX-9PE`, or `None` if it can't be a code.
pub fn normalize_code(input: &str) -> Option<String> {
    let chars: Vec<char> = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .map(|c| match c {
            '0' => 'Q', // people write O for Q-ish shapes; 0 isn't in the alphabet
            other => other,
        })
        .collect();
    if chars.len() != 9 || !chars.iter().all(|c| CODE_ALPHABET.contains(&(*c as u8))) {
        return None;
    }
    let mut s = String::with_capacity(11);
    for (i, c) in chars.iter().enumerate() {
        if i == 3 || i == 6 {
            s.push('-');
        }
        s.push(*c);
    }
    Some(s)
}

// ---------------------------------------------------------------------------
// Protocol
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Scene {
    Town,
    PcBang,
    Rave,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sound {
    Acid,
    Gothic,
    Trance,
}

impl Sound {
    pub const ALL: [Sound; 3] = [Sound::Acid, Sound::Gothic, Sound::Trance];
    pub fn label(self) -> &'static str {
        match self {
            Sound::Acid => "ACID TECHNO",
            Sound::Gothic => "GOTHIC TECHNO",
            Sound::Trance => "HARD TRANCE",
        }
    }
    pub fn default_bpm(self) -> f32 {
        match self {
            Sound::Acid => 136.0,
            Sound::Gothic => 130.0,
            Sound::Trance => 142.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayerState {
    pub id: u32,
    pub name: String,
    pub avatar: u8,
    pub dj: bool,
    pub x: f32,
    pub y: f32,
    pub facing: i8,
    pub moving: bool,
    pub dance: u8,
    pub seated: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RaveInfo {
    pub code: String,
    pub title: String,
    pub host_name: String,
    pub host_is_dj: bool,
    pub seed: u64,
    pub sound: Sound,
    pub bpm: f32,
    /// Server clock (ms since epoch) at beat zero of the generated set.
    pub start_ms: u64,
    /// When doors open, as advertised on the flyer.
    pub doors_ms: u64,
    /// Paste flyers around town (otherwise only people given the code know).
    pub flyers: bool,
    /// A DJ is streaming a live set right now.
    pub live: bool,
    pub crowd: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientMsg {
    Hello {
        name: String,
        avatar: u8,
        dj_key: Option<String>,
        version: u32,
    },
    Ping {
        t: f64,
    },
    /// Move to another scene. `code` is required for `Scene::Rave`.
    Enter {
        scene: Scene,
        code: Option<String>,
    },
    State {
        x: f32,
        y: f32,
        facing: i8,
        moving: bool,
        dance: u8,
        seated: bool,
    },
    Chat {
        text: String,
    },
    HostRave {
        title: String,
        sound: Sound,
        bpm: f32,
        flyers: bool,
        doors_in_min: u32,
    },
    /// DJ starts or stops streaming a live set into their own event.
    SetLive {
        live: bool,
    },
    GuestbookPost {
        text: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerMsg {
    Welcome {
        id: u32,
        name: String,
        dj: bool,
        server_ms: u64,
        motd: String,
    },
    Pong {
        t: f64,
        server_ms: u64,
    },
    Snapshot {
        players: Vec<PlayerState>,
    },
    Left {
        id: u32,
    },
    Chat {
        id: u32,
        name: String,
        dj: bool,
        text: String,
    },
    /// Every rave that has flyers out, for posters and the underground page.
    Flyers {
        raves: Vec<RaveInfo>,
    },
    RaveHosted {
        rave: RaveInfo,
    },
    Entered {
        scene: Scene,
        rave: Option<RaveInfo>,
    },
    RaveUpdate {
        rave: RaveInfo,
    },
    Denied {
        reason: String,
    },
    Guestbook {
        entries: Vec<(String, String)>,
    },
    Notice {
        text: String,
    },
}

/// Binary frames carry live DJ audio: one Opus packet each, 20 ms of 48 kHz
/// stereo. Client→server frames are the raw packet; server→client frames are
/// the same bytes (the server never decodes them).
pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_FRAME: usize = 960; // samples per channel per packet (20 ms)

pub fn encode<T: Serialize>(msg: &T) -> String {
    serde_json::to_string(msg).expect("protocol messages always serialise")
}

pub fn decode<'a, T: Deserialize<'a>>(text: &'a str) -> Option<T> {
    serde_json::from_str(text).ok()
}

// ---------------------------------------------------------------------------
// Hub
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Payload {
    Msg(ServerMsg),
    Audio(Arc<Vec<u8>>),
}

pub type Out = Vec<(u32, Payload)>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Room {
    Lobby, // connected, not said hello yet
    Town,
    PcBang,
    Rave(String),
}

struct Player {
    state: PlayerState,
    room: Room,
    greeted: bool,
    last_chat_ms: u64,
}

struct Rave {
    info: RaveInfo,
    host_id: u32,
    created_ms: u64,
    last_seen_ms: u64,
}

pub struct HubConfig {
    /// DJ key -> DJ name. A player who says hello with a known key becomes a
    /// DJ under that name (purple dot) and can stream live sets.
    pub djs: HashMap<String, String>,
    /// Offline/dev: any non-empty key makes you a DJ under your chosen name.
    pub any_key_is_dj: bool,
    pub motd: String,
}

impl Default for HubConfig {
    fn default() -> Self {
        HubConfig {
            djs: HashMap::new(),
            any_key_is_dj: false,
            motd: "welcome to quackville. the chapel never sleeps.".into(),
        }
    }
}

impl HubConfig {
    /// Parse `name=key` lines (or `name:key`), `#` comments allowed.
    pub fn parse_djs(text: &str) -> HashMap<String, String> {
        let mut out = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((name, key)) = line.split_once(['=', ':']) {
                let (name, key) = (name.trim(), key.trim());
                if !name.is_empty() && !key.is_empty() {
                    out.insert(key.to_string(), sanitize_name(name));
                }
            }
        }
        out
    }
}

pub struct Hub {
    cfg: HubConfig,
    players: HashMap<u32, Player>,
    raves: HashMap<String, Rave>,
    guestbook: VecDeque<(String, String)>,
    next_id: u32,
    rng: Rng,
    flyers_dirty: bool,
}

/// An empty rave stays open this long after it was created or last visited.
const RAVE_IDLE_MS: u64 = 3 * 60 * 60 * 1000;
const MAX_RAVES: usize = 500;

pub fn sanitize_name(name: &str) -> String {
    let s: String = name
        .chars()
        .filter(|c| !c.is_control())
        .take(14)
        .collect::<String>()
        .trim()
        .to_string();
    if s.is_empty() {
        "duckling".into()
    } else {
        s
    }
}

fn sanitize_text(text: &str, max: usize) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect::<String>()
        .trim()
        .to_string()
}

impl Hub {
    pub fn new(cfg: HubConfig, seed: u64) -> Self {
        let mut guestbook = VecDeque::new();
        guestbook.push_back((
            "mr. kim".to_string(),
            "first hour free. snacks are not.".to_string(),
        ));
        Hub {
            cfg,
            players: HashMap::new(),
            raves: HashMap::new(),
            guestbook,
            next_id: 1,
            rng: Rng::new(seed),
            flyers_dirty: true,
        }
    }

    pub fn player_count(&self) -> usize {
        self.players.values().filter(|p| p.greeted).count()
    }

    pub fn connect(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.players.insert(
            id,
            Player {
                state: PlayerState {
                    id,
                    name: String::new(),
                    avatar: 0,
                    dj: false,
                    x: 0.0,
                    y: 0.0,
                    facing: 1,
                    moving: false,
                    dance: 0,
                    seated: false,
                },
                room: Room::Lobby,
                greeted: false,
                last_chat_ms: 0,
            },
        );
        id
    }

    pub fn disconnect(&mut self, id: u32, _now: u64) -> Out {
        let mut out = Out::new();
        if let Some(p) = self.players.remove(&id) {
            self.leave_room(id, &p.room, &mut out);
        }
        out
    }

    fn room_members(&self, room: &Room) -> Vec<u32> {
        self.players
            .iter()
            .filter(|(_, p)| p.greeted && &p.room == room)
            .map(|(id, _)| *id)
            .collect()
    }

    fn leave_room(&mut self, id: u32, room: &Room, out: &mut Out) {
        for m in self.room_members(room) {
            if m != id {
                out.push((m, Payload::Msg(ServerMsg::Left { id })));
            }
        }
        if let Room::Rave(code) = room {
            if let Some(r) = self.raves.get_mut(code) {
                if r.host_id == id && r.info.live {
                    r.info.live = false;
                    let rave = r.info.clone();
                    self.flyers_dirty = true;
                    for m in self.room_members(room) {
                        out.push((m, Payload::Msg(ServerMsg::RaveUpdate { rave: rave.clone() })));
                    }
                }
            }
        }
    }

    fn flyer_list(&self) -> Vec<RaveInfo> {
        let mut v: Vec<RaveInfo> = self
            .raves
            .values()
            .filter(|r| r.info.flyers)
            .map(|r| r.info.clone())
            .collect();
        v.sort_by(|a, b| b.start_ms.cmp(&a.start_ms));
        v.truncate(24);
        v
    }

    pub fn handle(&mut self, id: u32, msg: ClientMsg, now: u64) -> Out {
        let mut out = Out::new();
        let Some(p) = self.players.get(&id) else {
            return out;
        };
        let greeted = p.greeted;
        match msg {
            ClientMsg::Hello {
                name,
                avatar,
                dj_key,
                version,
            } => {
                if version != PROTOCOL_VERSION {
                    out.push((
                        id,
                        Payload::Msg(ServerMsg::Denied {
                            reason: "this server runs a different version of melody quack".into(),
                        }),
                    ));
                    return out;
                }
                let mut name = sanitize_name(&name);
                let mut dj = false;
                if let Some(key) = dj_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
                    if let Some(dj_name) = self.cfg.djs.get(&key) {
                        name = dj_name.clone();
                        dj = true;
                    } else if self.cfg.any_key_is_dj {
                        dj = true;
                    } else {
                        out.push((
                            id,
                            Payload::Msg(ServerMsg::Notice {
                                text: "that dj key isn't on the list. joining as a raver.".into(),
                            }),
                        ));
                    }
                }
                if !dj && self.cfg.djs.values().any(|n| n.eq_ignore_ascii_case(&name)) {
                    // Nobody gets to impersonate a DJ.
                    name = format!("{}~", name.chars().take(13).collect::<String>());
                }
                let p = self.players.get_mut(&id).unwrap();
                p.state.name = name.clone();
                p.state.avatar = avatar % 8;
                p.state.dj = dj;
                p.greeted = true;
                p.room = Room::Lobby;
                out.push((
                    id,
                    Payload::Msg(ServerMsg::Welcome {
                        id,
                        name,
                        dj,
                        server_ms: now,
                        motd: self.cfg.motd.clone(),
                    }),
                ));
                out.push((id, Payload::Msg(ServerMsg::Flyers { raves: self.flyer_list() })));
                out.push((
                    id,
                    Payload::Msg(ServerMsg::Guestbook {
                        entries: self.guestbook.iter().cloned().collect(),
                    }),
                ));
            }
            _ if !greeted => {}
            ClientMsg::Ping { t } => {
                out.push((id, Payload::Msg(ServerMsg::Pong { t, server_ms: now })));
            }
            ClientMsg::Enter { scene, code } => {
                let new_room = match scene {
                    Scene::Town => Room::Town,
                    Scene::PcBang => Room::PcBang,
                    Scene::Rave => {
                        let Some(code) = code.as_deref().and_then(normalize_code) else {
                            out.push((
                                id,
                                Payload::Msg(ServerMsg::Denied {
                                    reason: "that doesn't look like a code. nine letters, like QK7-MAX-9PE.".into(),
                                }),
                            ));
                            return out;
                        };
                        if !self.raves.contains_key(&code) {
                            out.push((
                                id,
                                Payload::Msg(ServerMsg::Denied {
                                    reason: "no rave under that code. maybe the night is already over.".into(),
                                }),
                            ));
                            return out;
                        }
                        Room::Rave(code)
                    }
                };
                let old = self.players[&id].room.clone();
                if old != new_room {
                    self.leave_room(id, &old, &mut out);
                }
                let p = self.players.get_mut(&id).unwrap();
                p.room = new_room.clone();
                p.state.seated = false;
                p.state.dance = 0;
                let rave = if let Room::Rave(code) = &new_room {
                    let r = self.raves.get_mut(code).unwrap();
                    r.last_seen_ms = now;
                    Some(r.info.clone())
                } else {
                    None
                };
                out.push((id, Payload::Msg(ServerMsg::Entered { scene, rave })));
            }
            ClientMsg::State {
                x,
                y,
                facing,
                moving,
                dance,
                seated,
            } => {
                let p = self.players.get_mut(&id).unwrap();
                if x.is_finite() && y.is_finite() {
                    p.state.x = x.clamp(-10_000.0, 10_000.0);
                    p.state.y = y.clamp(-10_000.0, 10_000.0);
                }
                p.state.facing = facing.signum();
                p.state.moving = moving;
                p.state.dance = dance.min(8);
                p.state.seated = seated;
            }
            ClientMsg::Chat { text } => {
                let text = sanitize_text(&text, 120);
                let p = self.players.get_mut(&id).unwrap();
                if text.is_empty() || now.saturating_sub(p.last_chat_ms) < 400 {
                    return out;
                }
                p.last_chat_ms = now;
                let (name, dj, room) = (p.state.name.clone(), p.state.dj, p.room.clone());
                for m in self.room_members(&room) {
                    out.push((
                        m,
                        Payload::Msg(ServerMsg::Chat {
                            id,
                            name: name.clone(),
                            dj,
                            text: text.clone(),
                        }),
                    ));
                }
            }
            ClientMsg::HostRave {
                title,
                sound,
                bpm,
                flyers,
                doors_in_min,
            } => {
                let hosted_by_me = self.raves.values().filter(|r| r.host_id == id).count();
                if hosted_by_me >= 3 || self.raves.len() >= MAX_RAVES {
                    out.push((
                        id,
                        Payload::Msg(ServerMsg::Denied {
                            reason: "you already have three raves going. that's plenty.".into(),
                        }),
                    ));
                    return out;
                }
                let mut code = generate_code(&mut self.rng);
                while self.raves.contains_key(&code) {
                    code = generate_code(&mut self.rng);
                }
                let mut title = sanitize_text(&title, 24).to_uppercase();
                if title.is_empty() {
                    title = "UNTITLED RAVE".into();
                }
                let p = &self.players[&id];
                let info = RaveInfo {
                    code: code.clone(),
                    title,
                    host_name: p.state.name.clone(),
                    host_is_dj: p.state.dj,
                    seed: self.rng.next_u64(),
                    sound,
                    bpm: if bpm.is_finite() { bpm.clamp(110.0, 160.0) } else { sound.default_bpm() },
                    start_ms: now,
                    doors_ms: now + doors_in_min.min(24 * 60) as u64 * 60_000,
                    flyers,
                    live: false,
                    crowd: 0,
                };
                self.raves.insert(
                    code,
                    Rave {
                        info: info.clone(),
                        host_id: id,
                        created_ms: now,
                        last_seen_ms: now,
                    },
                );
                if flyers {
                    self.flyers_dirty = true;
                }
                out.push((id, Payload::Msg(ServerMsg::RaveHosted { rave: info })));
            }
            ClientMsg::SetLive { live } => {
                let p = &self.players[&id];
                let Room::Rave(code) = p.room.clone() else {
                    return out;
                };
                let is_dj = p.state.dj;
                let Some(r) = self.raves.get_mut(&code) else {
                    return out;
                };
                if live && (!is_dj || r.host_id != id) {
                    out.push((
                        id,
                        Payload::Msg(ServerMsg::Denied {
                            reason: "only the dj who hosts this event can play live here.".into(),
                        }),
                    ));
                    return out;
                }
                if r.host_id != id || r.info.live == live {
                    return out;
                }
                r.info.live = live;
                let rave = r.info.clone();
                self.flyers_dirty = true;
                for m in self.room_members(&Room::Rave(code)) {
                    out.push((m, Payload::Msg(ServerMsg::RaveUpdate { rave: rave.clone() })));
                }
            }
            ClientMsg::GuestbookPost { text } => {
                let text = sanitize_text(&text, 140);
                let p = self.players.get_mut(&id).unwrap();
                if text.is_empty() || now.saturating_sub(p.last_chat_ms) < 2000 {
                    return out;
                }
                p.last_chat_ms = now;
                self.guestbook.push_front((p.state.name.clone(), text));
                self.guestbook.truncate(40);
                let entries: Vec<_> = self.guestbook.iter().cloned().collect();
                for (pid, pl) in &self.players {
                    if pl.greeted {
                        out.push((*pid, Payload::Msg(ServerMsg::Guestbook { entries: entries.clone() })));
                    }
                }
            }
        }
        out
    }

    /// A DJ's Opus packet. Relayed to everyone else in their event.
    pub fn handle_audio(&mut self, id: u32, packet: Vec<u8>) -> Out {
        let mut out = Out::new();
        if packet.is_empty() || packet.len() > 4000 {
            return out;
        }
        let Some(p) = self.players.get(&id) else {
            return out;
        };
        let Room::Rave(code) = &p.room else {
            return out;
        };
        let Some(r) = self.raves.get(code) else {
            return out;
        };
        if r.host_id != id || !r.info.live {
            return out;
        }
        let data = Arc::new(packet);
        for m in self.room_members(&p.room) {
            if m != id {
                out.push((m, Payload::Audio(data.clone())));
            }
        }
        out
    }

    /// Call ~15 times a second: position snapshots, rave bookkeeping.
    pub fn tick(&mut self, now: u64) -> Out {
        let mut out = Out::new();

        // Crowd counts and expiry.
        let mut crowd: HashMap<String, u32> = HashMap::new();
        for p in self.players.values() {
            if let Room::Rave(code) = &p.room {
                *crowd.entry(code.clone()).or_default() += 1;
            }
        }
        let mut expired = Vec::new();
        for (code, r) in self.raves.iter_mut() {
            let c = crowd.get(code).copied().unwrap_or(0);
            if c > 0 {
                r.last_seen_ms = now;
            }
            if r.info.crowd != c {
                r.info.crowd = c;
                if r.info.flyers {
                    self.flyers_dirty = true;
                }
            }
            let idle_from = r.last_seen_ms.max(r.info.doors_ms).max(r.created_ms);
            if c == 0 && now.saturating_sub(idle_from) > RAVE_IDLE_MS {
                expired.push(code.clone());
            }
        }
        for code in expired {
            if let Some(r) = self.raves.remove(&code) {
                if r.info.flyers {
                    self.flyers_dirty = true;
                }
            }
        }

        if self.flyers_dirty {
            self.flyers_dirty = false;
            let raves = self.flyer_list();
            for (id, p) in &self.players {
                if p.greeted {
                    out.push((*id, Payload::Msg(ServerMsg::Flyers { raves: raves.clone() })));
                }
            }
        }

        // Snapshots per room.
        let mut rooms: HashMap<Room, Vec<PlayerState>> = HashMap::new();
        for p in self.players.values() {
            if p.greeted && p.room != Room::Lobby {
                rooms.entry(p.room.clone()).or_default().push(p.state.clone());
            }
        }
        for (room, players) in rooms {
            for m in self.room_members(&room) {
                out.push((m, Payload::Msg(ServerMsg::Snapshot { players: players.clone() })));
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(hub: &mut Hub, name: &str, key: Option<&str>) -> u32 {
        let id = hub.connect();
        hub.handle(
            id,
            ClientMsg::Hello {
                name: name.into(),
                avatar: 0,
                dj_key: key.map(Into::into),
                version: PROTOCOL_VERSION,
            },
            0,
        );
        id
    }

    fn msgs(out: &Out, to: u32) -> Vec<ServerMsg> {
        out.iter()
            .filter(|(id, _)| *id == to)
            .filter_map(|(_, p)| match p {
                Payload::Msg(m) => Some(m.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn codes_round_trip() {
        let mut rng = Rng::new(42);
        for _ in 0..1000 {
            let c = generate_code(&mut rng);
            assert_eq!(c.len(), 11);
            assert_eq!(normalize_code(&c).as_deref(), Some(c.as_str()));
            assert_eq!(normalize_code(&c.to_lowercase().replace('-', " ")).as_deref(), Some(c.as_str()));
        }
        assert_eq!(normalize_code("hello"), None);
        assert_eq!(normalize_code("QK7-M2X-9PB"), None); // 2 and B aren't in the alphabet
    }

    #[test]
    fn host_join_and_live() {
        let mut cfg = HubConfig::default();
        cfg.djs = HubConfig::parse_djs("# djs\nAuraDJ = secret1\n");
        let mut hub = Hub::new(cfg, 1);
        let dj = hello(&mut hub, "whoever", Some("secret1"));
        let fan = hello(&mut hub, "AuraDJ", None);
        assert!(hub.players[&dj].state.dj);
        assert_eq!(hub.players[&dj].state.name, "AuraDJ");
        assert_eq!(hub.players[&fan].state.name, "AuraDJ~");

        let out = hub.handle(
            dj,
            ClientMsg::HostRave {
                title: "acid mass".into(),
                sound: Sound::Gothic,
                bpm: 130.0,
                flyers: true,
                doors_in_min: 0,
            },
            10,
        );
        let code = match &msgs(&out, dj)[0] {
            ServerMsg::RaveHosted { rave } => rave.code.clone(),
            m => panic!("{m:?}"),
        };
        hub.handle(dj, ClientMsg::Enter { scene: Scene::Rave, code: Some(code.clone()) }, 20);
        let out = hub.handle(
            fan,
            ClientMsg::Enter { scene: Scene::Rave, code: Some(code.to_lowercase()) },
            20,
        );
        assert!(matches!(msgs(&out, fan)[0], ServerMsg::Entered { rave: Some(_), .. }));

        // The fan can't go live; the DJ can, and their audio reaches the fan.
        let out = hub.handle(fan, ClientMsg::SetLive { live: true }, 30);
        assert!(matches!(msgs(&out, fan)[0], ServerMsg::Denied { .. }));
        hub.handle(dj, ClientMsg::SetLive { live: true }, 30);
        let out = hub.handle_audio(dj, vec![1, 2, 3]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, fan);

        // DJ leaves: the set stops.
        hub.disconnect(dj, 40);
        let r = hub.raves.get(&code).unwrap();
        assert!(!r.info.live);
    }

    #[test]
    fn unknown_code_denied() {
        let mut hub = Hub::new(HubConfig::default(), 1);
        let a = hello(&mut hub, "a", None);
        let out = hub.handle(a, ClientMsg::Enter { scene: Scene::Rave, code: Some("ACD-EFG-HJK".into()) }, 0);
        assert!(matches!(msgs(&out, a)[0], ServerMsg::Denied { .. }));
    }
}
