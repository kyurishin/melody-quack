//! Melody Quack relay server.
//!
//!   mq-server [--port 7777] [--djs djs.txt]
//!
//! Environment: `PORT` (overrides the default port), `MQ_DJS` (path to the DJ
//! list), `MQ_MOTD`. The DJ list has one `name = key` per line; players who
//! enter that key on the title screen join as that DJ with the purple dot.

use futures_util::{SinkExt, StreamExt};
use mq_shared::{decode, encode, ClientMsg, Hub, HubConfig, Payload, DEFAULT_PORT};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

type Senders = Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Message>>>>;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn deliver(senders: &Senders, out: mq_shared::Out) {
    if out.is_empty() {
        return;
    }
    let senders = senders.lock().unwrap();
    for (id, payload) in out {
        if let Some(tx) = senders.get(&id) {
            let msg = match payload {
                Payload::Msg(m) => Message::text(encode(&m)),
                Payload::Audio(data) => Message::binary(data.as_ref().clone()),
            };
            let _ = tx.send(msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1).cloned())
    };
    let port: u16 = arg("--port")
        .or_else(|| std::env::var("PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let mut cfg = HubConfig::default();
    if let Some(path) = arg("--djs").or_else(|| std::env::var("MQ_DJS").ok()) {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                cfg.djs = HubConfig::parse_djs(&text);
                println!("loaded {} dj(s) from {path}", cfg.djs.len());
            }
            Err(e) => eprintln!("couldn't read dj list {path}: {e}"),
        }
    }
    if let Ok(motd) = std::env::var("MQ_MOTD") {
        cfg.motd = motd;
    }

    let hub = Arc::new(Mutex::new(Hub::new(cfg, now_ms())));
    let senders: Senders = Arc::new(Mutex::new(HashMap::new()));

    {
        let (hub, senders) = (hub.clone(), senders.clone());
        tokio::spawn(async move {
            let mut iv = tokio::time::interval(Duration::from_millis(66));
            loop {
                iv.tick().await;
                let out = hub.lock().unwrap().tick(now_ms());
                deliver(&senders, out);
            }
        });
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await.expect("couldn't bind port");
    println!("melody quack server listening on ws://{addr}");

    while let Ok((stream, peer)) = listener.accept().await {
        let (hub, senders) = (hub.clone(), senders.clone());
        tokio::spawn(async move {
            if let Err(e) = client(stream, hub, senders).await {
                eprintln!("{peer}: {e}");
            }
        });
    }
}

async fn client(
    stream: TcpStream,
    hub: Arc<Mutex<Hub>>,
    senders: Senders,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    stream.set_nodelay(true)?;
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let id = hub.lock().unwrap().connect();
    senders.lock().unwrap().insert(id, tx);

    let writer = tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            if sink.send(m).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = source.next().await {
        let out = match msg? {
            Message::Text(t) => match decode::<ClientMsg>(&t) {
                Some(m) => hub.lock().unwrap().handle(id, m, now_ms()),
                None => continue,
            },
            Message::Binary(b) => hub.lock().unwrap().handle_audio(id, b.to_vec()),
            Message::Close(_) => break,
            _ => continue,
        };
        deliver(&senders, out);
    }

    senders.lock().unwrap().remove(&id);
    let out = hub.lock().unwrap().disconnect(id, now_ms());
    deliver(&senders, out);
    writer.abort();
    Ok(())
}
