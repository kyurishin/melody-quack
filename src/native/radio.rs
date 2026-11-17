use macroquad::prelude::*;
use std::sync::{Mutex, Once};

const BRIDGE: &str = "http://127.0.0.1:4747/now";

#[derive(Default, Clone)]
struct Feed {
    serato: String,
    decks: Vec<(String, String)>,
    heard: String,
    next: Vec<String>,
    lyrics: Vec<(f64, String)>,
    sync: Option<(f64, f64)>,
}

static FEED: Mutex<Option<Feed>> = Mutex::new(None);
static START: Once = Once::new();
static OVERLAY_Y: Mutex<f32> = Mutex::new(0.0);
static CJK: Mutex<Option<Option<Font>>> = Mutex::new(None);

fn epoch() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

fn read_feed(v: &ureq::serde_json::Value) -> Feed {
    let text = |x: &ureq::serde_json::Value| x.as_str().unwrap_or("").to_string();
    let heard = v.get("heard").and_then(|h| h.get("track")).map(text).unwrap_or_default();
    Feed {
        serato: v.get("track").map(text).unwrap_or_default(),
        decks: v.get("decks").and_then(|d| d.as_array()).map(|a| a.iter().map(|d| (d.get("deck").map(|x| x.as_str().map(|s| s.to_string()).unwrap_or_else(|| x.to_string())).unwrap_or_default(), d.get("track").map(text).unwrap_or_default())).collect()).unwrap_or_default(),
        next: v.get("next").and_then(|d| d.as_array()).map(|a| a.iter().map(|n| n.get("track").map(text).unwrap_or_default()).filter(|t| !t.is_empty()).collect()).unwrap_or_default(),
        lyrics: v.get("lyrics").and_then(|d| d.as_array()).map(|a| a.iter().filter_map(|l| Some((l.get(0)?.as_f64()?, l.get(1).map(text).unwrap_or_default()))).collect()).unwrap_or_default(),
        sync: v.get("sync").and_then(|s| Some((s.get("pos")?.as_f64()?, s.get("stamp")?.as_f64()?))),
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

pub fn overlay(y: f32) {
    if let Ok(mut o) = OVERLAY_Y.lock() {
        *o = y;
    }
}

fn karaoke_lines(f: &Feed, now: f64) -> (String, String, f32) {
    let Some((pos0, stamp)) = f.sync else { return (String::new(), String::new(), 0.0) };
    if f.lyrics.is_empty() || f.heard.is_empty() {
        return (String::new(), String::new(), 0.0);
    }
    let pos = pos0 + (now - stamp);
    let at = f.lyrics.iter().rposition(|l| l.0 <= pos);
    match at {
        Some(i) if pos - f.lyrics[i].0 < 30.0 => {
            let end = f.lyrics.get(i + 1).map(|l| l.0).unwrap_or(f.lyrics[i].0 + 5.0);
            let fill = ((pos - f.lyrics[i].0) / (end - f.lyrics[i].0).max(0.5)).clamp(0.0, 1.0) as f32;
            let now_line = if f.lyrics[i].1.is_empty() { "\u{266a}".to_string() } else { f.lyrics[i].1.clone() };
            (now_line, f.lyrics.get(i + 1).map(|l| l.1.clone()).unwrap_or_default(), fill)
        }
        None if f.lyrics[0].0 - pos < 8.0 => ("\u{266a}".to_string(), f.lyrics[0].1.clone(), 0.0),
        _ => (String::new(), String::new(), 0.0),
    }
}

fn wide_font() -> Option<Font> {
    let mut slot = CJK.lock().ok()?;
    if slot.is_none() {
        let paths = [
            "/System/Library/Fonts/AppleSDGothicNeo.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        let font = paths.iter().find_map(|p| std::fs::read(p).ok().and_then(|b| load_ttf_font_from_bytes(&b).ok()));
        *slot = Some(font);
    }
    slot.clone().flatten()
}

fn line(text: &str, cx: f32, y: f32, size: u16, font: Option<&Font>, base: Color, fill: Option<(f32, Color)>) {
    let dims = measure_text(text, font, size, 1.0);
    let x = (cx - dims.width / 2.0).round();
    let params = |color: Color| TextParams { font, font_size: size, color, ..Default::default() };
    let shadow = (size as f32 / 14.0).max(1.0).round();
    draw_text_ex(text, x + shadow, y + shadow, params(Color::new(0.05, 0.04, 0.1, 0.8)));
    draw_text_ex(text, x, y, params(base));
    if let Some((amount, color)) = fill {
        if amount > 0.0 {
            let cut = (dims.width * amount).round();
            let mut done = String::new();
            for ch in text.chars() {
                done.push(ch);
                if measure_text(&done, font, size, 1.0).width > cut {
                    done.pop();
                    break;
                }
            }
            if !done.is_empty() {
                draw_text_ex(&done, x, y, params(color));
            }
        }
    }
}

pub fn draw_overlay() {
    let place = OVERLAY_Y.lock().map(|o| *o).unwrap_or(0.0);
    if place <= 0.0 {
        return;
    }
    let Some(f) = feed() else { return };
    let show_song = !f.heard.is_empty() && !flag(0);
    let (now_line, next_line, fill) = if flag(1) { karaoke_lines(&f, epoch()) } else { (String::new(), String::new(), 0.0) };
    if !show_song && now_line.is_empty() {
        return;
    }
    let font = wide_font();
    let note = if font.is_some() { "\u{266a}" } else { "~" };
    let now_line = if now_line == "\u{266a}" { note.to_string() } else { now_line };
    let font = font.as_ref();
    let unit = (screen_height() / 270.0).max(1.0);
    let cx = screen_width() / 2.0;
    let mut y = (screen_height() * place).round();
    let yellow = Color::from_rgba(255, 214, 110, 255);
    if show_song {
        line(&format!("{note} {}", f.heard), cx, y, (7.0 * unit) as u16, font, Color::new(1.0, 0.97, 0.88, 0.85), None);
        y += 12.0 * unit;
    }
    if !now_line.is_empty() {
        line(&now_line, cx, y, (11.0 * unit) as u16, font, WHITE, Some((fill, yellow)));
        y += 11.0 * unit;
        if !next_line.is_empty() {
            line(&next_line, cx, y, (8.0 * unit) as u16, font, Color::new(1.0, 1.0, 1.0, 0.6), None);
        }
    }
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
