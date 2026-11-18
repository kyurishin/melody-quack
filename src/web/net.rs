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
    fn cast_inputs_request();
    fn cast_inputs_state() -> i32;
    fn cast_inputs_len() -> i32;
    fn cast_inputs_get(out: *mut u8);
    #[link_name = "cast_start"]
    fn js_cast_start(index: u32) -> i32;
    #[link_name = "cast_stop"]
    fn js_cast_stop();
    fn cast_state() -> i32;
    #[link_name = "cast_level"]
    fn js_cast_level() -> f32;
    fn cast_error_len() -> i32;
    fn cast_error_get(out: *mut u8);
}

fn js_text(len: i32, get: unsafe extern "C" fn(*mut u8)) -> String {
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { get(buf.as_mut_ptr()) };
    String::from_utf8(buf).unwrap_or_default()
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
    if unsafe { cast_inputs_state() } != 2 {
        unsafe { cast_inputs_request() };
        return Vec::new();
    }
    js_text(unsafe { cast_inputs_len() }, cast_inputs_get).lines().map(|l| l.to_string()).filter(|l| !l.is_empty()).collect()
}

pub fn inputs_note() -> Option<String> {
    match unsafe { cast_inputs_state() } {
        2 => None,
        -1 => Some("the browser said no. allow the microphone for this site (the icon next to the address), then try again.".to_string()),
        _ => Some("your browser is asking to use your sound inputs. say allow, and they'll show up here.".to_string()),
    }
}

pub fn can_cast() -> bool {
    true
}

pub fn casting() -> bool {
    (unsafe { cast_state() }) > 0
}

pub fn cast_level() -> Option<f32> {
    let v = unsafe { js_cast_level() };
    if v < 0.0 { None } else { Some(v) }
}

pub fn cast_error() -> Option<String> {
    if (unsafe { cast_state() }) == -1 {
        Some(js_text(unsafe { cast_error_len() }, cast_error_get))
    } else {
        None
    }
}

pub fn cast_stop() {
    unsafe { js_cast_stop() };
}

pub fn cast_start(device: &str) -> Result<(), String> {
    let list = inputs();
    let Some(i) = list.iter().position(|d| d == device) else {
        return Err(format!("{device} isn't there right now"));
    };
    if unsafe { js_cast_start(i as u32) } == 1 { Ok(()) } else { Err("that input couldn't start".to_string()) }
}
