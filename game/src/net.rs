//! Talking to the server, or to an in-process hub when playing offline.

use mq_shared::{decode, encode, ClientMsg, Hub, HubConfig, Payload, ServerMsg, PROTOCOL_VERSION};
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

pub fn wall_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Connecting,
    Online,
    Offline,
    Lost(String),
}

enum ToNet {
    Msg(ClientMsg),
    Audio(Vec<u8>),
}

enum FromNet {
    Connected,
    Msg(ServerMsg),
    Audio(Vec<u8>),
    Lost(String),
}

enum Backend {
    Online {
        tx: Sender<ToNet>,
        rx: Receiver<FromNet>,
    },
    Offline {
        hub: Hub,
        me: u32,
        last_tick: f64,
        inbox: Vec<ServerMsg>,
    },
}

/// Cheap to clone: hands DJ audio packets to the network from the capture
/// thread.
#[derive(Clone)]
pub struct AudioUplink(Option<Sender<ToNet>>);

impl AudioUplink {
    pub fn send(&self, packet: Vec<u8>) {
        if let Some(tx) = &self.0 {
            let _ = tx.send(ToNet::Audio(packet));
        }
    }
}

pub struct Net {
    backend: Backend,
    pub status: Status,
    /// server_ms - wall_ms, estimated from pings.
    pub clock_offset: f64,
    best_rtt: f64,
    last_ping: f64,
    pub audio_in: Option<Sender<Vec<u8>>>,
}

impl Net {
    pub fn online(url: &str) -> Net {
        let (to_tx, to_rx) = channel::<ToNet>();
        let (from_tx, from_rx) = channel::<FromNet>();
        let url = url.trim().to_string();
        thread::Builder::new()
            .name("net".into())
            .spawn(move || net_thread(url, to_rx, from_tx))
            .expect("spawn net thread");
        Net {
            backend: Backend::Online { tx: to_tx, rx: from_rx },
            status: Status::Connecting,
            clock_offset: 0.0,
            best_rtt: f64::MAX,
            last_ping: 0.0,
            audio_in: None,
        }
    }

    pub fn offline() -> Net {
        let cfg = HubConfig {
            any_key_is_dj: true,
            motd: "offline. the chapel is all yours.".into(),
            ..HubConfig::default()
        };
        let mut hub = Hub::new(cfg, wall_ms() as u64);
        let me = hub.connect();
        Net {
            backend: Backend::Offline {
                hub,
                me,
                last_tick: 0.0,
                inbox: Vec::new(),
            },
            status: Status::Offline,
            clock_offset: 0.0,
            best_rtt: 0.0,
            last_ping: 0.0,
            audio_in: None,
        }
    }

    pub fn is_offline(&self) -> bool {
        matches!(self.backend, Backend::Offline { .. })
    }

    pub fn server_ms(&self) -> f64 {
        wall_ms() + self.clock_offset
    }

    pub fn hello(&mut self, name: &str, avatar: u8, dj_key: &str) {
        self.send(ClientMsg::Hello {
            name: name.to_string(),
            avatar,
            dj_key: Some(dj_key.to_string()).filter(|k| !k.trim().is_empty()),
            version: PROTOCOL_VERSION,
        });
    }

    pub fn send(&mut self, msg: ClientMsg) {
        match &mut self.backend {
            Backend::Online { tx, .. } => {
                let _ = tx.send(ToNet::Msg(msg));
            }
            Backend::Offline { hub, me, inbox, .. } => {
                let out = hub.handle(*me, msg, wall_ms() as u64);
                collect(out, *me, inbox);
            }
        }
    }

    pub fn uplink(&self) -> AudioUplink {
        match &self.backend {
            Backend::Online { tx, .. } => AudioUplink(Some(tx.clone())),
            Backend::Offline { .. } => AudioUplink(None),
        }
    }

    /// Drain incoming messages. Call once per frame.
    pub fn poll(&mut self) -> Vec<ServerMsg> {
        let now = wall_ms();
        let mut msgs = Vec::new();
        match &mut self.backend {
            Backend::Online { tx, rx } => {
                loop {
                    match rx.try_recv() {
                        Ok(FromNet::Connected) => {
                            self.status = Status::Online;
                        }
                        Ok(FromNet::Msg(m)) => msgs.push(m),
                        Ok(FromNet::Audio(p)) => {
                            if let Some(a) = &self.audio_in {
                                let _ = a.send(p);
                            }
                        }
                        Ok(FromNet::Lost(why)) => {
                            self.status = Status::Lost(why);
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            if !matches!(self.status, Status::Lost(_)) {
                                self.status = Status::Lost("connection closed".into());
                            }
                            break;
                        }
                    }
                }
                if self.status == Status::Online && now - self.last_ping > 2000.0 {
                    self.last_ping = now;
                    let _ = tx.send(ToNet::Msg(ClientMsg::Ping { t: now }));
                }
            }
            Backend::Offline {
                hub,
                me,
                last_tick,
                inbox,
            } => {
                if now - *last_tick > 66.0 {
                    *last_tick = now;
                    let out = hub.tick(now as u64);
                    collect(out, *me, inbox);
                }
                msgs.append(inbox);
            }
        }
        for m in &msgs {
            match m {
                ServerMsg::Welcome { server_ms, .. } if !self.is_offline() => {
                    if self.best_rtt == f64::MAX {
                        self.clock_offset = *server_ms as f64 - now;
                    }
                }
                ServerMsg::Pong { t, server_ms } => {
                    let rtt = now - t;
                    // Trust the fastest round trip we've seen (least queueing).
                    if rtt <= self.best_rtt * 1.2 {
                        self.best_rtt = self.best_rtt.min(rtt);
                        self.clock_offset = *server_ms as f64 + rtt / 2.0 - now;
                    }
                }
                _ => {}
            }
        }
        msgs
    }
}

fn collect(out: mq_shared::Out, me: u32, inbox: &mut Vec<ServerMsg>) {
    for (id, p) in out {
        if id == me {
            if let Payload::Msg(m) = p {
                inbox.push(m);
            }
        }
    }
}

fn set_timeout(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>, d: Option<Duration>) {
    let sock: Option<&TcpStream> = match ws.get_mut() {
        MaybeTlsStream::Plain(s) => Some(s),
        MaybeTlsStream::Rustls(s) => Some(s.get_ref()),
        _ => None,
    };
    if let Some(s) = sock {
        let _ = s.set_read_timeout(d);
        let _ = s.set_nodelay(true);
    }
}

fn net_thread(url: String, rx: Receiver<ToNet>, tx: Sender<FromNet>) {
    let url = if url.contains("://") {
        url
    } else {
        format!("ws://{url}")
    };
    let mut ws = match tungstenite::connect(url.as_str()) {
        Ok((ws, _)) => ws,
        Err(e) => {
            let _ = tx.send(FromNet::Lost(format!("couldn't reach {url} ({e})")));
            return;
        }
    };
    set_timeout(&mut ws, Some(Duration::from_millis(4)));
    let _ = tx.send(FromNet::Connected);
    let mut last_flush = Instant::now();
    loop {
        // Outgoing.
        let mut wrote = false;
        loop {
            match rx.try_recv() {
                Ok(ToNet::Msg(m)) => {
                    let _ = ws.write(Message::text(encode(&m)));
                    wrote = true;
                }
                Ok(ToNet::Audio(p)) => {
                    let _ = ws.write(Message::binary(p));
                    wrote = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = ws.close(None);
                    return;
                }
            }
        }
        if wrote || last_flush.elapsed() > Duration::from_millis(50) {
            last_flush = Instant::now();
            if let Err(e) = ws.flush() {
                if !is_wouldblock(&e) {
                    let _ = tx.send(FromNet::Lost(format!("connection lost ({e})")));
                    return;
                }
            }
        }
        // Incoming.
        loop {
            match ws.read() {
                Ok(Message::Text(t)) => {
                    if let Some(m) = decode::<ServerMsg>(&t) {
                        if tx.send(FromNet::Msg(m)).is_err() {
                            return;
                        }
                    }
                }
                Ok(Message::Binary(b)) => {
                    let _ = tx.send(FromNet::Audio(b.to_vec()));
                }
                Ok(Message::Close(_)) => {
                    let _ = tx.send(FromNet::Lost("the server closed the connection".into()));
                    return;
                }
                Ok(_) => {}
                Err(e) if is_wouldblock(&e) => break,
                Err(e) => {
                    let _ = tx.send(FromNet::Lost(format!("connection lost ({e})")));
                    return;
                }
            }
        }
    }
}

fn is_wouldblock(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(io) if io.kind() == ErrorKind::WouldBlock || io.kind() == ErrorKind::TimedOut)
}
