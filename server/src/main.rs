use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
mod accounts;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

const ALPHABET: &str = "ACDEFGHJKMNPQRTUVWXY3479";
const IDLE_MS: i64 = 6 * 60 * 60 * 1000;
const MAX_ROOMS: usize = 2000;
const MAX_LINE: usize = 240;
const ROOM_CAP: u32 = 100;

const LEAD_MS: i64 = 30 * 60 * 1000;
const WINDOW_MS: i64 = 10 * 1000;
const HEAD_START_MS: i64 = 5 * 60 * 1000;
const PRIORITY: usize = 30;
const GENERAL: usize = 70;
const LENGTH_MS: i64 = 3 * 60 * 60 * 1000;

fn valid_code(code: &str) -> bool {
    let b: Vec<char> = code.chars().collect();
    b.len() == 11 && b.iter().enumerate().all(|(i, c)| if i == 3 || i == 7 { *c == '-' } else { ALPHABET.contains(*c) })
}

fn clean(text: &str, max: usize) -> String {
    text.chars().filter(|c| !c.is_control()).take(max).collect::<String>().trim().to_string()
}

fn next_rand(rng: &mut u64) -> u64 {
    *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn make_code(rng: &mut u64) -> String {
    let letters: Vec<char> = ALPHABET.chars().collect();
    let mut s = String::new();
    for i in 0..9 {
        if i == 3 || i == 6 {
            s.push('-');
        }
        s.push(letters[(next_rand(rng) % letters.len() as u64) as usize]);
    }
    s
}

#[derive(Debug, PartialEq)]
enum Refuse {
    Early(i64),
    List(i64),
    Full,
}

struct Event {
    start: i64,
    code: String,
    seed: u32,
    attempts: HashMap<u64, i64>,
    settled: bool,
    priority: Vec<u64>,
    tickets: HashSet<u64>,
}

impl Event {
    fn new(start: i64, rng: &mut u64) -> Event {
        Event { start, code: make_code(rng), seed: next_rand(rng) as u32, attempts: HashMap::new(), settled: false, priority: Vec::new(), tickets: HashSet::new() }
    }

    fn mark(&self) -> i64 {
        self.start - LEAD_MS
    }

    fn info(&self) -> String {
        format!("1|132|{}|the long waddle", self.seed)
    }

    fn sit(&mut self, token: u64, now: i64) -> Option<i64> {
        if let Some(off) = self.attempts.get(&token) {
            return Some(*off);
        }
        let off = now - self.mark();
        if off.abs() > WINDOW_MS || self.settled {
            return None;
        }
        self.attempts.insert(token, off);
        Some(off)
    }

    fn settle(&mut self, now: i64) {
        if self.settled || now < self.mark() + WINDOW_MS {
            return;
        }
        self.settled = true;
        let mut all: Vec<(u64, i64)> = self.attempts.iter().map(|(t, o)| (*t, *o)).collect();
        all.sort_by_key(|(t, o)| (o.abs(), *o, *t));
        self.priority = all.into_iter().take(PRIORITY).map(|(t, _)| t).collect();
    }

    fn general_at(&self) -> i64 {
        self.mark() + HEAD_START_MS
    }

    fn claim(&mut self, token: u64, now: i64) -> Result<(), Refuse> {
        self.settle(now);
        if !self.settled {
            return Err(Refuse::Early(self.mark() + WINDOW_MS - now));
        }
        if self.priority.contains(&token) || self.tickets.contains(&token) {
            return Ok(());
        }
        if now < self.general_at() {
            return Err(Refuse::List(self.general_at() - now));
        }
        if self.tickets.len() >= GENERAL {
            return Err(Refuse::Full);
        }
        self.tickets.insert(token);
        Ok(())
    }

    fn phase(&self, now: i64) -> u8 {
        if now >= self.start {
            4
        } else if now >= self.general_at() {
            3
        } else if self.settled {
            2
        } else if now >= self.mark() - WINDOW_MS {
            1
        } else {
            0
        }
    }

    fn mine(&self, token: u64) -> (u8, usize) {
        if let Some(rank) = self.priority.iter().position(|t| *t == token) {
            (2, rank + 1)
        } else if self.tickets.contains(&token) {
            (3, 0)
        } else if self.attempts.contains_key(&token) {
            (if self.settled { 4 } else { 1 }, 0)
        } else {
            (0, 0)
        }
    }
}

struct Room {
    info: String,
    born: i64,
    seen: i64,
}

struct Ask {
    id: u32,
    dj: String,
    code: String,
}

struct Live {
    by: u32,
    line: String,
    mix: String,
    look: String,
}

struct Client {
    tx: Option<mpsc::UnboundedSender<Message>>,
    token: u64,
    room: Option<String>,
    players: u32,
    who: String,
    last: [String; 2],
    dj: Option<String>,
    admin: bool,
    name: String,
    spoke: i64,
    account: Option<String>,
    session: Option<String>,
    put_at: i64,
}

struct Hub {
    rooms: HashMap<String, Room>,
    clients: HashMap<u32, Client>,
    next: u32,
    event: Event,
    every: i64,
    rng: u64,
    sent: Vec<(u32, String)>,
    djs: Vec<(String, String)>,
    djs_path: Option<String>,
    admin_key: Option<String>,
    admin_name: String,
    muted: Vec<String>,
    patreon: String,
    credits: HashMap<String, u32>,
    credits_path: Option<String>,
    asks: Vec<Ask>,
    next_ask: u32,
    grants: HashSet<(String, String)>,
    live: HashMap<String, Live>,
    accounts: accounts::Accounts,
}

impl Hub {
    fn new(start: i64, every: i64, seed: u64) -> Hub {
        let mut rng = seed;
        let event = Event::new(start, &mut rng);
        Hub {
            rooms: HashMap::new(),
            clients: HashMap::new(),
            next: 0,
            event,
            every,
            rng,
            sent: Vec::new(),
            djs: Vec::new(),
            djs_path: None,
            admin_key: None,
            admin_name: "admin".to_string(),
            muted: Vec::new(),
            patreon: String::new(),
            credits: HashMap::new(),
            credits_path: None,
            asks: Vec::new(),
            next_ask: 0,
            grants: HashSet::new(),
            live: HashMap::new(),
            accounts: accounts::Accounts::default(),
        }
    }

    fn connect(&mut self, tx: Option<mpsc::UnboundedSender<Message>>) -> u32 {
        self.next += 1;
        let token = next_rand(&mut self.rng);
        self.clients.insert(self.next, Client { tx, token, room: None, players: 1, who: String::new(), last: [String::new(), String::new()], dj: None, admin: false, name: String::new(), spoke: -10_000, account: None, session: None, put_at: -100_000 });
        self.next
    }

    fn signed_in(&mut self, nonce: &str, provider: &str, account: String, label: String, now: i64) -> bool {
        let Some((id, session)) = self.accounts.finish(nonce, provider, account.clone(), label, now) else {
            return false;
        };
        if !self.clients.contains_key(&id) {
            return false;
        }
        let label = self.accounts.sessions.get(&session).map(|s| s.label.clone()).unwrap_or_default();
        if let Some(c) = self.clients.get_mut(&id) {
            c.account = Some(account);
            c.session = Some(session.clone());
        }
        self.say(id, format!("account {session} {provider} {label} new"));
        true
    }

    fn say(&mut self, id: u32, text: String) {
        match self.clients.get(&id).and_then(|c| c.tx.as_ref()) {
            Some(tx) => {
                let _ = tx.send(Message::text(text));
            }
            None => self.sent.push((id, text)),
        }
    }

    fn others(&self, id: u32) -> Vec<u32> {
        let Some(room) = self.clients.get(&id).and_then(|c| c.room.clone()) else {
            return Vec::new();
        };
        self.clients.iter().filter(|(k, c)| **k != id && c.room.as_deref() == Some(room.as_str())).map(|(k, _)| *k).collect()
    }

    fn crowd(&self, code: &str) -> u32 {
        self.clients.values().filter(|c| c.room.as_deref() == Some(code)).map(|c| c.players).sum()
    }

    fn load_djs(&mut self) {
        let Some(path) = self.djs_path.clone() else {
            return;
        };
        if let Ok(text) = std::fs::read_to_string(path) {
            self.djs = text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .filter_map(|l| l.split_once('='))
                .map(|(n, k)| (clean(n, 12), k.trim().to_string()))
                .filter(|(n, k)| !n.is_empty() && !k.is_empty())
                .collect();
        }
    }

    fn save_djs(&self) -> bool {
        let Some(path) = self.djs_path.clone() else {
            return false;
        };
        let text: String = self.djs.iter().map(|(n, k)| format!("{n} = {k}\n")).collect();
        let tmp = format!("{path}.tmp");
        std::fs::write(&tmp, text).is_ok() && std::fs::rename(&tmp, &path).is_ok()
    }

    fn dj_list(&mut self, id: u32) {
        let mut names: Vec<String> = self.djs.iter().map(|(n, _)| n.clone()).collect();
        names.sort();
        self.say(id, format!("djs {}", names.join(" ")).trim_end().to_string());
    }

    fn save_credits(&self) {
        if let Some(path) = &self.credits_path {
            let mut lines: Vec<String> = self.credits.iter().map(|(n, c)| format!("{c} {n}")).collect();
            lines.sort();
            let _ = std::fs::write(path, lines.join("\n") + "\n");
        }
    }

    fn admins(&self) -> Vec<u32> {
        self.clients.iter().filter(|(_, c)| c.admin).map(|(k, _)| *k).collect()
    }

    fn brief(&mut self, id: u32) {
        let asks: Vec<String> = self.asks.iter().map(|a| format!("request {} {} {}", a.id, a.code, a.dj)).collect();
        for line in asks {
            self.say(id, line);
        }
        let mut names: Vec<(String, u32)> = self.credits.iter().map(|(n, c)| (n.clone(), *c)).collect();
        names.sort();
        self.say(id, "balances".to_string());
        for (n, c) in names {
            self.say(id, format!("balance {c} {n}"));
        }
        self.load_djs();
        self.dj_list(id);
    }

    fn off_air(&mut self, id: u32) {
        let Some(room) = self.clients.get(&id).and_then(|c| c.room.clone()) else {
            return;
        };
        if self.live.get(&room).map(|l| l.by == id).unwrap_or(false) {
            self.live.remove(&room);
            let all: Vec<u32> = self.others(id).into_iter().chain([id]).collect();
            for to in all {
                self.say(to, "live 0".to_string());
            }
        }
    }

    fn leave(&mut self, id: u32, now: i64) {
        self.off_air(id);
        for other in self.others(id) {
            self.say(other, format!("left {id}"));
        }
        if let Some(c) = self.clients.get_mut(&id) {
            if let Some(room) = c.room.take() {
                if let Some(r) = self.rooms.get_mut(&room) {
                    r.seen = now;
                }
            }
            c.last = [String::new(), String::new()];
        }
    }

    fn drop_client(&mut self, id: u32, now: i64) {
        self.leave(id, now);
        self.clients.remove(&id);
    }

    fn tick(&mut self, now: i64) {
        self.event.settle(now);
        if now > self.event.start + LENGTH_MS {
            let mut start = self.event.start;
            while start - LEAD_MS - WINDOW_MS <= now {
                start += self.every;
            }
            self.rooms.remove(&self.event.code);
            self.event = Event::new(start, &mut self.rng);
        }
        let busy: Vec<String> = self.clients.values().filter_map(|c| c.room.clone()).collect();
        self.rooms.retain(|code, r| busy.contains(code) || now - r.seen < IDLE_MS);
        let ids: Vec<u32> = self.clients.keys().copied().collect();
        for id in ids {
            let token = self.clients[&id].token;
            let (mine, rank) = self.event.mine(token);
            let left = GENERAL.saturating_sub(self.event.tickets.len());
            let line = format!("event {} {} {} {} {} {}", self.event.start - now, self.event.phase(now), mine, rank, left, self.event.info());
            self.say(id, line);
        }
    }

    fn handle(&mut self, id: u32, line: &str, now: i64) {
        if line.len() > MAX_LINE || !self.clients.contains_key(&id) {
            return;
        }
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        match cmd {
            "hello" => {
                if let Ok(token) = rest.trim().parse::<u64>() {
                    if token != 0 {
                        self.clients.get_mut(&id).unwrap().token = token;
                    }
                }
                if !self.patreon.is_empty() {
                    let line = format!("patreon {}", self.patreon);
                    self.say(id, line);
                }
                let providers = self.accounts.providers();
                if !providers.is_empty() {
                    self.say(id, format!("providers {}", providers.join(" ")));
                }
            }
            "dj" => {
                let key = rest.trim();
                if key.is_empty() {
                    return;
                }
                if let Some(c) = self.clients.get_mut(&id) {
                    c.admin = false;
                    c.dj = None;
                }
                if key == "-" {
                    return;
                }
                if self.admin_key.as_deref() == Some(key) {
                    self.clients.get_mut(&id).unwrap().admin = true;
                    let line = format!("admin ok {}", self.admin_name);
                    self.say(id, line);
                    self.brief(id);
                    return;
                }
                self.load_djs();
                match self.djs.iter().find(|(_, k)| k == key).map(|(n, _)| n.clone()) {
                    Some(name) => {
                        self.clients.get_mut(&id).unwrap().dj = Some(name.clone());
                        let credit = self.credits.get(&name).copied().unwrap_or(0);
                        self.say(id, format!("dj ok {credit} {name}"));
                        let mine: Vec<String> = self.grants.iter().filter(|(n, _)| *n == name).map(|(_, c)| c.clone()).collect();
                        for code in mine {
                            self.say(id, format!("granted {code}"));
                        }
                    }
                    None => self.say(id, "dj no".to_string()),
                }
            }
            "ask" => {
                let (Some(dj), Some(code)) = (self.clients[&id].dj.clone(), self.clients[&id].room.clone()) else {
                    return;
                };
                if self.grants.contains(&(dj.clone(), code.clone())) {
                    self.say(id, format!("granted {code}"));
                    return;
                }
                if !self.asks.iter().any(|a| a.dj == dj && a.code == code) && self.asks.len() < 200 {
                    self.next_ask += 1;
                    let ask = Ask { id: self.next_ask, dj: dj.clone(), code: code.clone() };
                    let line = format!("request {} {} {}", ask.id, ask.code, ask.dj);
                    self.asks.push(ask);
                    for admin in self.admins() {
                        self.say(admin, line.clone());
                    }
                }
                self.say(id, format!("asked {code}"));
            }
            "grant" | "deny" => {
                if !self.clients[&id].admin {
                    return;
                }
                let Some(at) = rest.trim().parse::<u32>().ok().and_then(|n| self.asks.iter().position(|a| a.id == n)) else {
                    return;
                };
                let ask = self.asks.remove(at);
                let djs: Vec<u32> = self.clients.iter().filter(|(_, c)| c.dj.as_deref() == Some(ask.dj.as_str())).map(|(k, _)| *k).collect();
                if cmd == "grant" {
                    self.grants.insert((ask.dj.clone(), ask.code.clone()));
                    let credit = {
                        let c = self.credits.entry(ask.dj.clone()).or_insert(0);
                        *c += 1;
                        *c
                    };
                    self.save_credits();
                    for dj in djs {
                        self.say(dj, format!("granted {}", ask.code));
                        self.say(dj, format!("credit {credit}"));
                    }
                } else {
                    for dj in djs {
                        self.say(dj, format!("denied {}", ask.code));
                    }
                }
                for admin in self.admins() {
                    self.say(admin, format!("done {}", ask.id));
                    self.brief(admin);
                }
            }
            "newdj" | "removedj" => {
                if !self.clients[&id].admin {
                    return;
                }
                let name: String = rest.trim().to_lowercase();
                let ok = !name.is_empty() && name.chars().count() <= 12 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
                if !ok {
                    self.say(id, "djerror names are 1 to 12 letters, numbers, - _ or .".to_string());
                    return;
                }
                if self.djs_path.is_none() {
                    self.say(id, "djerror the server has no dj list file (start it with --djs)".to_string());
                    return;
                }
                self.load_djs();
                let before = self.djs.clone();
                self.djs.retain(|(n, _)| *n != name);
                let mut made = None;
                if cmd == "newdj" {
                    let alphabet = b"abcdefghjkmnpqrstuvwxyz23456789";
                    let raw = accounts::random_hex(20);
                    let bytes: Vec<u8> = (0..20).map(|i| u8::from_str_radix(&raw[i * 2..i * 2 + 2], 16).unwrap_or(0)).collect();
                    let groups: Vec<String> = bytes.chunks(5).map(|g| g.iter().map(|b| alphabet[*b as usize % alphabet.len()] as char).collect()).collect();
                    let key = format!("mq-{}", groups.join("-"));
                    self.djs.push((name.clone(), key.clone()));
                    made = Some(key);
                } else if self.djs.len() == before.len() {
                    self.say(id, format!("djerror {name} isn't on the list"));
                    return;
                }
                if !self.save_djs() {
                    self.djs = before;
                    self.say(id, "djerror couldn't write the dj list file".to_string());
                    return;
                }
                let gone: Vec<u32> = self.clients.iter().filter(|(_, c)| c.dj.as_deref() == Some(name.as_str())).map(|(k, _)| *k).collect();
                for dj in gone {
                    self.clients.get_mut(&dj).unwrap().dj = None;
                    self.say(dj, "dj no".to_string());
                }
                if let Some(key) = made {
                    self.say(id, format!("djkey {name} {key}"));
                }
                for admin in self.admins() {
                    self.dj_list(admin);
                }
            }
            "settle" => {
                if !self.clients[&id].admin {
                    return;
                }
                let name = clean(rest, 12);
                if self.credits.remove(&name).is_some() {
                    self.save_credits();
                    let djs: Vec<u32> = self.clients.iter().filter(|(_, c)| c.dj.as_deref() == Some(name.as_str())).map(|(k, _)| *k).collect();
                    for dj in djs {
                        self.say(dj, "credit 0".to_string());
                    }
                }
                self.brief(id);
            }
            "golive" => {
                let admin = self.clients[&id].admin;
                let who = if admin { Some(self.admin_name.clone()) } else { self.clients[&id].dj.clone() };
                let (Some(dj), Some(code)) = (who, self.clients[&id].room.clone()) else {
                    return;
                };
                let (mode, url) = rest.split_once(' ').unwrap_or((rest, ""));
                let url = url.trim();
                let fine = match mode {
                    "decks" => true,
                    "audio" => url.starts_with("https://") && url.len() < 200 && !url.contains(' '),
                    _ => false,
                };
                if !fine || (self.live.contains_key(&code) && !admin) || (!admin && !self.grants.remove(&(dj.clone(), code.clone()))) {
                    self.say(id, "nolive".to_string());
                    return;
                }
                if let Some(by) = self.live.get(&code).map(|l| l.by) {
                    self.off_air(by);
                }
                let line = format!("live {id} {mode} {} {dj}", if mode == "audio" { url } else { "-" });
                self.live.insert(code, Live { by: id, line: line.clone(), mix: String::new(), look: String::new() });
                let all: Vec<u32> = self.others(id).into_iter().chain([id]).collect();
                for to in all {
                    self.say(to, line.clone());
                }
            }
            "offair" => self.off_air(id),
            "mix" | "pad" | "look" => {
                let Some(code) = self.clients[&id].room.clone() else {
                    return;
                };
                let nums: Vec<f32> = rest.split(' ').filter_map(|n| n.parse().ok()).collect();
                let fine = if cmd == "mix" {
                    nums.len() == 4 && nums[3] >= 100.0 && nums[3] <= 170.0
                } else if cmd == "look" {
                    nums.len() == 1 && nums[0] >= 0.0 && nums[0] < 4.0
                } else { nums.len() == 1 && nums[0] >= 0.0 && nums[0] < 6.0 };
                match self.live.get_mut(&code) {
                    Some(l) if l.by == id && fine => {
                        if cmd == "mix" {
                            l.mix = format!("mix {rest}");
                        }
                        if cmd == "look" {
                            l.look = format!("look {rest}");
                        }
                    }
                    _ => return,
                }
                for other in self.others(id) {
                    self.say(other, format!("{cmd} {rest}"));
                }
            }
            "say" => {
                let Some(c) = self.clients.get(&id) else {
                    return;
                };
                if c.room.is_none() || now - c.spoke < 600 {
                    return;
                }
                let text = clean(&rest.replace('|', "/"), 120);
                if text.is_empty() {
                    return;
                }
                let admin = c.admin;
                let kind = if admin { 2 } else if c.dj.is_some() { 1 } else { 0 };
                let name = if admin {
                    self.admin_name.clone()
                } else if c.name.eq_ignore_ascii_case(&self.admin_name) {
                    format!("not {}", c.name)
                } else {
                    c.name.clone()
                };
                self.clients.get_mut(&id).unwrap().spoke = now;
                if admin && (text.starts_with("/mute ") || text.starts_with("/unmute ")) {
                    let (on, who) = text.split_once(' ').map(|(c, w)| (c == "/mute", w.trim().to_lowercase())).unwrap();
                    self.muted.retain(|m| *m != who);
                    if on {
                        self.muted.push(who.clone());
                    }
                    self.say(id, format!("said 2 {name}|{} {who}", if on { "muted" } else { "unmuted" }));
                    return;
                }
                let line = format!("said {kind} {name}|{text}");
                if self.muted.contains(&name.to_lowercase()) {
                    self.say(id, line);
                    return;
                }
                let all: Vec<u32> = self.others(id).into_iter().chain([id]).collect();
                for one in all {
                    self.say(one, line.clone());
                }
            }
            "login" => {
                let url = self.accounts.begin(id, rest.trim(), now);
                self.say(id, url.map(|u| format!("loginurl {u}")).unwrap_or_else(|| "nologin".to_string()));
            }
            "resume" => {
                let found = self.accounts.sessions.get(rest.trim()).map(|s| (s.account.clone(), s.provider.clone(), s.label.clone()));
                match found {
                    Some((account, provider, label)) => {
                        let session = rest.trim().to_string();
                        if let Some(c) = self.clients.get_mut(&id) {
                            c.account = Some(account);
                            c.session = Some(session.clone());
                        }
                        self.say(id, format!("account {session} {provider} {label} old"));
                    }
                    None => self.say(id, "noaccount".to_string()),
                }
            }
            "logout" => {
                if let Some(session) = self.clients.get_mut(&id).and_then(|c| {
                    c.account = None;
                    c.session.take()
                }) {
                    self.accounts.forget(&session);
                }
                self.say(id, "noaccount".to_string());
            }
            "cloudget" => {
                if let Some(account) = self.clients.get(&id).and_then(|c| c.account.clone()) {
                    let data = self.accounts.cloud_get(&account).unwrap_or_else(|| "-".to_string());
                    self.say(id, format!("cloud {data}"));
                }
            }
            "cloudput" => {
                let Some(c) = self.clients.get(&id) else {
                    return;
                };
                if let (Some(account), true) = (c.account.clone(), now - c.put_at >= 10_000) {
                    if self.accounts.cloud_put(&account, rest.trim()) {
                        self.clients.get_mut(&id).unwrap().put_at = now;
                        self.say(id, "cloudok".to_string());
                    }
                }
            }
            "sit" => {
                let token = self.clients[&id].token;
                match self.event.sit(token, now) {
                    Some(off) => self.say(id, format!("sat {off}")),
                    None => self.say(id, "sat x".to_string()),
                }
            }
            "claim" => {
                let token = self.clients[&id].token;
                let reply = match self.event.claim(token, now) {
                    Ok(()) => format!("code {}", self.event.code),
                    Err(Refuse::Early(ms)) => format!("nocode early {ms}"),
                    Err(Refuse::List(ms)) => format!("nocode list {ms}"),
                    Err(Refuse::Full) => "nocode full".to_string(),
                };
                self.say(id, reply);
            }
            "host" => {
                let (code, info) = rest.split_once(' ').unwrap_or((rest, ""));
                if !valid_code(code) || code == self.event.code {
                    return;
                }
                if !self.rooms.contains_key(code) && self.rooms.len() < MAX_ROOMS {
                    self.rooms.insert(code.to_string(), Room { info: clean(info, 80), born: now, seen: now });
                }
            }
            "find" | "join" => {
                let (code, arg) = rest.split_once(' ').unwrap_or((rest, ""));
                if code == self.event.code {
                    if now < self.event.start {
                        self.say(id, format!("early {code} {}", self.event.start - now));
                        return;
                    }
                    if !self.rooms.contains_key(code) {
                        self.rooms.insert(code.to_string(), Room { info: self.event.info(), born: self.event.start, seen: now });
                    }
                }
                if !self.rooms.contains_key(code) {
                    self.say(id, format!("none {code}"));
                    return;
                }
                let fields: Vec<&str> = arg.splitn(4, ' ').collect();
                let players = fields.first().and_then(|n| n.parse::<u32>().ok()).unwrap_or(1).clamp(1, 2);
                let already = self.clients[&id].room.as_deref() == Some(code);
                let here = self.crowd(code) - if already { self.clients[&id].players } else { 0 };
                if here + if cmd == "join" { players } else { 1 } > ROOM_CAP {
                    self.say(id, format!("full {code}"));
                    return;
                }
                let room = self.rooms.get_mut(code).unwrap();
                room.seen = now;
                let reply = format!("room {code} {} {}", now - room.born, room.info);
                if cmd == "join" && fields.len() >= 3 {
                    self.leave(id, now);
                    let ch0 = fields[1].parse::<u32>().unwrap_or(0);
                    let ch1 = fields[2].parse::<u32>().unwrap_or(0);
                    let name = clean(fields.get(3).copied().unwrap_or(""), 12);
                    let admin = self.clients[&id].admin;
                    let badge = if admin { 2 } else { self.clients[&id].dj.is_some() as u8 };
                    let name = if admin {
                        self.admin_name.clone()
                    } else if name.is_empty() {
                        "duckling".to_string()
                    } else if name.eq_ignore_ascii_case(&self.admin_name) {
                        format!("not {name}")
                    } else {
                        name
                    };
                    let who = format!("{players} {ch0} {ch1} {badge} {name}");
                    {
                        let c = self.clients.get_mut(&id).unwrap();
                        c.room = Some(code.to_string());
                        c.players = players;
                        c.who = who.clone();
                        c.name = name;
                    }
                    for other in self.others(id) {
                        self.say(other, format!("who {id} {who}"));
                        let (their, last) = {
                            let o = &self.clients[&other];
                            (o.who.clone(), o.last.clone())
                        };
                        self.say(id, format!("who {other} {their}"));
                        for (slot, at) in last.iter().enumerate() {
                            if !at.is_empty() {
                                self.say(id, format!("peer {other} {slot} {at}"));
                            }
                        }
                    }
                }
                self.say(id, reply);
                if cmd == "join" {
                    if let Some((line, mix, look)) = self.live.get(code).map(|l| (l.line.clone(), l.mix.clone(), l.look.clone())) {
                        self.say(id, line);
                        if !mix.is_empty() {
                            self.say(id, mix);
                        }
                        if !look.is_empty() {
                            self.say(id, look);
                        }
                    }
                }
            }
            "at" => {
                let nums: Vec<f32> = rest.split(' ').filter_map(|n| n.parse().ok()).collect();
                if nums.len() != 6 || nums.iter().any(|n| !n.is_finite() || n.abs() > 1000.0) {
                    return;
                }
                let slot = if nums[0] > 0.5 { 1 } else { 0 };
                let tail = rest.split_once(' ').map(|(_, t)| t.to_string()).unwrap_or_default();
                match self.clients.get_mut(&id) {
                    Some(c) if c.room.is_some() && (slot as u32) < c.players => c.last[slot] = tail.clone(),
                    _ => return,
                }
                for other in self.others(id) {
                    self.say(other, format!("peer {id} {slot} {tail}"));
                }
            }
            "sing" => {
                let nums: Vec<u32> = rest.split(' ').filter_map(|n| n.parse().ok()).collect();
                if nums.len() == 2 && nums[0] < 2 && nums[1] < 6 {
                    for other in self.others(id) {
                        self.say(other, format!("sing {id} {} {}", nums[0], nums[1]));
                    }
                }
            }
            "leave" => self.leave(id, now),
            _ => {}
        }
    }
}

fn unix_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned());
    let port: u16 = arg("--port").or_else(|| std::env::var("PORT").ok()).and_then(|p| p.parse().ok()).unwrap_or(7777);
    let every = arg("--every").and_then(|m| m.parse::<i64>().ok()).unwrap_or(240).max(45) * 60_000;
    let now = unix_ms();
    let start = if let Some(secs) = arg("--rave-in").and_then(|s| s.parse::<i64>().ok()) {
        now + secs * 1000
    } else if let Some(at) = arg("--rave").and_then(|s| s.parse::<i64>().ok()) {
        at * 1000
    } else {
        let mut t = (now / every + 1) * every;
        while t - LEAD_MS - WINDOW_MS <= now {
            t += every;
        }
        t
    };
    let mut first = Hub::new(start, every, now as u64);
    first.djs_path = arg("--djs").or_else(|| std::env::var("MQ_DJS").ok());
    first.admin_key = arg("--admin").or_else(|| std::env::var("MQ_ADMIN").ok()).filter(|k| !k.trim().is_empty());
    if let Some(name) = arg("--admin-name").or_else(|| std::env::var("MQ_ADMIN_NAME").ok()) {
        let name = clean(&name.replace('|', ""), 16);
        if !name.is_empty() {
            first.admin_name = name;
        }
    }
    first.patreon = arg("--patreon").or_else(|| std::env::var("MQ_PATREON").ok()).unwrap_or_default();
    let credits_path = arg("--credits").or_else(|| std::env::var("MQ_CREDITS").ok()).unwrap_or_else(|| "credits.txt".to_string());
    if let Ok(text) = std::fs::read_to_string(&credits_path) {
        for line in text.lines() {
            if let Some((n, name)) = line.split_once(' ') {
                if let Ok(n) = n.parse::<u32>() {
                    first.credits.insert(name.trim().to_string(), n);
                }
            }
        }
    }
    first.credits_path = Some(credits_path);
    let env = |flag: &str, var: &str| arg(flag).or_else(|| std::env::var(var).ok()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    first.accounts.public = env("--public-url", "MQ_PUBLIC_URL").unwrap_or_default().trim_end_matches('/').to_string();
    first.accounts.dir = Some(std::path::PathBuf::from(env("--accounts", "MQ_ACCOUNTS").unwrap_or_else(|| "accounts".to_string())));
    if let (Some(id), Some(secret)) = (env("--google-id", "MQ_GOOGLE_ID"), env("--google-secret", "MQ_GOOGLE_SECRET")) {
        first.accounts.google = Some((id, secret));
    }
    if let (Some(client), Some(team), Some(key_id), Some(key)) = (env("--apple-id", "MQ_APPLE_ID"), env("--apple-team", "MQ_APPLE_TEAM"), env("--apple-key-id", "MQ_APPLE_KEY_ID"), env("--apple-key", "MQ_APPLE_KEY")) {
        match std::fs::read_to_string(&key) {
            Ok(key_pem) => first.accounts.apple = Some(accounts::Apple { client, team, key_id, key_pem }),
            Err(_) => println!("the apple key file {key} could not be read"),
        }
    }
    first.accounts.load();
    println!("sign-in: {}", if first.accounts.providers().is_empty() { "off (set --public-url)".to_string() } else { first.accounts.providers().join(", ") });
    first.load_djs();
    println!("{} dj(s) on the list, admin key {}", first.djs.len(), if first.admin_key.is_some() { "set" } else { "not set" });
    let hub = Arc::new(Mutex::new(first));
    println!("the secret rave starts in {} s", (start - now) / 1000);
    {
        let hub = hub.clone();
        tokio::spawn(async move {
            let mut each = tokio::time::interval(Duration::from_millis(500));
            loop {
                each.tick().await;
                hub.lock().unwrap().tick(unix_ms());
            }
        });
    }
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.expect("could not bind the port");
    println!("melody quack server listening on ws://{addr}");
    let _ = Instant::now();
    while let Ok((stream, _)) = listener.accept().await {
        let hub = hub.clone();
        tokio::spawn(async move {
            let _ = client(stream, hub).await;
        });
    }
}

async fn client(stream: TcpStream, hub: Arc<Mutex<Hub>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stream.set_nodelay(true)?;
    let mut peek = [0u8; 2048];
    let mut tries = 0;
    let upgrade = loop {
        let n = stream.peek(&mut peek).await?;
        let start = String::from_utf8_lossy(&peek[..n]).to_ascii_lowercase();
        if start.contains("upgrade: websocket") {
            break true;
        }
        if start.contains("\r\n\r\n") || n == peek.len() || n == 0 || tries > 40 {
            break false;
        }
        tries += 1;
        tokio::time::sleep(Duration::from_millis(15)).await;
    };
    if !upgrade {
        accounts::http(stream, hub).await;
        return Ok(());
    }
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let id = hub.lock().unwrap().connect(Some(tx));
    let writer = tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            if sink.send(m).await.is_err() {
                break;
            }
        }
    });
    while let Some(Ok(msg)) = source.next().await {
        match msg {
            Message::Text(t) => hub.lock().unwrap().handle(id, &t, unix_ms()),
            Message::Close(_) => break,
            _ => {}
        }
    }
    hub.lock().unwrap().drop_client(id, unix_ms());
    writer.abort();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 10 * 60 * 60 * 1000;

    fn hub() -> Hub {
        Hub::new(T, 240 * 60_000, 7)
    }

    fn said(h: &mut Hub, id: u32) -> Vec<String> {
        let out: Vec<String> = h.sent.iter().filter(|(to, _)| *to == id).map(|(_, t)| t.clone()).collect();
        h.sent.retain(|(to, _)| *to != id);
        out
    }

    #[test]
    fn the_thirty_closest_get_the_first_go() {
        let mut h = hub();
        let mark = T - LEAD_MS;
        let ids: Vec<u32> = (0..40).map(|_| h.connect(None)).collect();
        for (k, id) in ids.iter().enumerate() {
            let off = (k as i64 - 20) * 97;
            h.handle(*id, "sit", mark + off);
            assert_eq!(said(&mut h, *id), vec![format!("sat {off}")]);
        }
        let late = h.connect(None);
        h.handle(late, "sit", mark + WINDOW_MS + 1);
        assert_eq!(said(&mut h, late), vec!["sat x".to_string()]);

        h.handle(ids[20], "claim", mark + 5_000);
        assert!(said(&mut h, ids[20])[0].starts_with("nocode early"));

        let after = mark + WINDOW_MS + 1;
        h.tick(after);
        assert_eq!(h.event.priority.len(), 30);
        assert_eq!(h.event.priority[0], h.clients[&ids[20]].token);
        h.handle(ids[20], "claim", after);
        assert!(said(&mut h, ids[20]).iter().any(|l| l.starts_with("code ")));
        h.handle(ids[0], "claim", after);
        assert!(said(&mut h, ids[0]).iter().any(|l| l.starts_with("nocode list")));
        assert_eq!(h.event.mine(h.clients[&ids[0]].token).0, 4);
    }

    #[test]
    fn seventy_more_then_the_list_is_full() {
        let mut h = hub();
        let open = T - LEAD_MS + HEAD_START_MS + 1;
        for k in 0..GENERAL {
            let id = h.connect(None);
            h.handle(id, "claim", open);
            assert!(said(&mut h, id).iter().any(|l| l.starts_with("code ")), "ticket {k}");
        }
        let unlucky = h.connect(None);
        h.handle(unlucky, "claim", open);
        assert_eq!(said(&mut h, unlucky), vec!["nocode full".to_string()]);
    }

    #[test]
    fn doors_open_on_time_and_the_chapel_holds_a_hundred() {
        let mut h = hub();
        let code = h.event.code.clone();
        let early = h.connect(None);
        h.handle(early, &format!("find {code}"), T - 1000);
        assert_eq!(said(&mut h, early), vec![format!("early {code} 1000")]);
        for k in 0..50 {
            let id = h.connect(None);
            h.handle(id, &format!("join {code} 2 0 4 pair {k}"), T + 10);
            assert!(said(&mut h, id).iter().any(|l| l.starts_with("room ")), "pair {k}");
        }
        let extra = h.connect(None);
        h.handle(extra, &format!("join {code} 1 0 0 late"), T + 20);
        assert_eq!(said(&mut h, extra), vec![format!("full {code}")]);
    }

    #[test]
    fn the_admin_hands_out_and_takes_back_dj_keys() {
        let mut h = hub();
        let path = std::env::temp_dir().join(format!("mq-djs-{}.txt", accounts::random_hex(6)));
        std::fs::write(&path, "priori = k1\n").unwrap();
        h.djs_path = Some(path.to_string_lossy().to_string());
        h.admin_key = Some("boss".to_string());
        let (admin, fan) = (h.connect(None), h.connect(None));
        h.handle(admin, "dj boss", 0);
        assert!(said(&mut h, admin).contains(&"djs priori".to_string()));
        h.handle(fan, "newdj sneaky", 0);
        assert!(said(&mut h, fan).is_empty());
        h.handle(admin, "newdj Bad Name!", 0);
        assert!(said(&mut h, admin)[0].starts_with("djerror"));
        h.handle(admin, "newdj Cayennepfefa", 0);
        let got = said(&mut h, admin);
        let key = got[0].strip_prefix("djkey cayennepfefa ").unwrap().to_string();
        assert!(key.starts_with("mq-") && key.len() == 26);
        assert_eq!(got[1], "djs cayennepfefa priori");
        assert!(std::fs::read_to_string(&path).unwrap().contains(&format!("cayennepfefa = {key}")));
        h.handle(fan, &format!("dj {key}"), 0);
        assert_eq!(said(&mut h, fan), vec!["dj ok 0 cayennepfefa".to_string()]);
        h.handle(admin, "newdj cayennepfefa", 0);
        let again = said(&mut h, admin)[0].clone();
        assert_ne!(again, format!("djkey cayennepfefa {key}"));
        assert_eq!(said(&mut h, fan), vec!["dj no".to_string()]);
        h.handle(admin, "removedj priori", 0);
        assert_eq!(said(&mut h, admin), vec!["djs cayennepfefa".to_string()]);
        h.handle(admin, "removedj priori", 0);
        assert!(said(&mut h, admin)[0].starts_with("djerror"));
        assert!(!std::fs::read_to_string(&path).unwrap().contains("priori"));
    }

    #[test]
    fn players_sign_in_and_keep_their_save_in_the_cloud() {
        let mut h = hub();
        let dir = std::env::temp_dir().join(format!("mq-hub-{}", accounts::random_hex(6)));
        h.accounts.public = "https://rave.example".to_string();
        h.accounts.dir = Some(dir);
        h.accounts.load();
        let (a, b) = (h.connect(None), h.connect(None));
        h.handle(a, "hello 1", 0);
        assert!(said(&mut h, a).contains(&"providers steam".to_string()));
        h.handle(a, "login apple", 0);
        assert_eq!(said(&mut h, a), vec!["nologin".to_string()]);
        h.handle(a, "cloudput QUJD", 0);
        h.handle(a, "cloudget", 0);
        assert!(said(&mut h, a).is_empty());
        h.handle(a, "login steam", 0);
        let url = said(&mut h, a)[0].strip_prefix("loginurl ").unwrap().to_string();
        let nonce = url.split("n=").nth(1).unwrap().to_string();
        assert!(!h.signed_in(&nonce, "steam", "steam:5".to_string(), "steam-5".to_string(), 30 * 60 * 1000));
        h.handle(a, "login steam", 0);
        let url = said(&mut h, a)[0].strip_prefix("loginurl ").unwrap().to_string();
        let nonce = url.split("n=").nth(1).unwrap().to_string();
        assert!(h.signed_in(&nonce, "steam", "steam:5".to_string(), "steam-5".to_string(), 1));
        let line = said(&mut h, a)[0].clone();
        assert!(line.starts_with("account ") && line.ends_with(" steam steam-5 new"));
        let session = line.split(' ').nth(1).unwrap().to_string();
        h.handle(a, "cloudget", 2);
        assert_eq!(said(&mut h, a), vec!["cloud -".to_string()]);
        h.handle(a, "cloudput QUJD", 20_000);
        h.handle(a, "cloudput REVG", 21_000);
        assert_eq!(said(&mut h, a), vec!["cloudok".to_string()]);
        h.handle(b, &format!("resume {session}"), 22_000);
        assert_eq!(said(&mut h, b), vec![format!("account {session} steam steam-5 old")]);
        h.handle(b, "cloudget", 22_000);
        assert_eq!(said(&mut h, b), vec!["cloud QUJD".to_string()]);
        h.handle(b, "logout", 23_000);
        h.handle(a, &format!("resume {session}"), 24_000);
        assert_eq!(said(&mut h, b), vec!["noaccount".to_string()]);
        assert_eq!(said(&mut h, a), vec!["noaccount".to_string()]);
    }

    #[tokio::test]
    async fn the_server_answers_browsers_and_websockets_on_one_port() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut first = hub();
        first.accounts.public = "https://rave.example".to_string();
        first.accounts.dir = Some(std::env::temp_dir().join(format!("mq-http-{}", accounts::random_hex(6))));
        first.accounts.load();
        let nonce = first.accounts.begin(99, "steam", unix_ms()).unwrap().split("n=").nth(1).unwrap().to_string();
        let hub = Arc::new(Mutex::new(first));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let h2 = hub.clone();
        tokio::spawn(async move {
            while let Ok((s, _)) = listener.accept().await {
                let h = h2.clone();
                tokio::spawn(async move {
                    let _ = client(s, h).await;
                });
            }
        });
        let fetch = |path: String| async move {
            let mut s = TcpStream::connect(addr).await.unwrap();
            s.write_all(format!("GET {path} HTTP/1.1\r\nhost: x\r\n\r\n").as_bytes()).await.unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).await.unwrap();
            out
        };
        let r = fetch(format!("/login/steam?n={nonce}")).await;
        assert!(r.starts_with("HTTP/1.1 302"));
        assert!(r.contains("location: https://steamcommunity.com/openid/login?"));
        assert!(fetch("/login/steam?n=nope".to_string()).await.contains("link expired"));
        assert!(fetch(format!("/auth/steam?n={nonce}&openid.return_to=https%3A%2F%2Fevil.example")).await.contains("another page"));
        assert!(fetch("/".to_string()).await.contains("rave server is running"));
        let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/")).await.unwrap();
        ws.send(Message::text("hello 5")).await.unwrap();
        let mut got = Vec::new();
        while let Ok(Some(Ok(m))) = tokio::time::timeout(Duration::from_millis(300), ws.next()).await {
            got.push(m.to_string());
        }
        assert!(got.contains(&"providers steam".to_string()));
    }

    #[test]
    fn a_dj_asks_the_admin_grants_and_the_room_hears_it() {
        let mut h = hub();
        h.djs = vec![("priori".to_string(), "k1".to_string())];
        h.admin_key = Some("boss".to_string());
        let (admin, dj, fan) = (h.connect(None), h.connect(None), h.connect(None));
        h.handle(admin, "dj boss", 0);
        assert!(said(&mut h, admin).contains(&"admin ok admin".to_string()));
        h.handle(fan, "dj wrong", 0);
        assert_eq!(said(&mut h, fan), vec!["dj no".to_string()]);
        h.handle(dj, "dj k1", 0);
        assert_eq!(said(&mut h, dj), vec!["dj ok 0 priori".to_string()]);
        h.handle(dj, "host ACD-EFG-HJK 1|132|9|duck mass", 0);
        h.handle(dj, "join ACD-EFG-HJK 1 0 0 priori", 1);
        h.handle(fan, "join ACD-EFG-HJK 1 2 0 mina", 2);
        assert!(said(&mut h, fan).contains(&format!("who {dj} 1 0 0 1 priori")));
        said(&mut h, dj);
        h.handle(dj, "golive decks", 3);
        assert_eq!(said(&mut h, dj), vec!["nolive".to_string()]);
        h.handle(fan, "ask", 3);
        assert!(said(&mut h, fan).is_empty());
        h.handle(dj, "ask", 4);
        assert_eq!(said(&mut h, dj), vec!["asked ACD-EFG-HJK".to_string()]);
        assert!(said(&mut h, admin).contains(&"request 1 ACD-EFG-HJK priori".to_string()));
        h.handle(fan, "grant 1", 5);
        assert!(said(&mut h, dj).is_empty());
        h.handle(admin, "grant 1", 5);
        assert_eq!(said(&mut h, dj), vec!["granted ACD-EFG-HJK".to_string(), "credit 1".to_string()]);
        assert!(said(&mut h, admin).contains(&"balance 1 priori".to_string()));
        h.handle(dj, "golive decks", 6);
        let line = format!("live {dj} decks - priori");
        assert_eq!(said(&mut h, dj), vec![line.clone()]);
        assert_eq!(said(&mut h, fan), vec![line.clone()]);
        h.handle(dj, "mix 1 0 2 140", 7);
        h.handle(dj, "pad 3", 7);
        h.handle(fan, "pad 3", 7);
        h.handle(dj, "look 2", 7);
        h.handle(dj, "look 9", 7);
        h.handle(fan, "look 1", 7);
        assert_eq!(said(&mut h, fan), vec!["mix 1 0 2 140".to_string(), "pad 3".to_string(), "look 2".to_string()]);
        h.admin_name = "boss9000".to_string();
        h.handle(fan, "say hello | all", 1000);
        h.handle(fan, "say too fast", 1100);
        assert_eq!(said(&mut h, dj), vec!["said 0 mina|hello / all".to_string()]);
        h.handle(dj, "say tune", 2000);
        h.handle(admin, "say not in a room", 2000);
        h.handle(admin, "join ACD-EFG-HJK 1 0 0 whoever", 2100);
        h.handle(admin, "say welcome", 3000);
        h.handle(admin, "say /mute mina", 4000);
        h.handle(fan, "say can you hear me", 5000);
        let heard = said(&mut h, dj);
        assert!(heard.contains(&"said 1 priori|tune".to_string()) && heard.contains(&"said 2 boss9000|welcome".to_string()));
        assert!(!heard.iter().any(|l| l.contains("can you hear me") || l.contains("muted")));
        let own: Vec<String> = said(&mut h, fan).into_iter().filter(|l| l.starts_with("said")).collect();
        assert_eq!(own, vec!["said 0 mina|hello / all".to_string(), "said 1 priori|tune".to_string(), "said 2 boss9000|welcome".to_string(), "said 0 mina|can you hear me".to_string()]);
        said(&mut h, admin);
        let late = h.connect(None);
        h.handle(late, "join ACD-EFG-HJK 1 1 0 jo", 8);
        let got = said(&mut h, late);
        assert!(got.contains(&line) && got.contains(&"mix 1 0 2 140".to_string()) && got.contains(&"look 2".to_string()));
        h.drop_client(dj, 9);
        assert!(said(&mut h, fan).contains(&"live 0".to_string()));
        let again = h.connect(None);
        h.handle(again, "dj k1", 10);
        h.handle(again, "join ACD-EFG-HJK 1 0 0 priori", 11);
        said(&mut h, again);
        h.handle(again, "golive audio https://example.com/live", 12);
        assert_eq!(said(&mut h, again), vec!["nolive".to_string()]);
        let boss = h.connect(None);
        h.handle(boss, "dj boss", 20);
        let room_fan = h.connect(None);
        h.handle(room_fan, "join ACD-EFG-HJK 1 2 0 zed", 20);
        h.handle(boss, "join ACD-EFG-HJK 1 0 0 whoever", 20);
        h.grants.insert(("priori".to_string(), "ACD-EFG-HJK".to_string()));
        h.handle(again, "golive decks", 20);
        said(&mut h, room_fan);
        said(&mut h, again);
        h.handle(boss, "golive audio https://stream.example/live.mp3", 21);
        let took = said(&mut h, room_fan);
        assert_eq!(took[0], "live 0");
        assert!(took[1].ends_with("audio https://stream.example/live.mp3 boss9000"), "{took:?}");
        h.handle(again, "mix 1 1 0 130", 22);
        assert!(said(&mut h, room_fan).is_empty());
    }

    #[test]
    fn friends_see_both_players_and_names() {
        let mut h = hub();
        let (a, b) = (h.connect(None), h.connect(None));
        h.handle(a, "host ACD-EFG-HJK 1|132|9|duck mass", 0);
        h.handle(a, "join ACD-EFG-HJK 2 0 4 ky", 10);
        h.handle(a, "at 1 3.00 4.00 0.00 -1.00 0", 20);
        said(&mut h, a);
        h.handle(b, "join ACD-EFG-HJK 1 2 0 mina", 30);
        let got = said(&mut h, b);
        assert!(got.contains(&format!("who {a} 2 0 4 0 ky")));
        assert!(got.contains(&format!("peer {a} 1 3.00 4.00 0.00 -1.00 0")));
        assert!(said(&mut h, a).contains(&format!("who {b} 1 2 0 0 mina")));
        h.handle(b, "sing 0 3", 40);
        assert_eq!(said(&mut h, a), vec![format!("sing {b} 0 3")]);
        h.drop_client(b, 50);
        assert_eq!(said(&mut h, a), vec![format!("left {b}")]);
    }
}
