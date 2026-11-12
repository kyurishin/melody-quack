#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn store_put(key: *const u8, key_len: u32, value: *const u8, value_len: u32);
    fn store_len(key: *const u8, key_len: u32) -> i32;
    fn store_get(key: *const u8, key_len: u32, out: *mut u8);
    fn store_remove(key: *const u8, key_len: u32);
    fn download_bytes(name: *const u8, name_len: u32, data: *const u8, data_len: u32);
    fn app_restart();
}

#[unsafe(no_mangle)]
pub extern "C" fn melody_store_crate_version() -> u32 {
    1
}

fn get(key: &str) -> Option<String> {
    let len = unsafe { store_len(key.as_ptr(), key.len() as u32) };
    if len < 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    unsafe { store_get(key.as_ptr(), key.len() as u32, buf.as_mut_ptr()) };
    String::from_utf8(buf).ok()
}

fn put(key: &str, value: &str) {
    unsafe { store_put(key.as_ptr(), key.len() as u32, value.as_ptr(), value.len() as u32) };
}

pub fn read() -> Option<String> {
    get("melody-quack-save")
}

pub fn write(text: &str) {
    put("melody-quack-save", text);
}

pub fn clear() {
    let key = "melody-quack-save";
    unsafe { store_remove(key.as_ptr(), key.len() as u32) };
}

pub fn restart() {
    unsafe { app_restart() };
}

pub fn save_jam(bytes: &[u8]) -> Option<String> {
    let n = get("melody-quack-jams").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0) + 1;
    put("melody-quack-jams", &n.to_string());
    let name = format!("melody-quack-jam-{n:02}.wav");
    unsafe { download_bytes(name.as_ptr(), name.len() as u32, bytes.as_ptr(), bytes.len() as u32) };
    Some(format!("downloads/{name}"))
}
