use std::sync::{Mutex, Once};

const BRIDGE: &str = "http://127.0.0.1:4747/now";

#[derive(Default, Clone)]
struct Feed {
    serato: String,
    decks: Vec<(String, String)>,
    heard: String,
    next: Vec<String>,
}

static FEED: Mutex<Option<Feed>> = Mutex::new(None);
static START: Once = Once::new();

fn read_feed(v: &ureq::serde_json::Value) -> Feed {
    let text = |x: &ureq::serde_json::Value| x.as_str().unwrap_or("").to_string();
    let heard = v.get("heard").and_then(|h| h.get("track")).map(text).unwrap_or_default();
    Feed {
        serato: v.get("track").map(text).unwrap_or_default(),
        decks: v.get("decks").and_then(|d| d.as_array()).map(|a| a.iter().map(|d| (d.get("deck").map(|x| x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string())).unwrap_or_default(), d.get("track").map(text).unwrap_or_default())).collect()).unwrap_or_default(),
        next: v.get("next").and_then(|d| d.as_array()).map(|a| a.iter().map(|n| n.get("track").map(text).unwrap_or_default()).filter(|t| !t.is_empty()).collect()).unwrap_or_default(),
        heard,
    }
}

fn feed() -> Option<Feed> {
    START.call_once(|| {
        std::thread::spawn(|| loop {
            let agent = ureq::AgentBuilder::new().timeout(std::time::Duration::from_millis(800)).build();
            let got = agent.get(BRIDGE).call().ok().and_then(|r| r.into_json::<ureq::serde_json::Value>().ok()).map(|v| read_feed(&v));
            let away = got.is_none();
            if let Ok(mut f) = FEED.lock() {
                *f = got;
            }
            std::thread::sleep(std::time::Duration::from_secs(if away { 5 } else { 2 }));
        });
    });
    FEED.lock().ok().and_then(|f| f.clone())
}

pub fn now_playing() -> String {
    let Some(f) = feed() else { return String::new() };
    let live = if flag(0) { if f.serato.is_empty() { f.heard } else { f.serato } } else { f.heard };
    if live.is_empty() { String::new() } else { format!("LIVE {live}") }
}

pub fn online() -> bool {
    feed().is_some()
}

pub fn bpm() -> f32 {
    0.0
}

pub fn decks() -> Vec<(String, String)> {
    match feed() {
        Some(f) if flag(0) && f.decks.len() > 1 => f.decks,
        _ => Vec::new(),
    }
}

pub fn next() -> Vec<String> {
    let Some(f) = feed() else { return Vec::new() };
    let mut out: Vec<String> = if f.heard.is_empty() { Vec::new() } else { vec![format!("={}", f.heard)] };
    out.extend(f.next);
    out
}

pub fn overlay(_y: f32) {}

pub fn karaoke_here() -> bool {
    false
}

static FLAGS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);

pub fn flag(i: i32) -> bool {
    FLAGS.load(std::sync::atomic::Ordering::Relaxed) & (1 << i) != 0
}

pub fn set_flag(i: i32, on: bool) {
    let bit = 1u8 << i;
    if on {
        FLAGS.fetch_or(bit, std::sync::atomic::Ordering::Relaxed);
    } else {
        FLAGS.fetch_and(!bit, std::sync::atomic::Ordering::Relaxed);
    }
}

pub fn touch_ui() -> bool {
    false
}

pub fn report(_text: &str) {}

pub fn ask_text(_prompt: &str, _current: &str) -> Option<String> {
    None
}
