use futures_util::{SinkExt, StreamExt};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
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

struct Client {
    tx: Option<mpsc::UnboundedSender<Message>>,
    token: u64,
    room: Option<String>,
    players: u32,
    who: String,
    last: [String; 2],
}

struct Hub {
    rooms: HashMap<String, Room>,
    clients: HashMap<u32, Client>,
    next: u32,
    event: Event,
    every: i64,
    rng: u64,
    sent: Vec<(u32, String)>,
}

impl Hub {
    fn new(start: i64, every: i64, seed: u64) -> Hub {
        let mut rng = seed;
        let event = Event::new(start, &mut rng);
        Hub { rooms: HashMap::new(), clients: HashMap::new(), next: 0, event, every, rng, sent: Vec::new() }
    }

    fn connect(&mut self, tx: Option<mpsc::UnboundedSender<Message>>) -> u32 {
        self.next += 1;
        let token = next_rand(&mut self.rng);
        self.clients.insert(self.next, Client { tx, token, room: None, players: 1, who: String::new(), last: [String::new(), String::new()] });
        self.next
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

    fn leave(&mut self, id: u32, now: i64) {
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
                    let who = format!("{players} {ch0} {ch1} {}", if name.is_empty() { "duckling".to_string() } else { name });
                    {
                        let c = self.clients.get_mut(&id).unwrap();
                        c.room = Some(code.to_string());
                        c.players = players;
                        c.who = who.clone();
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
    let hub = Arc::new(Mutex::new(Hub::new(start, every, now as u64)));
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
    fn friends_see_both_players_and_names() {
        let mut h = hub();
        let (a, b) = (h.connect(None), h.connect(None));
        h.handle(a, "host ACD-EFG-HJK 1|132|9|duck mass", 0);
        h.handle(a, "join ACD-EFG-HJK 2 0 4 ky", 10);
        h.handle(a, "at 1 3.00 4.00 0.00 -1.00 0", 20);
        said(&mut h, a);
        h.handle(b, "join ACD-EFG-HJK 1 2 0 mina", 30);
        let got = said(&mut h, b);
        assert!(got.contains(&format!("who {a} 2 0 4 ky")));
        assert!(got.contains(&format!("peer {a} 1 3.00 4.00 0.00 -1.00 0")));
        assert!(said(&mut h, a).contains(&format!("who {b} 1 2 0 mina")));
        h.handle(b, "sing 0 3", 40);
        assert_eq!(said(&mut h, a), vec![format!("sing {b} 0 3")]);
        h.drop_client(b, 50);
        assert_eq!(said(&mut h, a), vec![format!("left {b}")]);
    }
}
