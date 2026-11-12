#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn net_open(url: *const u8, len: u32);
    fn net_send(text: *const u8, len: u32);
    fn net_state() -> i32;
    fn net_recv_len() -> i32;
    fn net_recv_get(out: *mut u8);
    fn net_server_len() -> i32;
    fn net_server_get(out: *mut u8);
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
