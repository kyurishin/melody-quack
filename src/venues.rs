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
    Mail,
    Decks,
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

#[derive(Clone)]
pub(crate) struct OnAir {
    pub(crate) audio: bool,
    pub(crate) url: String,
    pub(crate) name: String,
}

pub(crate) struct Friend {
    pub(crate) admin: bool,
    pub(crate) dj: bool,
    pub(crate) id: u32,
    pub(crate) players: u32,
    pub(crate) characters: [usize; 2],
    pub(crate) name: String,
}

#[derive(Clone)]
pub(crate) struct Tonight {
    pub(crate) starts: f64,
    pub(crate) phase: u8,
    pub(crate) mine: u8,
    pub(crate) rank: usize,
    pub(crate) left: usize,
    pub(crate) info: String,
}

pub(crate) struct Peer {
    pub(crate) id: u32,
    pub(crate) slot: usize,
    pub(crate) character: usize,
    pub(crate) pos: Vec2,
    pub(crate) to: Vec2,
    pub(crate) facing: Vec2,
    pub(crate) walking: bool,
    pub(crate) sing: f32,
}

#[derive(Default)]
pub(crate) struct VenueState {
    pub(crate) dj_key: String,
    pub(crate) key_draft: String,
    pub(crate) key_sent: bool,
    pub(crate) dj_name: Option<String>,
    pub(crate) credit: u32,
    pub(crate) admin: bool,
    pub(crate) requests: Vec<(u32, String, String)>,
    pub(crate) balances: Vec<(u32, String)>,
    pub(crate) granted: Vec<String>,
    pub(crate) asked: bool,
    pub(crate) on_air: Option<OnAir>,
    pub(crate) mix: (bool, bool, usize, f32),
    pub(crate) bpm_now: f32,
    pub(crate) stream: String,
    pub(crate) welcome: bool,
    pub(crate) dj_page: bool,
    pub(crate) post_tab: bool,
    pub(crate) look: usize,
    pub(crate) chat_open: bool,
    pub(crate) tour: u8,
    pub(crate) tour_clock: f32,
    pub(crate) chat_draft: String,
    pub(crate) chat_log: Vec<(u8, String, String, f64)>,
    pub(crate) field: u8,
    pub(crate) patreon: String,
    pub(crate) name: String,
    pub(crate) name_draft: String,
    pub(crate) token: u64,
    pub(crate) hello: bool,
    pub(crate) friends: Vec<Friend>,
    pub(crate) tonight: Option<Tonight>,
    pub(crate) mail_seen: (String, u8, u8),
    pub(crate) sat: Option<(String, f32)>,
    pub(crate) ask: bool,
    pub(crate) live_list: bool,
    pub(crate) peers: Vec<Peer>,
    pub(crate) net_retry: f32,
    pub(crate) net_send: f32,
    pub(crate) joined: bool,
    pub(crate) announced: usize,
    pub(crate) find: Option<(String, f32)>,
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

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(crate) fn b64_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
        for k in 0..chunk.len() + 1 {
            out.push(B64[((n >> (18 - 6 * k)) & 63) as usize] as char);
        }
    }
    out
}

pub(crate) fn b64_decode(text: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut have = 0;
    let mut out = Vec::new();
    for c in text.trim().trim_end_matches('=').bytes() {
        let c = match c {
            b'+' => b'-',
            b'/' => b'_',
            c => c,
        };
        let v = B64.iter().position(|x| *x == c)? as u32;
        bits = (bits << 6) | v;
        have += 6;
        if have >= 8 {
            have -= 8;
            out.push((bits >> have) as u8);
            bits &= (1 << have) - 1;
        }
    }
    Some(out)
}

fn squash(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase()
}

struct Hard {
    port: u32,
    commits: [String; 3],
    sig: String,
    guest: String,
}

impl VenueState {
    pub(crate) fn hard(&self) -> bool {
        matches!(self.name.trim().to_lowercase().as_str(), "leon" | "neomole")
    }

    fn hard_setup(&self) -> Hard {
        let mut rng = self.secret.as_ref().map(|r| r.seed).unwrap_or(7) ^ 0x1e0a_5eed;
        let mut hex = |n: usize| (0..n).map(|_| char::from_digit(next(&mut rng) % 16, 16).unwrap()).collect::<String>();
        let commits = [hex(7), hex(7), hex(7)];
        let port = 3000 + next(&mut rng) % 6000;
        let sig: String = (0..43).map(|_| B64[(next(&mut rng) % 64) as usize] as char).collect();
        let guest = format!("{}.{}.{}", b64_encode(br#"{"alg":"HS256","typ":"JWT"}"#), b64_encode(br#"{"sub":"goose","role":"guest"}"#), sig);
        Hard { port, commits, sig, guest }
    }

    fn run_hard(&mut self, line: &str) -> bool {
        let h = self.hard_setup();
        self.term.push(format!("leon@pc7:~$ {line}"));
        let line = line.trim();
        let (cmd, arg) = line.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((line, ""));
        let mut out: Vec<String> = Vec::new();
        let mut won = false;
        let mut say = |t: &str| out.push(t.to_string());
        match (cmd, arg) {
            ("", _) => {}
            ("help", _) => {
                say("hard mode. tools: ls [-a]  cat  git log|show <sha>");
                say("base64 [-d] <text>  curl [-H 'k: v'] <url>");
                say("xxd -r -p <hex>  hint  whoami  clear");
                say("goal: tonight's code. it's behind the goose's api.");
            }
            ("hint", _) => {
                say("1. history remembers what HEAD forgot.");
                say("2. a token is three base64url parts. read them.");
                say("3. look at how app.js trusts the header.");
            }
            ("whoami", _) => say("leon (the goose has heard of you)"),
            ("ls", "-a") | ("ls", "-la") | ("ls", "-al") => say(".  ..  .git  .env  app.js  notes.txt"),
            ("ls", _) => say("app.js  notes.txt"),
            ("clear", _) => {}
            ("cat", "notes.txt") => {
                say("kim: the goose's ravenet api is up on");
                say(&format!("localhost:{}. he says the code endpoint", h.port));
                say("is admin only now. he rotated a secret");
                say("after 'an incident'. i don't ask.");
            }
            ("cat", ".env") => {
                say("JWT_SECRET=<rotated, ask the goose>");
                say("SESSION=");
            }
            ("cat", "app.js") => {
                for l in [
                    "get('/admin/code', (req, res) => {",
                    "  const t = decode(req.headers.authorization)",
                    "  if (!verify(t)) return res.status(401).end()",
                    "  if (t.payload.sub !== 'goose') return res.status(404).end()",
                    "  if (t.payload.role !== 'admin') return res.status(403).end()",
                    "  res.send(hex(tonight.code))",
                    "})",
                    "const verify = t => t.header.alg === 'none'",
                    "  || hmac(t.body, env.JWT_SECRET) === t.sig",
                    "// TODO(goose): drop alg none before prod",
                ] {
                    say(l);
                }
            }
            ("cat", "") => say("cat: missing operand"),
            ("cat", _) => say("cat: no such file"),
            ("git", "log") | ("git", "log --oneline") => {
                say(&format!("{} (HEAD) rotate jwt secret, clear session", h.commits[0]));
                say(&format!("{} add admin-only code endpoint", h.commits[1]));
                say(&format!("{} quick test session for the door", h.commits[2]));
            }
            ("git", a) if a.starts_with("show") => {
                let sha = a.trim_start_matches("show").trim();
                if sha.len() >= 4 && h.commits[2].starts_with(sha) {
                    say(&format!("commit {}", h.commits[2]));
                    say("diff --git a/.env b/.env");
                    say("-SESSION=");
                    say(&format!("+SESSION={}", h.guest));
                } else if sha.len() >= 4 && h.commits.iter().any(|c| c.starts_with(sha)) {
                    say("diff --git a/app.js b/app.js");
                    say("(nothing interesting in this one)");
                } else {
                    say("fatal: bad revision. try git log");
                }
            }
            ("git", _) => say("usage: git log | git show <sha>"),
            ("base64", a) if a.starts_with("-d") => {
                let text = a.trim_start_matches("-d").trim().trim_matches(|c| c == '\'' || c == '"');
                match b64_decode(text) {
                    Some(bytes) => say(&String::from_utf8_lossy(&bytes)),
                    None => say("base64: invalid input"),
                }
            }
            ("base64", "") => say("base64: give it some text"),
            ("base64", a) => {
                let text = a.trim_matches(|c| c == '\'' || c == '"');
                say(&b64_encode(text.as_bytes()));
                say("(base64url, no padding)");
            }
            ("curl", a) => {
                let token = a.split_whitespace().skip_while(|w| !w.trim_matches(|c| c == '\'' || c == '"').eq_ignore_ascii_case("bearer")).nth(1).map(|t| t.trim_matches(|c| c == '\'' || c == '"').to_string());
                let url = a.split_whitespace().find(|w| w.contains("localhost") || w.contains("127.0.0.1")).unwrap_or("").trim_matches(|c| c == '\'' || c == '"');
                let port = url.split(':').nth(if url.starts_with("http") { 2 } else { 1 }).and_then(|p| p.split('/').next()).and_then(|p| p.parse::<u32>().ok());
                if url.is_empty() {
                    say("curl: give it a url, like localhost:<port>/");
                } else if port != Some(h.port) {
                    say("curl: (7) connection refused");
                } else if !url.contains("/admin/code") {
                    say("{\"service\":\"ravenet\",\"routes\":[\"/admin/code\"]}");
                } else {
                    match token {
                        None => say("HTTP/1.1 401 Unauthorized"),
                        Some(t) => {
                            let parts: Vec<&str> = t.split('.').collect();
                            let dec = |p: &str| b64_decode(p).map(|b| squash(&String::from_utf8_lossy(&b))).unwrap_or_default();
                            let header = parts.first().map(|p| dec(p)).unwrap_or_default();
                            let payload = parts.get(1).map(|p| dec(p)).unwrap_or_default();
                            let none = header.contains(r#""alg":"none""#);
                            let genuine = parts.len() == 3 && t == h.guest;
                            let signed_ok = genuine || (none && parts.len() >= 2);
                            if !signed_ok {
                                say("HTTP/1.1 401 Unauthorized");
                                say(if parts.len() == 3 && parts[2] == h.sig { "invalid signature" } else { "malformed token" });
                            } else if !payload.contains(r#""sub":"goose""#) {
                                say("HTTP/1.1 404 Not Found");
                            } else if !payload.contains(r#""role":"admin""#) {
                                say("HTTP/1.1 403 Forbidden");
                                say("{\"error\":\"admins only\"}");
                            } else if self.live_list {
                                say("HTTP/1.1 200 OK");
                                say("asking the goose's list...");
                                self.ask = true;
                            } else {
                                let code = self.secret.as_ref().map(|r| r.code.clone()).unwrap_or_default();
                                let hexed: String = code.bytes().map(|b| format!("{b:02x}")).collect();
                                say("HTTP/1.1 200 OK");
                                say(&hexed);
                            }
                        }
                    }
                }
            }
            ("xxd", a) if a.starts_with("-r") => {
                let hexed: String = a.split_whitespace().last().unwrap_or("").chars().filter(|c| c.is_ascii_hexdigit()).collect();
                let bytes: Vec<u8> = (0..hexed.len() / 2).filter_map(|i| u8::from_str_radix(&hexed[i * 2..i * 2 + 2], 16).ok()).collect();
                let text = String::from_utf8_lossy(&bytes).to_string();
                say(&text);
                if let Some(code) = tidy_code(&text).filter(|c| self.secret.as_ref().map(|r| &r.code == c).unwrap_or(false)) {
                    say("flag captured. respect, leon. the goose will know this code.");
                    self.learn(&code);
                    self.flag = true;
                    won = true;
                }
            }
            ("xxd", _) => say("usage: xxd -r -p <hex>"),
            _ => say("command not found. type help."),
        }
        self.term.extend(out);
        if cmd == "clear" {
            self.term.clear();
        }
        let n = self.term.len();
        if n > 60 {
            self.term.drain(..n - 60);
        }
        won
    }

    pub(crate) fn run(&mut self, line: &str) -> bool {
        if self.hard() {
            return self.run_hard(line);
        }
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
                if user == "goose" && self.live_list {
                    say("asking the goose's list...");
                    self.ask = true;
                } else if user == "goose" {
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

    pub(crate) fn new(name: String, dj_key: String, welcome: bool) -> VenueState {
        let mut v = VenueState { pc_bpm: 132.0, pc_sound: 1, rng: 0x51ab_1e55, name_draft: name.clone(), key_draft: dj_key.clone(), dj_page: !dj_key.is_empty(), name, dj_key, welcome, ..Default::default() };
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
        v.term = vec![if v.hard() { "ravenet dev box. hard mode. type help.".to_string() } else { "quack 95 terminal. type help.".to_string() }];
        v.token = ((next(&mut v.rng) as u64) << 32) | next(&mut v.rng) as u64 | 1;
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

pub(crate) const LOOKS: [&str; 4] = ["gothic chapel", "jungle sunset", "san francisco view", "kinky party"];

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

pub(crate) fn guide_arrow(b: &mut Batch, base: Vec3, px: f32) {
    let gold = Color::from_rgba(255, 214, 110, 255);
    let dark = Color::from_rgba(40, 24, 60, 255);
    let shape: [(i32, i32); 13] = [(0, 0), (-1, 1), (0, 1), (1, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2), (0, 3), (0, 4), (0, 5), (0, 6)];
    let was = b.unlit;
    b.unlit = true;
    for (dx, dy) in shape {
        b.cube(base + vec3(dx as f32 * px, dy as f32 * px, -0.02), vec3(px * 1.8, px * 1.8, 0.03), dark);
    }
    for (dx, dy) in shape {
        b.cube(base + vec3(dx as f32 * px, dy as f32 * px, 0.0), vec3(px, px, 0.04), gold);
    }
    b.unlit = was;
}

pub(crate) fn draw_exteriors(b: &mut Batch, dusk: f32, see_through: bool, focus: Vec2, time: f32) {
    let was_unlit = b.unlit;
    let mark = TOUR_MARK.with(|m| m.get());
    if !see_through && mark > 0 {
        let door = if mark == 1 { PCBANG_DOOR } else { CHAPEL_DOOR };
        let bob = (time * 4.0).sin().abs() * 0.25;
        let top = if mark == 1 { 3.3 } else { 5.0 };
        b.unlit = true;
        guide_arrow(b, vec3(door.x, top + bob, door.y + 0.3), 0.1);
        b.unlit = was_unlit;
    }

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
    pub(crate) static TOUR_MARK: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
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
