#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn net_open(url: *const u8, len: u32);
    fn net_send(text: *const u8, len: u32);
    fn net_state() -> i32;
    fn net_recv_len() -> i32;
    fn clip_paste_len() -> i32;
    fn clip_paste_get(out: *mut u8);
    fn clip_write(text: *const u8, len: u32);
    fn net_recv_get(out: *mut u8);
    fn net_server_len() -> i32;
    fn net_server_get(out: *mut u8);
    fn net_open_url(url: *const u8, len: u32);
    fn net_listen(url: *const u8, len: u32);
}

pub fn server() -> Option<String> {
    let len = unsafe { net_server_len() };
    if len <= 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { net_server_get(buf.as_mut_ptr()) };
    String::from_utf8(buf).ok()
}

pub fn open(url: &str) {
    unsafe { net_open(url.as_ptr(), url.len() as u32) };
}

pub fn state() -> i32 {
    unsafe { net_state() }
}

pub fn send(text: &str) {
    unsafe { net_send(text.as_ptr(), text.len() as u32) };
}

pub fn recv() -> Option<String> {
    let len = unsafe { net_recv_len() };
    if len < 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    if len > 0 {
        unsafe { net_recv_get(buf.as_mut_ptr()) };
    }
    String::from_utf8(buf).ok()
}

pub fn copy(text: &str) {
    macroquad::miniquad::window::clipboard_set(text);
    unsafe { clip_write(text.as_ptr(), text.len() as u32) };
}

pub fn paste_now() -> Option<String> {
    None
}

pub fn pasted() -> Option<String> {
    let len = unsafe { clip_paste_len() };
    if len <= 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { clip_paste_get(buf.as_mut_ptr()) };
    String::from_utf8(buf).ok()
}

pub fn open_url(url: &str) {
    if url.starts_with("https://") {
        unsafe { net_open_url(url.as_ptr(), url.len() as u32) };
    }
}

pub fn listen(url: &str) {
    unsafe { net_listen(url.as_ptr(), url.len() as u32) };
}

pub fn can_listen() -> bool {
    true
}

pub fn inputs() -> Vec<String> {
    Vec::new()
}

pub fn can_cast() -> bool {
    false
}

pub fn casting() -> bool {
    false
}

pub fn cast_level() -> Option<f32> {
    None
}

pub fn cast_stop() {}

pub fn cast_start(_device: &str) -> Result<(), String> {
    Err("streaming from this computer needs the desktop app".to_string())
}
