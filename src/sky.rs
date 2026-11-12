#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

pub(crate) const SKY_TOP: Color = Color::new(0.55, 0.74, 0.86, 1.0);

pub(crate) const FOG: Color = Color::new(0.80, 0.89, 0.84, 1.0);

pub(crate) const DUSK_FOG: Color = Color::new(0.36, 0.3, 0.48, 1.0);

#[derive(Clone, Copy)]
pub(crate) struct Sky {
    pub(crate) night: f32,
    pub(crate) glow: f32,
    pub(crate) hour: f32,
    pub(crate) day: i64,
    pub(crate) season: usize,
    pub(crate) weather: usize,
    pub(crate) temp: Option<f32>,
}

pub(crate) const SEASONS: [&str; 4] = ["spring", "summer", "autumn", "winter"];

pub(crate) const WEATHERS: [&str; 4] = ["clear", "cloudy", "rain", "snow"];

thread_local! {
    pub(crate) static SKY: std::cell::Cell<Sky> = std::cell::Cell::new(Sky { night: 0.0, glow: 0.0, hour: 12.0, day: 0, season: 1, weather: 0, temp: None });
}

pub(crate) fn sky() -> Sky {
    SKY.with(|s| s.get())
}

pub(crate) fn civil_month(day: i64) -> (u32, u32) {
    let z = day + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (m, d)
}

pub(crate) fn update_sky() -> Sky {
    let now = clock::now();
    let offset = clock::offset();
    let local = now + offset;
    let day = (local / 86400.0).floor() as i64;
    let hour = ((local - day as f64 * 86400.0) / 3600.0) as f32;
    let (month, mday) = civil_month(day);
    let lat = clock::lat();
    let south = lat < 0.0;
    let m = if south { (month + 5) % 12 + 1 } else { month };
    let season = match m {
        3..=5 => 0,
        6..=8 => 1,
        9..=11 => 2,
        _ => 3,
    };
    let doy = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(month - 1) as usize] as f32 + mday as f32;
    let (sunrise, sunset) = {
        let (a, b) = (clock::sun(0), clock::sun(1));
        if a > 0.0 && b > 0.0 {
            (((a + offset).rem_euclid(86400.0) / 3600.0) as f32, ((b + offset).rem_euclid(86400.0) / 3600.0) as f32)
        } else {
            let decl = 23.44f32.to_radians() * (2.0 * PI * (284.0 + doy) / 365.0).sin();
            let cos_h = (-(lat.to_radians().tan()) * decl.tan()).clamp(-1.0, 1.0);
            let half = cos_h.acos().to_degrees() / 15.0;
            let noon = 12.3 + (offset as f32 / 3600.0 - 1.0).clamp(-1.0, 1.0);
            (noon - half, noon + half)
        }
    };
    let ramp = |x: f32| x.clamp(0.0, 1.0);
    let wrap = |x: f32| (x + 12.0).rem_euclid(24.0) - 12.0;
    let day_len = (sunset - sunrise).rem_euclid(24.0);
    let pos = (hour - sunrise).rem_euclid(24.0);
    let edge = if pos < day_len { pos.min(day_len - pos) } else { -(pos - day_len).min(24.0 - pos) };
    let night = ramp((0.4 - edge) / 1.1);
    let glow = (1.0 - wrap(hour - sunset).abs() / 0.9).max(0.0) + 0.7 * (1.0 - wrap(hour - sunrise).abs() / 0.8).max(0.0);
    let live_weather = clock::weather();
    let weather = if live_weather >= 0 {
        live_weather as usize
    } else {
        let roll = hash(day as i32, (hour / 3.0) as i32, 77);
        let table: [f32; 3] = match season {
            0 => [0.4, 0.7, 1.0],
            1 => [0.55, 0.8, 1.0],
            2 => [0.25, 0.6, 1.0],
            _ => [0.2, 0.5, 0.75],
        };
        if roll < table[0] { 0 } else if roll < table[1] { 1 } else if roll < table[2] { 2 } else { 3 }
    };
    let weather = if weather == 3 && season == 1 { 2 } else { weather };
    let out = Sky { night, glow: glow.min(1.0), hour, day, season, weather, temp: clock::temp() };
    SKY.with(|s| s.set(out));
    out
}

pub(crate) fn sky_haze(c: Color, dusk: f32) -> Color {
    let s = sky();
    let mut c = c;
    if s.glow > 0.0 {
        c = mix(c, Color::new(1.0, 0.6, 0.48, c.a), s.glow * 0.45 * (1.0 - dusk * 0.4));
    }
    match s.weather {
        1 => mix(c, Color::new(0.62, 0.64, 0.68, c.a), 0.25),
        2 => mix(c, Color::new(0.45, 0.5, 0.58, c.a), 0.38),
        3 => mix(c, Color::new(0.88, 0.9, 0.95, c.a), 0.3),
        _ => c,
    }
}
