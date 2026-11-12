//! Diary entries, stars and dates.

#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone)]
pub(crate) struct DiaryEntry {
    pub(crate) when: f64,
    pub(crate) seconds: u32,
    pub(crate) stars: u8,
    pub(crate) who: (usize, usize),
    pub(crate) events: Vec<(f32, u16, f32)>,
}

pub(crate) fn stars_for(seconds: f32) -> u8 {
    if seconds >= 180.0 {
        3
    } else if seconds >= 60.0 {
        2
    } else if seconds >= 30.0 {
        1
    } else {
        0
    }
}

pub(crate) fn date_label(epoch: f64) -> String {
    let days = (epoch / 86400.0).floor() as i64;
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let names = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    format!("{} {}", names[(month - 1).clamp(0, 11) as usize], day)
}
