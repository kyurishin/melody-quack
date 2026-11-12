use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

const ALPHABET: &str = "ACDEFGHJKMNPQRTUVWXY3479";
const IDLE: Duration = Duration::from_secs(6 * 60 * 60);
const MAX_ROOMS: usize = 2000;
const MAX_LINE: usize = 240;

struct Room {
    info: String,
    born: Instant,
    seen: Instant,
}

struct Client {
    tx: mpsc::UnboundedSender<Message>,
    room: Option<String>,
    character: u32,
    last: String,
}

#[derive(Default)]
struct Hub {
    rooms: HashMap<String, Room>,
    clients: HashMap<u32, Client>,
    next: u32,
}

fn valid_code(code: &str) -> bool {
    let b: Vec<char> = code.chars().collect();
    b.len() == 11 && b.iter().enumerate().all(|(i, c)| if i == 3 || i == 7 { *c == '-' } else { ALPHABET.contains(*c) })
}

fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(80).collect()
}

impl Hub {
    fn say(&self, id: u32, text: String) {
        if let Some(c) = self.clients.get(&id) {
            let _ = c.tx.send(Message::text(text));
        }
    }

    fn others(&self, id: u32) -> Vec<u32> {
        let Some(room) = self.clients.get(&id).and_then(|c| c.room.clone()) else {
            return Vec::new();
        };
        self.clients.iter().filter(|(k, c)| **k != id && c.room.as_deref() == Some(room.as_str())).map(|(k, _)| *k).collect()
    }

    fn leave(&mut self, id: u32) {
        for other in self.others(id) {
            self.say(other, format!("left {id}"));
        }
        if let Some(c) = self.clients.get_mut(&id) {
            if let Some(room) = c.room.take() {
                if let Some(r) = self.rooms.get_mut(&room) {
                    r.seen = Instant::now();
                }
            }
            c.last.clear();
        }
    }

    fn sweep(&mut self) {
        let busy: Vec<String> = self.clients.values().filter_map(|c| c.room.clone()).collect();
        self.rooms.retain(|code, r| busy.contains(code) || r.seen.elapsed() < IDLE);
    }

    fn handle(&mut self, id: u32, line: &str) {
        if line.len() > MAX_LINE {
            return;
        }
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        match cmd {
            "host" => {
                let (code, info) = rest.split_once(' ').unwrap_or((rest, ""));
                if !valid_code(code) {
                    return;
                }
                if !self.rooms.contains_key(code) {
                    if self.rooms.len() >= MAX_ROOMS {
                        self.sweep();
                    }
                    if self.rooms.len() >= MAX_ROOMS {
                        return;
                    }
                    self.rooms.insert(code.to_string(), Room { info: clean(info), born: Instant::now(), seen: Instant::now() });
                }
            }
            "find" | "join" => {
                let (code, arg) = rest.split_once(' ').unwrap_or((rest, ""));
                let Some(room) = self.rooms.get_mut(code) else {
                    self.say(id, format!("none {code}"));
                    return;
                };
                room.seen = Instant::now();
                let reply = format!("room {code} {} {}", room.born.elapsed().as_millis(), room.info);
                if cmd == "join" {
                    self.leave(id);
                    if let Some(c) = self.clients.get_mut(&id) {
                        c.room = Some(code.to_string());
                        c.character = arg.trim().parse().unwrap_or(0);
                    }
                    for other in self.others(id) {
                        if let Some(o) = self.clients.get(&other) {
                            if !o.last.is_empty() {
                                self.say(id, format!("peer {other} {} {}", o.character, o.last));
                            }
                        }
                    }
                }
                self.say(id, reply);
            }
            "at" => {
                let nums: Vec<f32> = rest.split(' ').filter_map(|n| n.parse().ok()).collect();
                if nums.len() != 5 || nums.iter().any(|n| !n.is_finite() || n.abs() > 1000.0) {
                    return;
                }
                let character = match self.clients.get_mut(&id) {
                    Some(c) if c.room.is_some() => {
                        c.last = rest.to_string();
                        c.character
                    }
                    _ => return,
                };
                for other in self.others(id) {
                    self.say(other, format!("peer {id} {character} {rest}"));
                }
            }
            "sing" => {
                let Ok(note) = rest.trim().parse::<u32>() else {
                    return;
                };
                if note < 6 {
                    for other in self.others(id) {
                        self.say(other, format!("sing {id} {note}"));
                    }
                }
            }
            "leave" => self.leave(id),
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| std::env::var("PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(7777);
    let hub = Arc::new(Mutex::new(Hub::default()));
    {
        let hub = hub.clone();
        tokio::spawn(async move {
            let mut every = tokio::time::interval(Duration::from_secs(600));
            loop {
                every.tick().await;
                hub.lock().unwrap().sweep();
            }
        });
    }
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.expect("could not bind the port");
    println!("melody quack server listening on ws://{addr}");
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
    let id = {
        let mut h = hub.lock().unwrap();
        h.next += 1;
        let id = h.next;
        h.clients.insert(id, Client { tx, room: None, character: 0, last: String::new() });
        id
    };
    let writer = tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            if sink.send(m).await.is_err() {
                break;
            }
        }
    });
    while let Some(Ok(msg)) = source.next().await {
        match msg {
            Message::Text(t) => hub.lock().unwrap().handle(id, &t),
            Message::Close(_) => break,
            _ => {}
        }
    }
    {
        let mut h = hub.lock().unwrap();
        h.leave(id);
        h.clients.remove(&id);
    }
    writer.abort();
    Ok(())
}
