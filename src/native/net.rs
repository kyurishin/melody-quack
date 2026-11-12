use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

struct Conn {
    tx: Sender<String>,
    rx: Receiver<String>,
    state: Arc<AtomicI32>,
}

static CONN: Mutex<Option<Conn>> = Mutex::new(None);

pub fn server() -> Option<String> {
    if let Ok(url) = std::env::var("MELODY_QUACK_SERVER") {
        if !url.trim().is_empty() {
            return Some(url.trim().to_string());
        }
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    let text = std::fs::read_to_string(std::path::Path::new(&home).join(".melody-quack").join("server.txt")).ok()?;
    let url = text.trim();
    if url.is_empty() { None } else { Some(url.to_string()) }
}

pub fn open(url: &str) {
    let (to_tx, to_rx) = channel::<String>();
    let (from_tx, from_rx) = channel::<String>();
    let state = Arc::new(AtomicI32::new(1));
    let shared = state.clone();
    let url = if url.contains("://") { url.to_string() } else { format!("ws://{url}") };
    std::thread::spawn(move || run(url, to_rx, from_tx, shared));
    *CONN.lock().unwrap() = Some(Conn { tx: to_tx, rx: from_rx, state });
}

pub fn state() -> i32 {
    CONN.lock().unwrap().as_ref().map(|c| c.state.load(Ordering::Relaxed)).unwrap_or(0)
}

pub fn send(text: &str) {
    if let Some(c) = CONN.lock().unwrap().as_ref() {
        if c.state.load(Ordering::Relaxed) == 2 {
            let _ = c.tx.send(text.to_string());
        }
    }
}

pub fn recv() -> Option<String> {
    CONN.lock().unwrap().as_ref().and_then(|c| c.rx.try_recv().ok())
}

fn would_block(e: &tungstenite::Error) -> bool {
    matches!(e, tungstenite::Error::Io(io) if io.kind() == ErrorKind::WouldBlock || io.kind() == ErrorKind::TimedOut)
}

fn run(url: String, rx: Receiver<String>, tx: Sender<String>, state: Arc<AtomicI32>) {
    let mut ws: WebSocket<MaybeTlsStream<TcpStream>> = match tungstenite::connect(url.as_str()) {
        Ok((ws, _)) => ws,
        Err(_) => {
            state.store(-1, Ordering::Relaxed);
            return;
        }
    };
    let sock = match ws.get_mut() {
        MaybeTlsStream::Plain(s) => Some(&*s),
        MaybeTlsStream::Rustls(s) => Some(s.get_ref()),
        _ => None,
    };
    if let Some(s) = sock {
        let _ = s.set_read_timeout(Some(Duration::from_millis(5)));
        let _ = s.set_nodelay(true);
    }
    state.store(2, Ordering::Relaxed);
    loop {
        let mut wrote = false;
        loop {
            match rx.try_recv() {
                Ok(text) => {
                    let _ = ws.write(Message::text(text));
                    wrote = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = ws.close(None);
                    return;
                }
            }
        }
        if wrote {
            if let Err(e) = ws.flush() {
                if !would_block(&e) {
                    state.store(-1, Ordering::Relaxed);
                    return;
                }
            }
        }
        loop {
            match ws.read() {
                Ok(Message::Text(t)) => {
                    if tx.send(t.to_string()).is_err() {
                        return;
                    }
                }
                Ok(Message::Close(_)) => {
                    state.store(-1, Ordering::Relaxed);
                    return;
                }
                Ok(_) => {}
                Err(e) if would_block(&e) => break,
                Err(_) => {
                    state.store(-1, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
}
