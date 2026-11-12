#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn radio_text_len() -> i32;
    fn radio_text_get(out: *mut u8);
    fn radio_decks_len() -> i32;
    fn radio_decks_get(out: *mut u8);
    fn radio_next_len() -> i32;
    fn live_flag_get(i: i32) -> i32;
    fn live_flag_set(i: i32, on: i32);
    fn live_overlay(y: f32);
    fn radio_next_get(out: *mut u8);
    fn radio_bpm() -> f32;
    fn touch_mode() -> i32;
    fn ui_report(text: *const u8, len: u32);
    fn ask_text_open(prompt: *const u8, prompt_len: u32, current: *const u8, current_len: u32) -> i32;
    fn ask_text_get(out: *mut u8);
}

pub fn ask_text(prompt: &str, current: &str) -> Option<String> {
    let len = unsafe { ask_text_open(prompt.as_ptr(), prompt.len() as u32, current.as_ptr(), current.len() as u32) };
    if len < 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    if len > 0 {
        unsafe { ask_text_get(buf.as_mut_ptr()) };
    }
    String::from_utf8(buf).ok()
}

pub fn report(text: &str) {
    unsafe { ui_report(text.as_ptr(), text.len() as u32) };
}

pub fn touch_ui() -> bool {
    unsafe { touch_mode() != 0 }
}

pub fn now_playing() -> String {
    let len = unsafe { radio_text_len() };
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { radio_text_get(buf.as_mut_ptr()) };
    String::from_utf8(buf).unwrap_or_default()
}

pub fn decks() -> Vec<(String, String)> {
    let len = unsafe { radio_decks_len() };
    if len <= 0 {
        return Vec::new();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { radio_decks_get(buf.as_mut_ptr()) };
    String::from_utf8(buf)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| line.split_once('\t').map(|(d, t)| (d.to_string(), t.to_string())))
        .collect()
}

pub fn bpm() -> f32 {
    unsafe { radio_bpm() }
}

pub fn flag(i: i32) -> bool {
    unsafe { live_flag_get(i) != 0 }
}

pub fn set_flag(i: i32, on: bool) {
    unsafe { live_flag_set(i, on as i32) };
}

pub fn overlay(y: f32) {
    unsafe { live_overlay(y) };
}

pub fn next() -> Vec<String> {
    let len = unsafe { radio_next_len() };
    if len <= 0 {
        return Vec::new();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { radio_next_get(buf.as_mut_ptr()) };
    String::from_utf8(buf).unwrap_or_default().lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_string()).collect()
}
