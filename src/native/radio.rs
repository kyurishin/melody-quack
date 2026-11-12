pub fn now_playing() -> String {
    String::new()
}

pub fn bpm() -> f32 {
    0.0
}

pub fn decks() -> Vec<(String, String)> {
    Vec::new()
}

pub fn next() -> Vec<String> {
    Vec::new()
}

pub fn overlay(_y: f32) {}

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
