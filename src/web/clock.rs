#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn sky_now() -> f64;
    fn sky_offset() -> f64;
    fn sky_weather() -> i32;
    fn sky_temp() -> f32;
    fn sky_sun(i: i32) -> f64;
    fn sky_lat() -> f32;
}

pub fn now() -> f64 {
    unsafe { sky_now() }
}

pub fn offset() -> f64 {
    unsafe { sky_offset() }
}

pub fn weather() -> i32 {
    unsafe { sky_weather() }
}

pub fn temp() -> Option<f32> {
    let t = unsafe { sky_temp() };
    if t < -500.0 { None } else { Some(t) }
}

pub fn sun(i: i32) -> f64 {
    unsafe { sky_sun(i) }
}

pub fn lat() -> f32 {
    unsafe { sky_lat() }
}
