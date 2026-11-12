//! Starts the real server and plays a DJ and a fan against it.

use mq_shared::{decode, encode, ClientMsg, Scene, ServerMsg, Sound, PROTOCOL_VERSION};
use std::net::TcpStream;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

type Ws = WebSocket<MaybeTlsStream<TcpStream>>;

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

fn connect(port: u16) -> Ws {
    let start = Instant::now();
    loop {
        match tungstenite::connect(format!("ws://127.0.0.1:{port}")) {
            Ok((ws, _)) => {
                if let MaybeTlsStream::Plain(s) = ws.get_ref() {
                    s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                }
                return ws;
            }
            Err(_) if start.elapsed() < Duration::from_secs(10) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => panic!("server never came up: {e}"),
        }
    }
}

fn send(ws: &mut Ws, m: ClientMsg) {
    ws.send(Message::text(encode(&m))).unwrap();
}

/// Read until `f` matches a JSON message.
fn wait<T>(ws: &mut Ws, mut f: impl FnMut(&ServerMsg) -> Option<T>) -> T {
    loop {
        match ws.read().expect("read") {
            Message::Text(t) => {
                if let Some(m) = decode::<ServerMsg>(&t) {
                    if let Some(v) = f(&m) {
                        return v;
                    }
                }
            }
            _ => {}
        }
    }
}

fn wait_binary(ws: &mut Ws) -> Vec<u8> {
    loop {
        if let Message::Binary(b) = ws.read().expect("read") {
            return b.to_vec();
        }
    }
}

#[test]
fn dj_hosts_fan_joins_by_code_and_hears_the_set() {
    let port = 20_000 + (std::process::id() % 20_000) as u16;
    let djs = std::env::temp_dir().join(format!("mq-djs-{port}.txt"));
    std::fs::write(&djs, "# test\nPriori = letmein\n").unwrap();
    let _srv = Server(
        Command::new(env!("CARGO_BIN_EXE_mq-server"))
            .args(["--port", &port.to_string(), "--djs", djs.to_str().unwrap()])
            .spawn()
            .unwrap(),
    );

    let mut dj = connect(port);
    send(&mut dj, ClientMsg::Hello { name: "anything".into(), avatar: 2, dj_key: Some("letmein".into()), version: PROTOCOL_VERSION });
    let (name, is_dj) = wait(&mut dj, |m| match m {
        ServerMsg::Welcome { name, dj, .. } => Some((name.clone(), *dj)),
        _ => None,
    });
    assert_eq!(name, "Priori");
    assert!(is_dj);

    let mut fan = connect(port);
    send(&mut fan, ClientMsg::Hello { name: "fan".into(), avatar: 0, dj_key: None, version: PROTOCOL_VERSION });
    wait(&mut fan, |m| matches!(m, ServerMsg::Welcome { dj: false, .. }).then_some(()));

    // The DJ books a rave from the PC bang.
    send(&mut dj, ClientMsg::HostRave { title: "acid mass".into(), sound: Sound::Gothic, bpm: 132.0, flyers: true, doors_in_min: 0 });
    let code = wait(&mut dj, |m| match m {
        ServerMsg::RaveHosted { rave } => Some(rave.code.clone()),
        _ => None,
    });

    // The fan sees the flyer around town.
    let seen = wait(&mut fan, |m| match m {
        ServerMsg::Flyers { raves } if !raves.is_empty() => Some(raves[0].code.clone()),
        _ => None,
    });
    assert_eq!(seen, code);

    // A wrong code is turned away; the right one, typed sloppily, gets in.
    send(&mut fan, ClientMsg::Enter { scene: Scene::Rave, code: Some("ACD-EFG-HJK".into()) });
    wait(&mut fan, |m| matches!(m, ServerMsg::Denied { .. }).then_some(()));
    send(&mut dj, ClientMsg::Enter { scene: Scene::Rave, code: Some(code.clone()) });
    wait(&mut dj, |m| matches!(m, ServerMsg::Entered { scene: Scene::Rave, .. }).then_some(()));
    send(&mut fan, ClientMsg::Enter { scene: Scene::Rave, code: Some(code.to_lowercase().replace('-', " ")) });
    wait(&mut fan, |m| matches!(m, ServerMsg::Entered { scene: Scene::Rave, .. }).then_some(()));

    // DJ goes live; the fan is told, then gets the audio.
    send(&mut dj, ClientMsg::SetLive { live: true });
    wait(&mut fan, |m| match m {
        ServerMsg::RaveUpdate { rave } if rave.live => Some(()),
        _ => None,
    });
    dj.send(Message::binary(vec![0xF8, 1, 2, 3])).unwrap();
    assert_eq!(wait_binary(&mut fan), vec![0xF8, 1, 2, 3]);

    // Chat stays in the room.
    send(&mut fan, ClientMsg::Chat { text: "this drop!!".into() });
    let text = wait(&mut dj, |m| match m {
        ServerMsg::Chat { text, .. } => Some(text.clone()),
        _ => None,
    });
    assert_eq!(text, "this drop!!");
}
